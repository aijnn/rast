use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::vec::IntoIter;
use syn::{parse_file, visit::Visit, Item};

const CLR_CLEAR: &str = "\x1b[0m";
const CLR_EMPH: &str = "\x1b[93m";
const CLR_FIND: &str = "\x1b[91m";

pub struct DirIter {
    stack: Vec<IntoIter<PathBuf>>,
}

impl DirIter {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let initial_iter = vec![path.as_ref().to_path_buf()].into_iter();
        DirIter {
            stack: vec![initial_iter],
        }
    }
}

impl Iterator for DirIter {
    type Item = Result<PathBuf, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(iter) = self.stack.last_mut() {
                if let Some(next_path) = iter.next() {
                    if fs::symlink_metadata(&next_path)
                        .ok()?
                        .file_type()
                        .is_symlink()
                    {
                        continue;
                    }

                    if next_path.is_dir() {
                        let entries_result = fs::read_dir(&next_path).map(|entry_iter| {
                            entry_iter
                                .map(|entry_result| entry_result.map(|entry| entry.path()))
                                .collect::<Result<Vec<_>, _>>()
                        });

                        match entries_result {
                            Ok(Ok(entries)) => {
                                self.stack.push(entries.into_iter());
                            }
                            Ok(Err(err)) => return Some(Err(err)),
                            Err(err) => return Some(Err(err)),
                        }
                    }
                    return Some(Ok(next_path));
                } else {
                    self.stack.pop();
                }
            } else {
                return None;
            }
        }
    }
}

pub struct Scanner<T: Findings + for<'ast> Visit<'ast>> {
    rule: T,
}

impl<T: Findings + for<'ast> Visit<'ast>> Scanner<T> {
    pub fn new(rule: T) -> Self {
        Self { rule: rule }
    }

    pub fn scan(&mut self, dir: &str) -> ScanResults {
        let mut results = ScanResults::default();

        for entry in DirIter::new(dir) {
            match entry {
                Ok(path) => {
                    if path.is_dir() {
                        results.total.dirs += 1;
                    } else {
                        results.total.files += 1;

                        if path.extension() == Some(OsStr::new("rs")) {
                            results.total.ast += 1;
                            match self.scan_file(&path) {
                                Ok(n) => results.findings += n,
                                Err(err) => {
                                    if err.downcast_ref::<io::Error>().is_some() {
                                        results.errors.files += 1;
                                    } else {
                                        results.errors.ast += 1;
                                    }
                                }
                            };
                        }
                    }
                }
                Err(_) => results.errors.dirs += 1,
            }
        }

        println!(
            "{}--- Results ---\n\
            Findings: {}\n\
            Dirs:     {} scanned ({} errors)\n\
            Files:    {} scanned ({} errors)\n\
            AST:      {} scanned ({} errors){}",
            CLR_EMPH,
            results.findings,
            results.total.dirs,
            results.errors.dirs,
            results.total.files,
            results.errors.files,
            results.total.ast,
            results.errors.ast,
            CLR_CLEAR
        );

        results
    }

    fn scan_file(&mut self, path: &Path) -> Result<u32, Box<dyn (Error)>> {
        let mut file = fs::File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let ast = parse_file(&content)?;

        self.rule.visit_file(&ast);

        let findings = self.rule.consume_findings();
        let n_findings = findings.len() as u32;

        for finding in findings {
            println!(
                "{}[FINDING] {}: {}{}",
                CLR_FIND,
                path.display(),
                finding.text,
                CLR_CLEAR
            );

            let mut lines = content.lines().skip(finding.start - 1);

            for line_number in finding.start..=finding.end + 2 {
                println!("{:4}: {}", line_number, lines.next().unwrap_or(""));
            }
        }

        Ok(n_findings)
    }
}

#[derive(Default, Clone)]
pub struct ScanFinding {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Default)]
pub struct ScanResults {
    findings: u32,
    total: ScanStats,
    errors: ScanStats,
}

#[derive(Default)]
pub struct ScanStats {
    dirs: u32,
    files: u32,
    ast: u32,
}

#[derive(Default)]
pub struct BaseRule {
    pub findings: Vec<ScanFinding>,
    pub stack: Vec<String>,
}

impl BaseRule {
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
            stack: Vec::new(),
        }
    }

    pub fn add_finding(&mut self, finding: ScanFinding) {
        self.findings.push(finding);
    }

    pub fn get_findings(&self) -> Vec<ScanFinding> {
        self.findings.clone()
    }

    pub fn consume_findings(&mut self) -> Vec<ScanFinding> {
        let findings = self.findings.clone();
        self.findings = Vec::new();
        findings
    }

    pub fn push(&mut self, item: &Item) {
        self.stack.push(BaseRule::get_item_name(&item));
    }

    pub fn pop(&mut self) -> Option<String> {
        self.stack.pop()
    }

    pub fn dump(&self, item: &syn::Item) {
        println!(
            "{}[+]: {}{}\n{:?}",
            CLR_FIND,
            self.stack.join(" -> "),
            CLR_CLEAR,
            item
        );
    }

    pub fn get_item_name(item: &syn::Item) -> String {
        let debug_str = format!("{:?}", item);
        debug_str
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_string()
    }
}

pub trait Findings {
    fn consume_findings(&mut self) -> Vec<ScanFinding>;
}

pub fn run_from_args<T: Findings + for<'ast> Visit<'ast>>(rule: T) {
    let args: Vec<String> = env::args().collect();
    let dir = args.get(1).map(AsRef::as_ref).unwrap_or(".");
    let mut scanner = Scanner::new(rule);

    println!("{}Running rule against: {}{}", CLR_EMPH, dir, CLR_CLEAR);
    let _ = scanner.scan(dir);
}
