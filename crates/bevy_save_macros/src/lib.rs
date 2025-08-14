use bevy_macro_utils::derive_label;
use proc_macro::TokenStream;
use quote::format_ident;
use syn::{
    DeriveInput,
    parse_macro_input,
};

fn bevy_save_path() -> syn::Path {
    format_ident!("bevy_save").into()
}

fn bevy_path() -> syn::Path {
    format_ident!("bevy").into()
}

/// Derive macro generating an impl of the trait `FlowLabel`.
///
/// This does not work for unions.
#[proc_macro_derive(FlowLabel)]
pub fn derive_flow_label(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut label = bevy_save_path();
    label.segments.push(format_ident!("flows").into());
    label.segments.push(format_ident!("FlowLabel").into());

    let mut dyn_eq = bevy_path();
    dyn_eq.segments.push(format_ident!("app").into());
    dyn_eq.segments.push(format_ident!("DynEq").into());

    derive_label(input, "FlowLabel", &label, &dyn_eq)
}
