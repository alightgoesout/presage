use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Generics, Ident, ItemFn, Signature, Token, Type};

use crate::utils::{error, extract_error_type, extract_input, HandlerInput};

pub fn command_handler(arguments: TokenStream, handler: TokenStream) -> TokenStream {
    let arguments = parse_macro_input!(arguments as CommandHandlerArguments);
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
        return error(handler_name, "A command handler must be async");
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
                r#"arguments of a command handler should match "(context: &mut C, command: _)""#,
            )
        }
    };

    let error_type = match arguments
        .error
        .as_ref()
        .or_else(|| extract_error_type(&output))
    {
        Some(error_type) => error_type,
        None => return error(handler_name, MISSING_ERROR_TYPE),
    };

    TokenStream::from(quote! {
        #(#attrs)*
        #[allow(non_camel_case_types)]
        #vis struct #handler_name;

        #[presage::async_trait]
        impl<#params> presage::CommandHandler<#context_type, #error_type> for #handler_name #where_clause {
            fn command_name(&self) -> &'static str {
                <#parameter_type as presage::Command>::NAME
            }

            async fn handle(&self, #context: &mut #context_type, command: presage::BoxedCommand) #output {
                let #parameter: #parameter_type = command.downcast()?;
                #block
            }
        }
    })
}

#[derive(Default)]
struct CommandHandlerArguments {
    error: Option<Type>,
}

impl Parse for CommandHandlerArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            Ok(Default::default())
        } else {
            let argument = input.parse::<Ident>()?;
            if argument != "error" {
                Err(syn::Error::new_spanned(argument, "unexpected argument"))
            } else {
                input.parse::<Token![=]>()?;
                Ok(CommandHandlerArguments {
                    error: Some(input.parse()?),
                })
            }
        }
    }
}

const MISSING_ERROR_TYPE: &str = r"Cannot find the error type.

Help: specify the error type with `#[command_handler(error = <path>)] or by detailing the result type in the function signature (`Result<Events, Error>`)";
