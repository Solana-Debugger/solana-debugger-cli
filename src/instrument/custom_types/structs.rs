use quote::quote;
use syn::{parse_quote, Item, ItemImpl, ItemStruct};

pub fn get_serialize_impl(node: &ItemStruct) -> ItemImpl {

    let name = &node.ident;
    let fields = &node.fields;

    // Generate the field serialization code based on the struct fields
    let serialize_fields = match fields {
        syn::Fields::Named(fields_named) => {
            let field_statements = fields_named.named.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = field_name.to_string();
                quote! {
                    crate::_solana_debugger_serialize::_SolanaDebuggerSerialize::_solana_debugger_serialize(&self.#field_name, #field_name_str);
                }
            });
            quote! {
                #(#field_statements)*
            }
        },
        syn::Fields::Unnamed(fields_unnamed) => {
            let field_statements = fields_unnamed.unnamed.iter().enumerate().map(|(i, _)| {
                let index = syn::Index::from(i);
                let index_str = format!("{}", i);
                quote! {
                    crate::_solana_debugger_serialize::_SolanaDebuggerSerialize::_solana_debugger_serialize(&self.#index, #index_str);
                }
            });
            quote! {
                #(#field_statements)*
            }
        },
        syn::Fields::Unit => {
            quote! {}
        }
    };

    parse_quote! {
        impl crate::_solana_debugger_serialize::_SolanaDebuggerSerialize for #name {
            fn _solana_debugger_serialize(&self, name: &str) {
                solana_program::log::sol_log("START_NODE");
                solana_program::log::sol_log("complex");
                solana_program::log::sol_log(name);
                solana_program::log::sol_log(std::any::type_name_of_val(self));
                solana_program::log::sol_log("no_data");

                #serialize_fields

                solana_program::log::sol_log("END_NODE");
            }
        }
    }
}