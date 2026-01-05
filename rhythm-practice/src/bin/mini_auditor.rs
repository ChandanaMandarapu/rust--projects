use std::fs;
use colored::*;
use serde::Serialize;
use syn::{
    visit::Visit,
    File, ItemStruct, ItemFn, Attribute, ExprBinary, BinOp, Type, FnArg,
};
use quote::ToTokens;

/* ============================
   DATA MODELS
============================ */

#[derive(Debug, Serialize)]
enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Serialize)]
struct Finding {
    severity: Severity,
    title: String,
    description: String,
    location: String,
    recommendation: String,
}

#[derive(Default)]
struct AuditContext {
    findings: Vec<Finding>,
}

/* ============================
   MAIN VISITOR
============================ */

struct Auditor {
    ctx: AuditContext,
}

impl Auditor {
    fn new() -> Self {
        Self {
            ctx: AuditContext::default(),
        }
    }

    fn report(&self) {
        println!("\n{}", "=== AUDIT REPORT ===".bold());

        for f in &self.ctx.findings {
            let sev = match f.severity {
                Severity::Critical => "CRITICAL".red(),
                Severity::High => "HIGH".bright_red(),
                Severity::Medium => "MEDIUM".yellow(),
                Severity::Low => "LOW".blue(),
            };

            println!("\n{} {}", sev, f.title.bold());
            println!("  Location: {}", f.location);
            println!("  Issue: {}", f.description);
            println!("  Fix: {}", f.recommendation.green());
        }

        println!(
            "\n{} {} findings\n",
            "Total:".bold(),
            self.ctx.findings.len()
        );
    }
}

/* ============================
   VISIT IMPLEMENTATION
============================ */

impl<'ast> Visit<'ast> for Auditor {

    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        let mut is_account = false;
        let mut is_mut = false;
        let mut has_signer = false;

        for attr in &node.attrs {
            if is_account_attr(attr) {
                is_account = true;

                let tokens = attr.meta.to_token_stream().to_string();

                if tokens.contains("mut") {
                    is_mut = true;
                }
                if tokens.contains("signer") {
                    has_signer = true;
                }
            }
        }

        if is_account && is_mut && !has_signer {
            self.ctx.findings.push(Finding {
                severity: Severity::Critical,
                title: "Missing signer on mutable account".into(),
                description: format!(
                    "Account struct `{}` is mutable but has no signer constraint.",
                    node.ident
                ),
                location: format!("struct {}", node.ident),
                recommendation: "Add `signer` to #[account(...)] constraints.".into(),
            });
        }

        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        for input in &node.sig.inputs {
            if let FnArg::Typed(pat) = input {
                if let Type::Path(tp) = &*pat.ty {
                    let ty = tp.to_token_stream().to_string();
                    if ty.contains("AccountInfo") {
                        self.ctx.findings.push(Finding {
                            severity: Severity::High,
                            title: "Unchecked AccountInfo usage".into(),
                            description: format!(
                                "Function `{}` accepts raw AccountInfo.",
                                node.sig.ident
                            ),
                            location: format!("fn {}", node.sig.ident),
                            recommendation:
                                "Avoid AccountInfo; use typed Anchor accounts with constraints."
                                    .into(),
                        });
                    }
                }
            }
        }

        syn::visit::visit_item_fn(self, node);
    }

    fn visit_expr_binary(&mut self, node: &'ast ExprBinary) {
        match node.op {
            BinOp::Add(_) | BinOp::Sub(_) | BinOp::Mul(_) => {
                let expr = node.to_token_stream().to_string();

                if expr.contains("balance")
                    || expr.contains("amount")
                    || expr.contains("reward")
                {
                    self.ctx.findings.push(Finding {
                        severity: Severity::Medium,
                        title: "Unsafe arithmetic on financial value".into(),
                        description: format!(
                            "Arithmetic expression `{}` is unchecked.",
                            expr
                        ),
                        location: "expression".into(),
                        recommendation:
                            "Use checked_add / checked_sub or explicit overflow handling."
                                .into(),
                    });
                }
            }
            _ => {}
        }

        syn::visit::visit_expr_binary(self, node);
    }
}

/* ============================
   HELPERS
============================ */

fn is_account_attr(attr: &Attribute) -> bool {
    attr.path().is_ident("account")
}

/* ============================
   MAIN
============================ */

fn main() {
    println!("{}", "Mini Solana Security Auditor".bold());

    // 🔒 BULLETPROOF INPUT HANDLING (BOM + WINDOWS SAFE)
    let bytes = fs::read("sample.rs")
        .expect("Failed to read sample.rs");

    let mut code = String::from_utf8_lossy(&bytes).to_string();

    // strip BOM if present
    if code.starts_with('\u{feff}') {
        code = code.trim_start_matches('\u{feff}').to_string();
    }

    let ast: File = syn::parse_file(&code)
        .expect("Failed to parse Rust file");

    let mut auditor = Auditor::new();
    auditor.visit_file(&ast);

    auditor.report();

    let json = serde_json::to_string_pretty(&auditor.ctx.findings)
        .expect("json");
    fs::write("audit_report.json", json)
        .expect("write json");

    println!("{}", "Report written to audit_report.json".green());
}
