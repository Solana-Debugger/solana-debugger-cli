use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, ExprPath, Item, Stmt};
use syn::spanned::Spanned;
use syn::visit_mut::VisitMut;
use crate::compile::sbf_with_errors::CompileError;

struct CorrectContext {
    errors: Vec<CompileError>,
}

pub fn correct_file(path: &Path, errors: Vec<CompileError>) -> Result<(), Box<dyn std::error::Error>> {
    if errors.is_empty() {
        unreachable!();
    }

    //eprintln!("Process file: {}", path.display());
    //eprintln!("Error length: {}", errors.len());

    let input = fs::read_to_string(path)?;
    let mut ast = syn::parse_file(&input)?;

    let mut ctx = CorrectContext { errors };

    ctx.visit_file_mut(&mut ast);

    if !ctx.errors.is_empty() {
        eprintln!("Some errors were not corrected:");
        dbg!(&ctx.errors);
        Err("Unrecoverable compile error")?
    }

    let output = prettyplease::unparse(&ast);
    //eprintln!("Write file: {}", path.display());
    let mut output_file = File::create(path)?;
    output_file.write_all(output.as_bytes())?;

    Ok(())
}

impl VisitMut for CorrectContext {
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        if let Stmt::Expr(expr, _) = stmt {
            if is_solana_debugger_serialize_call(expr) {
                //dbg!(&expr);
                let stmt_span = stmt.span();
                let mut err_cov = vec![];
                let mut err_uncov = vec![];
                for err in self.errors.clone() {
                    let cov_source_span = err.source_spans.iter().find(|source_span|
                        stmt_span.byte_range().contains(&source_span.start) &&
                        stmt_span.byte_range().contains(&source_span.end)
                    );
                    if cov_source_span.is_some() {
                        err_cov.push(err);
                    } else {
                        err_uncov.push(err);
                    }
                }

                if !err_cov.is_empty() {
                    /*
                    eprintln!("Remove serialize statement at line {}", stmt_span.start().line);
                    eprintln!("{}", quote!(#stmt));
                    for err in err_cov {
                        eprintln!("{}", err.error_message);
                    }
                     */
                    *stmt = Stmt::Item(Item::Verbatim(TokenStream::new()));
                }
                self.errors = err_uncov
            }
        }
        syn::visit_mut::visit_stmt_mut(self, stmt);
    }
}

fn is_solana_debugger_serialize_call(expr: &Expr) -> bool {
    let serialize_path: ExprPath = syn::parse2::<ExprPath>(
        quote!(crate::_solana_debugger_serialize::_SolanaDebuggerSerialize::_solana_debugger_serialize)
    ).unwrap();

    match expr {
        Expr::Call(call) => {
            match &*call.func {
                Expr::Path(path) if path.eq(&serialize_path) => true,
                _ => false
            }
        }
        _ => false
    }
}