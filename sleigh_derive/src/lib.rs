extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::Meta::{List, Word};
use syn::NestedMeta::{Literal, Meta};

#[proc_macro_derive(Sleigh, attributes(sleigh))]
pub fn sleigh_derive(input: TokenStream) -> TokenStream {
    // Construct a represntation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_sleigh_macro(&ast)
}

fn get_sleigh_meta_items(attr: &syn::Attribute) -> Option<Vec<syn::NestedMeta>> {
    if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "sleigh" {
        match attr.interpret_meta() {
            Some(List(ref meta)) => Some(meta.nested.iter().cloned().collect()),
            _ => {
                // TODO: produce an error
                None
            }
        }
    } else {
        None
    }
}

fn find_key(ast: &syn::DeriveInput) -> Option<syn::Ident> {
    let data = match &ast.data {
        syn::Data::Struct(ds) => ds,
        _ => return None,
    };

    let mut key: Option<syn::Ident> = None;
    for f in data.fields.iter() {
        //eprintln!("{:?}", f);
        for a in f.attrs.iter() {
            if a.style == syn::AttrStyle::Outer
                && a.path.segments.len() == 1
                && a.path.segments[0].ident == "sleigh"
            {
                //eprintln!("{:?}", a.path.segments[0].ident);
            }
        }
        for meta_items in f.attrs.iter().filter_map(get_sleigh_meta_items) {
            for meta_item in meta_items {
                match meta_item {
                    Meta(Word(ref word)) if word == "key" => {
                        if let Some(ref id) = f.ident {
                            key = Some(id.clone());
                        }
                    }
                    Meta(ref meta_item) => {}

                    Literal(ref lit) => {}
                }
            }
        }
    }

    key
}

fn impl_sleigh_macro(ast: &syn::DeriveInput) -> TokenStream {
    let key = match find_key(ast) {
        Some(key) => key,
        None => return quote! {compile_error!("")}.into(),
    };

    let name = &ast.ident;
    let gen = quote! {
        impl Sleigh for #name {
            fn bucket() -> &'static [u8] {
                stringify!(#name).as_bytes()
            }

            fn key_name() -> &'static str {
                stringify!(#key)
            }

            fn key_value(&self) -> Vec<u8> {
                sleigh::encode_key(&self.#key)
            }
        }
    };
    gen.into()
}

#[cfg(test)]
mod tests {}
