use syn::{File, Item, ItemMod};
use syn::fold::Fold;

#[derive(Clone, Debug)]
struct InstContext {
}

pub fn inst_ast_custom_types(file: File) -> File {
    let mut ctx = InstContext {};
    ctx.fold_file(file)
}

impl Fold for InstContext {
    fn fold_file(&mut self, mut node: File) -> File {
        insert_serialize_impl(&mut node.items);
        syn::fold::fold_file(self, node)
    }

    fn fold_item_mod(&mut self, mut node: ItemMod) -> ItemMod {
        match &mut node.content {
            Some((_, items)) => {
                insert_serialize_impl(items)
            }
            None => {},
        }
        syn::fold::fold_item_mod(self, node)
    }
}

fn insert_serialize_impl(items: &mut Vec<Item>) {
    let mut i: usize = 0;
    while i < items.len() {
        match &items[i] {
            Item::Struct(val) => {
                items.insert(i+1, syn::Item::Impl(crate::instrument::custom_types::structs::get_serialize_impl(val)));
                i += 2
            },
            Item::Enum(val) => {
                items.insert(i+1, syn::Item::Impl(crate::instrument::custom_types::enums::get_serialize_impl(val)));
                i += 2
            },
            _ => {
                i += 1
            }
        }
    }
}