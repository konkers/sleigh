// This is derived from serede's context object.  The Refcell is removed
// in favor of passing mutable references.
//
// Serde code licenced under dual MIT/Apache-2 license.  See:
//   https://github.com/serde-rs/serde

use proc_macro2;
use quote::ToTokens;
use std::fmt::Display;
use std::thread;
use syn;

/// A type to collect errors together and format them.
///
/// Dropping this object will cause a panic. It must be consumed using `check`.
///
/// References can be shared since this type uses run-time exclusive mut checking.
#[derive(Default)]
pub struct Context {
    // The contents will be set to `None` during checking. This is so that checking can be
    // enforced.
    errors: Option<Vec<syn::Error>>,
}

impl Context {
    /// Create a new context object.
    ///
    /// This object contains no errors, but will still trigger a panic if it is not `check`ed.
    pub fn new() -> Self {
        Context {
            errors: Some(Vec::new()),
        }
    }

    /// Add an error to the context object with a tokenenizable object.
    ///
    /// The object is used for spanning in error messages.
    pub fn error_spanned_by<A: ToTokens, T: Display>(&mut self, obj: A, msg: T) {
        self.errors
            .as_mut()
            .unwrap()
            // Curb monomorphization from generating too many identical methods.
            .push(syn::Error::new_spanned(obj.into_token_stream(), msg));
    }

    /// Consume this object, producing a formatted error string if there are errors.
    pub fn check(mut self) -> Result<(), Vec<syn::Error>> {
        let errors = self.errors.take().unwrap();
        match errors.len() {
            0 => Ok(()),
            _ => Err(errors),
        }
    }

    pub fn convert_to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
        let compile_errors = errors.iter().map(syn::Error::to_compile_error);
        quote!(#(#compile_errors)*)
    }
}


impl Drop for Context {
    fn drop(&mut self) {
        if !thread::panicking() && self.errors.is_some() {
            panic!("forgot to check for errors");
        }
    }
}