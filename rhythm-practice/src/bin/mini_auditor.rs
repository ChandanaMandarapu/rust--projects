use std::fs;
use syn::{
    visit::Visit, File, ExprBinary, BinOp,
};
use quote::ToTokens;

struct Auditor;

impl<'ast> Visit<'ast> for Auditor {
    fn visit_expr_binary(&mut self, node: &'ast ExprBinary) {
        match node.op {
            BinOp::Add(_) | BinOp::Sub(_) | BinOp::Mul(_) => {
                let expr = node.to_token_stream().to_string();
                if expr.contains("balance") || expr.contains("amount") {
                    println!("MEDIUM: unsafe arithmetic -> {}", expr);
                }
            }
            _ => {}
        }
        syn::visit::visit_expr_binary(self, node);
    }
}

fn main() {
    let code = fs::read_to_string("sample.rs").unwrap();
    let ast: File = syn::parse_file(&code).unwrap();
    Auditor.visit_file(&ast);
}
