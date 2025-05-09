use syn::{parse_quote, File, Item};
use crate::instrument::{inst_ast_general, inst_ast_custom_types};

#[derive(Debug)]
pub struct InstAstSpec {
    pub mod_fixed_serialization: bool,
    pub feature_min_specialization: bool,
    #[allow(dead_code)]
    pub debugee_file_path: String,
    pub line_inst: Option<usize>,
    pub custom_type_serialization: bool
}

pub fn inst_ast(mut input: File, spec: &InstAstSpec) -> File {

    if let Some(line) = &spec.line_inst {
        input = inst_ast_general(input, *line);
    }

    if spec.custom_type_serialization {
        input = inst_ast_custom_types(input);
    }

    if spec.mod_fixed_serialization {
        input.items.insert(0, Item::Mod(parse_quote! {
            mod _solana_debugger_serialize;
        }));
    }

    if spec.feature_min_specialization {
        input.attrs.insert(0, parse_quote! {
            #![feature(min_specialization)]
        });
    }

    input
}
/*
if config.main_module {

}

 */
