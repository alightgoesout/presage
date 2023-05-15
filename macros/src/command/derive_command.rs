use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Attribute, Ident, Item, LitStr, Token};

use crate::utils::{create_str_literal_from_ident, error, has_name};

pub fn derive_command(command: TokenStream) -> TokenStream {
    let item = parse_macro_input!(command as Item);

    let CommandInfo {
        type_name,
        command_name,
    } = match item.try_into() {
        Ok(info) => info,
        Err(error) => return error,
    };

    TokenStream::from(quote! {
        impl presage::Command for #type_name {
            const NAME: &'static str = #command_name;
        }
    })
}

struct CommandInfo {
    type_name: Ident,
    command_name: LitStr,
}

impl CommandInfo {
    fn try_from(type_name: Ident, attributes: &[Attribute]) -> Result<Self, TokenStream> {
        let arguments =
            DeriveCommandArguments::try_from(attributes).map_err(syn::Error::into_compile_error)?;
        let command_name = arguments
            .command_name
            .unwrap_or_else(|| create_str_literal_from_ident(&type_name));
        Ok(CommandInfo {
            type_name,
            command_name,
        })
    }
}

impl TryFrom<Item> for CommandInfo {
    type Error = TokenStream;

    fn try_from(item: Item) -> Result<Self, Self::Error> {
        match item {
            Item::Struct(item) => CommandInfo::try_from(item.ident, &item.attrs),
            Item::Enum(item) => CommandInfo::try_from(item.ident, &item.attrs),
            _ => Err(error(
                item,
                "Command can only be derived for a struct or an enum",
            )),
        }
    }
}

#[derive(Default)]
struct DeriveCommandArguments {
    command_name: Option<LitStr>,
}

impl Parse for DeriveCommandArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            Ok(DeriveCommandArguments::default())
        } else {
            let argument = input.parse::<Ident>()?;
            if argument != "name" {
                Err(syn::Error::new_spanned(argument, "unexpected argument"))
            } else {
                input.parse::<Token![=]>()?;
                Ok(DeriveCommandArguments {
                    command_name: Some(input.parse()?),
                })
            }
        }
    }
}

impl TryFrom<&[Attribute]> for DeriveCommandArguments {
    type Error = syn::Error;

    fn try_from(attributes: &[Attribute]) -> Result<Self, Self::Error> {
        for attribute in attributes {
            if has_name(attribute, "presage") {
                return attribute.parse_args();
            }
        }
        Ok(DeriveCommandArguments::default())
    }
}
