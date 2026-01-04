use syn::{
    visit::Visit, Attribute, ItemStruct, Meta,
};
use std::fs;
use quote::ToTokens;

struct AnchorAttrVisitor;

impl<'ast> Visit<'ast> for AnchorAttrVisitor {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        for attr in &node.attrs {
            if is_account_attr(attr) {
                println!("Struct: {}", node.ident);
                parse_account_attr(attr);
            }
        }

        syn::visit::visit_item_struct(self, node);
    }
}

fn is_account_attr(attr: &Attribute) -> bool {
    attr.path().is_ident("account")
}

fn parse_account_attr(attr: &Attribute) {
    match attr.parse_meta() {
        Ok(Meta::List(meta_list)) => {
            // Convert tokens → string
            let token_str = meta_list.tokens.to_string();

            // Split constraints naively (YOU will improve this later)
            for part in token_str.split(',') {
                println!("  Constraint: {}", part.trim());
            }
        }
        _ => {}
    }
}

fn main() {
    let code = fs::read_to_string("sample.rs")
        .expect("failed to read sample.rs");

    let ast = syn::parse_file(&code)
        .expect("failed to parse rust file");

    let mut visitor = AnchorAttrVisitor;
    visitor.visit_file(&ast);
}
