use std::collections::HashSet;
use proc_macro2::Ident;
use quote::quote;
use syn::fold::Fold;
use syn::*;
use syn::spanned::Spanned;

#[derive(Clone, Debug)]
struct InstContext {
    // We use a Vec instead of a HashSet to keep the order in which Idents are added. This makes debugging easier
    bindings: Vec<Ident>,
    line: usize
    //file_path: String
}

/// TODO: should support execution path, a set of lines etc.
pub fn inst_ast_general(file: File, line: usize) -> File {
    let mut ctx = InstContext {
        bindings: Vec::new(),
        line
    };
    ctx.fold_file(file)
}

impl Fold for InstContext {
    fn fold_arm(&mut self, node: Arm) -> Arm
    {
        let mut ctx = self.clone();
        // Get the bindings introduced by the arm's pattern
        match &node.pat {
            // Heuristic:
            // If the match arm only consists of a single Pat::Ident, this means it's likely a unit variant and not a new binding
            // So, we ignore it.
            Pat::Ident(_ident) => {},
            _ => {
                ctx.bindings.extend(get_bindings_from_pat(&node.pat));
            }
        }

        syn::fold::fold_arm(&mut ctx, node)
    }

    fn fold_block(&mut self, mut node: Block) -> Block {
        let mut stmts: Vec<Stmt> = vec![];
        for stmt in node.stmts {
            let line_number = stmt.span().start().line;

            // Instrumentation statements that come before stmt (but only if we're at the right line)
            if line_number == self.line {
                let mut inst_stmts: Vec<Stmt> = vec![];
                let line_start_str = format!("-.!;LINE_START;{}", line_number);
                inst_stmts.push(parse_quote! {
                    solana_program::log::sol_log(#line_start_str);
                });
                for ident in &self.bindings {
                    let ident_str = ident.to_string();
                    let print_var: Stmt = parse_quote! {
                        crate::_solana_debugger_serialize::_SolanaDebuggerSerialize::_solana_debugger_serialize(&#ident, #ident_str);
                    };
                    inst_stmts.push(print_var);
                }
                inst_stmts.push(parse_quote! {
                    solana_program::log::sol_log("-.!;LINE_END");
                });
                let inst_block = Block {
                    brace_token: syn::token::Brace::default(),
                    stmts: inst_stmts
                };
                stmts.push(parse2::<Stmt>(quote!(#inst_block)).unwrap());
            }

            // Get new local bindings introduced by this statement
            let in_scope_bindings = get_in_scope_bindings_from_stmt(&stmt);

            // self.clone() so that fold_stmt doesn't add its local bindings to ours
            stmts.push(self.clone().fold_stmt(stmt));

            // Add the new bindings to print them before the next statements
            self.bindings.extend(in_scope_bindings);
        };
        node.stmts = stmts;
        node
    }

    fn fold_expr_if(&mut self, mut node: ExprIf) -> ExprIf {
        let mut then_ctx = self.clone();
        let mut else_ctx = self.clone();

        if let Expr::Let(ref expr) = *node.cond {
            let let_bindings = get_bindings_from_pat(&*expr.pat);
            //dbg!(&let_bindings);
            then_ctx.bindings.extend(let_bindings);
        }

        let then_branch = node.then_branch.clone();
        node.then_branch = parse_quote!({});
        node = syn::fold::fold_expr_if(&mut else_ctx, node);
        node.then_branch = then_ctx.fold_block(then_branch);
        node
    }

    fn fold_impl_item_fn(&mut self, node: ImplItemFn) -> ImplItemFn
    {
        self.bindings = get_bindings_from_fn_sig(&node.sig);
        syn::fold::fold_impl_item_fn(self, node)

        /*
        let inst_block = inst_fn_block(ctx.fold_block(node.block), &node.sig.output, &node.sig.ident, &self.file_path);
        ImplItemFn {
            attrs: self.fold_attributes(node.attrs),
            vis: self.fold_visibility(node.vis),
            defaultness: node.defaultness,
            sig: self.fold_signature(node.sig),
            block: inst_block,
        }
         */
        // Idea: first do fold_block, THEN add header and footer inst for the fn
    }

    /*
    We can skip this since the matchee doesn't introduce new bindings
    We can copy the context in the arms instead

    fn fold_expr_match(&mut self, mut node: ExprMatch) -> ExprMatch
    {
        node = syn::fold::fold_expr_match(self, node);
        node.arms = node.arms.into_iter().map(|arm|
            // Use one copy of self for each arm
            self.clone().fold_arm(arm)
        ).collect();
        node
    }
     */

    fn fold_item_fn(&mut self, node: ItemFn) -> ItemFn
    {
        self.bindings = get_bindings_from_fn_sig(&node.sig);
        syn::fold::fold_item_fn(self, node)

        /*
        let inst_block = inst_fn_block(ctx.fold_block(*node.block), &node.sig.output, &node.sig.ident, &self.file_path);
        ItemFn {
            attrs: self.fold_attributes(node.attrs),
            vis: self.fold_visibility(node.vis),
            sig: self.fold_signature(node.sig),
            block: Box::new(inst_block),
        }
         */
    }
}

fn get_bindings_from_fn_sig(sig: &Signature) -> Vec<Ident> {
    let mut bindings = Vec::new();
    for arg in sig.inputs.iter() {
        match arg {
            FnArg::Receiver(receiver) => {
                bindings.push(receiver.self_token.into());
            },
            FnArg::Typed(PatType { pat, .. }) => {
                bindings.extend(get_bindings_from_pat(pat));
            }
        }
    }
    bindings
}

fn get_bindings_from_pat(p: &Pat) -> Vec<Ident> {
    let mut bindings = Vec::new();
    match p {
        Pat::Ident(PatIdent { ident, .. })  => {
            bindings.push(ident.clone());
        },
        Pat::TupleStruct(PatTupleStruct { elems, .. })  => {
            for el in elems {
                bindings.extend(get_bindings_from_pat(el));
            }
        },
        Pat::Type(PatType { pat, .. }) => {
            bindings.extend(get_bindings_from_pat(&*pat));
        },
        Pat::Struct(PatStruct { fields, .. }) => {
            for field in fields {
                bindings.extend(get_bindings_from_pat(&field.pat));
            }
        },
        Pat::Tuple(PatTuple { elems, .. }) => {
            for el in elems {
                bindings.extend(get_bindings_from_pat(el));
            }
        },
        // TODO: other cases
        _ => {}
    }
    bindings

    /*
    match p {
        Pat::Ident(PatIdent { ident, subpat, .. }) => {
            bindings.push(ident.clone());
            // Handle optional sub-pattern (occurs in `binding @ SUBPATTERN`)
            // Untested
            if let Some((_, subpat)) = subpat {
                bindings.extend(get_bindings_from_pat(&*subpat));
            }
        },
        Pat::TupleStruct(PatTupleStruct { elems, .. }) => {
            for el in elems {
                bindings.extend(get_bindings_from_pat(el));
            }
        },
        Pat::Type(PatType { pat, .. }) => {
            bindings.extend(get_bindings_from_pat(&*pat));
        },
        Pat::Const(_) => {
            // Const patterns don't bind variables
        },
        Pat::Lit(_) => {
            // Literal patterns don't bind variables
        },
        Pat::Macro(_) => {
            // Can't inspect inside a macro pattern
        },
        Pat::Or(PatOr { cases, .. }) => {
            // For alternations like `a | b`, collect bindings from all cases
            for case in cases {
                bindings.extend(get_bindings_from_pat(case));
            }
        },
        Pat::Paren(PatParen { pat, .. }) => {
            bindings.extend(get_bindings_from_pat(&*pat));
        },
        Pat::Path(_) => {
            // Path patterns (like enum variants) don't bind variables
        },
        Pat::Range(_) => {
            // Range patterns don't bind variables
        },
        Pat::Reference(PatReference { pat, .. }) => {
            bindings.extend(get_bindings_from_pat(&*pat));
        },
        Pat::Rest(_) => {
            // Rest patterns (..) don't bind variables
        },
        Pat::Slice(PatSlice { elems, .. }) => {
            for el in elems {
                bindings.extend(get_bindings_from_pat(el));
            }
        },
        Pat::Struct(PatStruct { fields, .. }) => {
            for field in fields {
                bindings.extend(get_bindings_from_pat(&field.pat));
            }
        },
        Pat::Tuple(PatTuple { elems, .. }) => {
            for el in elems {
                bindings.extend(get_bindings_from_pat(el));
            }
        },
        Pat::Verbatim(_) => {
            // Can't inspect inside verbatim tokens
        },
        Pat::Wild(_) => {
            // Wildcard patterns (_) don't bind variables
        },
    }
     */
}

// TODO: include this in the inst
#[allow(dead_code)]
fn inst_fn_block(block: Block, return_type: &ReturnType, fn_name: &Ident, file_path: &str) -> Block {

    let var_type = match return_type {
        ReturnType::Default => parse_quote! { () },
        ReturnType::Type(_, ty) => (**ty).clone(),
    };

    let fn_name_str = fn_name.to_string();

    parse_quote! {{
        sol_log("-.!;FN_START");
        sol_log(#fn_name_str);
        sol_log(#file_path);
        let ret: #var_type = #block;
        sol_log("-.!;FN_END");
        ret
    }}

    /*
    // Doesn't work since it changes the function to -> ()
    block.stmts.insert(0, parse_quote!(
       sol_log("-.!;FN_START");
    ));

    block.stmts.push(parse_quote!(
       sol_log("-.!;FN_END");
    ));

    block
     */
}


/*
TODO
handle: if let Some(x) = y
Expr::If
*/

/// Get new bindings introduced by stmt that are valid in its parent scope
fn get_in_scope_bindings_from_stmt(stmt: &Stmt) -> HashSet<Ident> {
    let mut bindings = HashSet::new();
    match stmt {
        Stmt::Local(local) => {
            bindings.extend(get_bindings_from_pat(&local.pat));
        }
        _ => {}
    }
    bindings
}