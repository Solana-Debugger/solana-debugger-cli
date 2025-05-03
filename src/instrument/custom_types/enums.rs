use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, ItemEnum, ItemImpl, Variant};

pub fn get_serialize_impl(node: &ItemEnum) -> ItemImpl {
    let name = &node.ident;

    let variant_str_arms = node.variants.iter().map(get_variant_str_arm);

    let variant_content_arms = node.variants.iter().map(get_variant_content_arm);

    parse_quote! {
        impl crate::_solana_debugger_serialize::_SolanaDebuggerSerialize for #name {
            fn _solana_debugger_serialize(&self, name: &str) {
                solana_program::log::sol_log("START_NODE");
                solana_program::log::sol_log("complex");
                solana_program::log::sol_log(name);
                solana_program::log::sol_log(std::any::type_name_of_val(self));
                solana_program::log::sol_log("str_ident");

                let variant_str = match self {
                    #(#variant_str_arms),*
                };
                solana_program::log::sol_log(variant_str);

                match self {
                    #(#variant_content_arms)*
                }

                solana_program::log::sol_log("END_NODE");
            }
        }
    }
}

/// Example
///
/// ```
/// let variant_str = match self {
///   Ok(_) => "Ok",
///   Err(_) => "Err"
/// };
/// ```
fn get_variant_str_arm(variant: &Variant) -> TokenStream {
    let variant_name = &variant.ident;
    let variant_name_str = &variant.ident.to_string();

    match &variant.fields {
        syn::Fields::Named(_) => {
            quote! {
                Self::#variant_name { .. } => #variant_name_str
            }
        }
        syn::Fields::Unnamed(_) => {
            quote! {
                Self::#variant_name(..) => #variant_name_str
            }
        }
        syn::Fields::Unit => {
            quote! {
                Self::#variant_name => #variant_name_str
            }
        }
    }
}

/// Example
///
/// ```
/// match self {
///     Ok(v) => {
///         crate::_solana_debugger_serialize::_SolanaDebuggerSerialize::_solana_debugger_serialize(&v, "0");
///     },
///     Err(v) => {
///         crate::_solana_debugger_serialize::_SolanaDebuggerSerialize::_solana_debugger_serialize(&v, "0");
///     }
/// }
/// ```

fn get_variant_content_arm(variant: &Variant) -> TokenStream {
    let variant_name = &variant.ident;

    match &variant.fields {
        syn::Fields::Named(fields) => {

            let field_names = fields.named.iter().map(|field| &field.ident);

            let field_stmts = fields.named.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = field_name.to_string();
                quote! {
                    crate::_solana_debugger_serialize::_SolanaDebuggerSerialize::_solana_debugger_serialize(&#field_name, #field_name_str);
                }
            });

            quote! {
                Self::#variant_name { #(#field_names),* } => {
                    #(#field_stmts)*
                }
            }
        }
        syn::Fields::Unnamed(fields) => {

            let field_names = (0..fields.unnamed.len()).map(|i| {
                syn::Ident::new(&format!("f{}", i), variant.ident.span())
            }).collect::<Vec<_>>();

            let field_stmts = field_names.iter().enumerate().map(|(i, var)| {
                let index_str = format!("{}", i);
                quote! {
                    crate::_solana_debugger_serialize::_SolanaDebuggerSerialize::_solana_debugger_serialize(&*#var, #index_str);
                }
            });

            quote! {
                Self::#variant_name(#(#field_names),*) => {
                    #(#field_stmts)*
                }
            }
        }
        syn::Fields::Unit => {
            quote! {
                Self::#variant_name => {}
            }
        }
    }
}