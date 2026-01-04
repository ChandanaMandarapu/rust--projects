use syn::{visit::Visit, File, ItemFn, ItemStruct};
use std::fs;

struct AstVisitor;

impl<'ast> Visit<'ast> for AstVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        println!("Function: {}", node.sig.ident);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        println!("Struct: {}", node.ident);
        syn::visit::visit_item_struct(self, node);
    }
}

fn main() {
    let code = fs::read_to_string("sample.rs")
        .expect("Failed to read sample.rs");

    let ast: File = syn::parse_file(&code)
        .expect("Failed to parse Rust file");

    let mut visitor = AstVisitor;
    visitor.visit_file(&ast);
}
