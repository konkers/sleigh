extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

use syn::Meta::{List, Word};
use syn::NestedMeta::{Literal, Meta};
use syn::{Field, NestedMeta, Type};
mod context;
use context::Context;

#[proc_macro_derive(Sleigh, attributes(sleigh))]
pub fn sleigh_derive(input: TokenStream) -> TokenStream {
    // Construct a represntation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_sleigh_macro(&ast)
}

#[derive(Debug, Default)]
struct Attrs {
    key: Option<syn::Ident>,
    key_type: Option<syn::Type>,
    auto: bool,
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

fn handle_meta_item(context: &mut Context, attrs: &mut Attrs, f: &Field, meta_item: &NestedMeta) {
    match meta_item {
        Meta(Word(ref word)) if word == "key" => {
            if let Some(ref id) = f.ident {
                attrs.key = Some(id.clone());
            }
        }

        Meta(Word(ref word)) if word == "auto" => {
            attrs.auto = true;
            let is_u64 = match &f.ty {
                Type::Path(p) => p.path.is_ident("u64"),
                _ => false,
            };

            if !is_u64 {
                context
                    .error_spanned_by(f.ty.clone(), "auto keys are only supported for u64 fields");
            }
        }

        Meta(ref meta_item) => {
            context.error_spanned_by(
                meta_item.name(),
                format!("unknown sleigh field attribute `{}`", meta_item.name()),
            );
        }

        Literal(ref lit) => {
            context.error_spanned_by(lit, "unexpected literal in sleigh field attribute");
        }
    }
}

fn parse_attrs(context: &mut Context, ast: &syn::DeriveInput) -> Attrs {
    let mut attrs: Attrs = Default::default();

    let data = match &ast.data {
        syn::Data::Struct(ds) => ds,
        _ => {
            context.error_spanned_by(ast, "sleigh derive only supported on structs.");
            return attrs;
        }
    };

    for f in data.fields.iter() {
        for meta_items in f.attrs.iter().filter_map(get_sleigh_meta_items) {
            for meta_item in meta_items {
                handle_meta_item(context, &mut attrs, f, &meta_item);
            }
        }
    }

    attrs
}

fn impl_sleigh_macro(ast: &syn::DeriveInput) -> TokenStream {

    let mut context = Context::new();
    let attrs = parse_attrs(&mut context, ast);

    // Extract attrs we need for code gen into local vars.
    let key = match attrs.key {
        Some(key) => key,
        None => return quote! {compile_error!("")}.into(),
    };
    let name = &ast.ident;

    // Only generate auto key code if needed.
    let prep = if attrs.auto {
        quote! {
            fn prep(&mut self, db: &sleigh::Db) -> Result<(), sleigh::Error>{
                if self.#key == u64::default() {
                    self.#key = db.get_unique_key()?;
                }
                Ok(())
            }
        }
    } else {
        quote! {
            fn prep(&mut self, db: &sleigh::Db) -> Result<(), sleigh::Error>{
                Ok(())
            }
        }
    };

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
            #prep
        }
    };

    match context.check() {
        Ok(_) => gen.into(),
        Err(e) => Context::convert_to_compile_errors(e).into(),
    }

}

#[cfg(test)]
mod tests {}
