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
        visit_item(self, item);
        self.base.pop();
    }

    fn visit_foreign_item_fn(&mut self, item: &'ast syn::ForeignItemFn) {
        self.report_fn(&item.sig);
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.report_fn(&item.sig);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        self.report_fn(&item.sig);
    }

    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        self.report_fn(&item.sig);
    }
}

impl Rule {
    fn report_fn(&mut self, sig: &syn::Signature) {
        let name = format!("{}", sig.ident);
        let span = sig.ident.span();

        self.base.add_finding(ScanFinding {
            text: format!("Found function: {}", name),
            start: span.start().line,
            end: span.end().line,
        })
    }
}

fn main() {
    run_from_args(Rule::default());
}
