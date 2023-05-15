use proc_macro::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseStream};
use syn::token::Bracket;
use syn::{bracketed, parse_macro_input, Generics, Ident, ItemFn, LitStr, Signature, Token, Type};

use crate::utils::{error, extract_error_type, extract_input, HandlerInput};

pub fn event_handler(arguments: TokenStream, handler: TokenStream) -> TokenStream {
    let arguments = parse_macro_input!(arguments as EventHandlerArguments);
    let ItemFn {
        vis,
        sig,
        block,
        attrs,
        ..
    } = parse_macro_input!(handler as ItemFn);

    let Signature {
        asyncness,
        ident: handler_name,
        inputs,
        output,
        generics: Generics {
            params,
            where_clause,
            ..
        },
        ..
    } = sig;

    if asyncness.is_none() {
        return error(handler_name, "an event handler must be async");
    }

    let HandlerInput {
        context,
        context_type,
        parameter,
        parameter_type,
    } = match extract_input(&inputs) {
        Some(result) => result,
        None => {
            return error(
                inputs,
                r#"arguments of an event handler should match "(context: &mut C, event: _)""#,
            )
        }
    };

    let event_type = if arguments.event_names.is_some() {
        quote! { #parameter_type }
    } else {
        quote! { &presage::SerializedEvent }
    };

    let event_conversion = if arguments.event_names.is_some() {
        quote! { let #parameter = event; }
    } else {
        quote! { let #parameter: #parameter_type = event.clone().deserialize()?; }
    };

    let error_type = match arguments
        .error
        .as_ref()
        .or_else(|| extract_error_type(&output))
    {
        Some(error_type) => error_type,
        None => return error(handler_name, MISSING_ERROR_TYPE),
    };

    let event_names = arguments
        .event_names
        .unwrap_or_else(|| vec![EventName::Event(parameter_type.clone())]);

    TokenStream::from(quote! {
        #(#attrs)*
        #[allow(non_camel_case_types)]
        #vis struct #handler_name;

        #[presage::async_trait]
        impl<#params> presage::EventHandler<#context_type, #error_type> for #handler_name #where_clause {
            fn event_names(&self) -> &[&'static str] {
                &[#(#event_names),*]
            }

            async fn handle(&self, #context: &mut #context_type, event: #event_type) #output {
                #event_conversion
                #block
            }
        }
    })
}

struct EventHandlerArguments {
    error: Option<Type>,
    event_names: Option<Vec<EventName>>,
}

impl Parse for EventHandlerArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut error = None;
        let mut event_names = None;

        while !input.is_empty() {
            let ident = input.parse::<Ident>()?;
            match ident.to_string().as_str() {
                "error" => {
                    input.parse::<Token![=]>()?;
                    error = Some(input.parse()?);
                }
                "events" => {
                    input.parse::<Token![=]>()?;
                    event_names = Some(parse_event_names(input)?)
                }
                _ => return Err(syn::Error::new_spanned(ident, "unknown argument")),
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(EventHandlerArguments { error, event_names })
    }
}

fn parse_event_names(input: ParseStream) -> syn::Result<Vec<EventName>> {
    if input.peek(Bracket) {
        let content;
        bracketed!(content in input);
        Ok(content
            .parse_terminated(EventName::parse, Token![,])?
            .into_iter()
            .collect())
    } else {
        Ok(vec![input.parse::<EventName>()?])
    }
}

enum EventName {
    Literal(LitStr),
    Event(Type),
}

impl Parse for EventName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ahead = input.fork();
        if let Ok(literal) = ahead.parse::<LitStr>() {
            input.advance_to(&ahead);
            if literal.value().trim().is_empty() {
                Err(syn::Error::new_spanned(
                    literal,
                    "an event name must not be empty",
                ))
            } else {
                Ok(EventName::Literal(literal))
            }
        } else if let Ok(event_type) = input.parse::<Type>() {
            Ok(EventName::Event(event_type))
        } else {
            Err(input.error("string literal or event type expected"))
        }
    }
}

impl ToTokens for EventName {
    fn to_tokens(&self, tokens: &mut quote::__private::TokenStream) {
        match self {
            Self::Literal(literal) => tokens.append_all(quote! {#literal}),
            Self::Event(event_type) => {
                tokens.append_all(quote! {<#event_type as presage::Event>::NAME})
            }
        }
    }
}

const MISSING_ERROR_TYPE: &str = r"Cannot find the error type.

Help: specify the error type with `#[event_handler(error = <path>)] or by detailing the result type in the function signature (`Result<Commands, Error>`)";
