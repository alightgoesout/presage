use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::ToTokens;
use std::fmt::Display;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Attribute, FnArg, GenericArgument, Ident, LitStr, Pat, PathArguments, ReturnType, Type};

pub fn error(tokens: impl ToTokens, message: impl Display) -> TokenStream {
    syn::Error::new_spanned(tokens, message)
        .to_compile_error()
        .into()
}

pub fn has_name(attribute: &Attribute, name: &str) -> bool {
    let path = attribute.path();
    path.segments.len() == 1 && path.segments[0].ident == name
}

pub fn extract_error_type(return_type: &ReturnType) -> Option<&Type> {
    if let ReturnType::Type(_, return_type) = return_type {
        if let Type::Path(path) = return_type.as_ref() {
            if let Some(last_segment) = path.path.segments.last() {
                if last_segment.ident == "Result" {
                    if let PathArguments::AngleBracketed(bracketed) = &last_segment.arguments {
                        if bracketed.args.len() == 2 {
                            if let GenericArgument::Type(error) = &bracketed.args[1] {
                                return Some(error);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

pub struct HandlerInput<'a> {
    pub context: &'a Pat,
    pub context_type: &'a Type,
    pub parameter: &'a Pat,
    pub parameter_type: &'a Type,
}

pub fn extract_input(inputs: &Punctuated<FnArg, Comma>) -> Option<HandlerInput> {
    if inputs.len() == 2 {
        match (&inputs[0], &inputs[1]) {
            (FnArg::Typed(context), FnArg::Typed(parameter)) => Some(HandlerInput {
                context: context.pat.as_ref(),
                context_type: extract_context_type(context.ty.as_ref())?,
                parameter: parameter.pat.as_ref(),
                parameter_type: parameter.ty.as_ref(),
            }),
            _ => None,
        }
    } else {
        None
    }
}

pub fn extract_context_type(context_type: &Type) -> Option<&Type> {
    match context_type {
        Type::Reference(reference) => Some(reference.elem.as_ref()),
        _ => None,
    }
}

pub fn create_str_literal_from_ident(type_name: &Ident) -> LitStr {
    LitStr::new(
        &type_name.to_string().to_case(Case::Kebab),
        type_name.span(),
    )
}
