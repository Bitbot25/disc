#![feature(proc_macro_diagnostic)]

use proc_macro::{Diagnostic, Level, TokenStream};
use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, DataEnum, Generics, spanned::Spanned};

#[proc_macro_attribute]
pub fn disc(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        Diagnostic::new(Level::Error, "No arguments are expected. This will be changed in the future so that other types such as u32 can be used.").emit();
        return TokenStream::new();
    }

    let input = parse_macro_input!(input as DeriveInput);
    
    match input.data {
        Data::Enum(ref data) => {
            let implementation = generate_implementation(data, &input.ident, &input.generics);
            return TokenStream::from(quote! {
                #[repr(u8)]
                #input

                #implementation
            });
        },
        Data::Struct(..) => Diagnostic::new(Level::Error, "Incompatible data type.")
            .span_error(input.ident.span().unwrap(), "`struct` has no discriminant.")
            .help("Use `enum` instead.")
            .emit(),
        Data::Union(..) => Diagnostic::new(Level::Error, "Incompatible data type.")
            .span_error(input.ident.span().unwrap(), "`union` has no discriminant.")
            .help("Use `enum` instead.")
            .emit(),
    }
    TokenStream::new() 
}

fn verify_fields(data: &DataEnum) -> bool {
    for variant in data.variants.iter() {
        if !variant.fields.is_empty() {
            Diagnostic::new(Level::Error, "A disc enumeration cannot have fields.").span_error(variant.fields.span().unwrap(), "Here").emit();
            return false;
        }
    }
    true
}

fn generate_implementation(data: &DataEnum, name: &Ident, generics: &Generics) -> TokenStream2 {
    if !verify_fields(data) {
        return TokenStream2::new();
    }

    let found_crate = crate_name("disc").expect("Couldn't find the crate `disc`.");

    let from_discriminant_ty = match found_crate {
        FoundCrate::Itself => quote!(crate::FromDiscriminant),
        FoundCrate::Name(name) => {
            let ident = format_ident!("{}", name);
            quote!(#ident::FromDiscriminant)
        }
    };

    // TODO: Add a implementation for all discriminants when `auto` is false.
    let auto = !data.variants.iter().any(|variant| variant.discriminant.is_some());
    let n = data.variants.len();

    if n > (u8::MAX as usize) {
        Diagnostic::new(Level::Error, "Cannot have more than `u8::MAX` (255) variants.").emit()
    }
    let n = n as u8;
    
    let body = if auto {
        quote! {
            if d >= #n {
                return None;
            }
            Some(unsafe { ::core::mem::transmute(d) })
        }
    } else {
        todo!()
    };
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let tokens = quote! {
        impl #impl_generics #from_discriminant_ty<u8> for #name #ty_generics #where_clause {
            fn from_discriminant(d: u8) -> Option<Self> {
                #body
            }
        }
    };
    tokens
}