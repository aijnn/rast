extern crate rast;
use rast::{run_from_args, BaseRule, Findings, ScanFinding};
use syn::{
    visit::{visit_item, Visit},
    Item,
};

#[derive(Default)]
pub struct Rule {
    base: BaseRule,
}

impl Findings for Rule {
    fn consume_findings(&mut self) -> Vec<ScanFinding> {
        self.base.consume_findings()
    }
}

impl<'ast> Visit<'ast> for Rule {
    fn visit_item(&mut self, item: &'ast Item) {
        self.base.push(&item);
        self.base.dump(&item);
        visit_item(self, item);
        self.base.pop();
    }
}

fn main() {
    run_from_args(Rule::default());
}
