use std::fs;
use syn::{visit::Visit, File, ItemStruct};

struct Auditor;

impl<'ast> Visit<'ast> for Auditor {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        println!("Found struct: {}", node.ident);
        syn::visit::visit_item_struct(self, node);
    }
}

fn main() {
    let code = fs::read_to_string("sample.rs").unwrap();
    let ast: File = syn::parse_file(&code).unwrap();

    let mut auditor = Auditor;
    auditor.visit_file(&ast);
}
