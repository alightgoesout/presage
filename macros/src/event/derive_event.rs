use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Attribute, Ident, Item, ItemEnum, ItemStruct, LitStr, Token};

use crate::utils::{create_str_literal_from_ident, error, has_name};

pub fn derive_event(event: TokenStream) -> TokenStream {
    let item = parse_macro_input!(event as Item);

    let EventInfo {
        type_name,
        event_name,
    } = match item.try_into() {
        Ok(info) => info,
        Err(error) => return error,
    };

    TokenStream::from(quote! {
        impl presage::Event for #type_name {
            const NAME: &'static str = #event_name;
        }
    })
}

#[derive(Default)]
struct DeriveEventArguments {
    event_name: Option<LitStr>,
}

impl Parse for DeriveEventArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            Ok(DeriveEventArguments::default())
        } else {
            let argument = input.parse::<Ident>()?;
            if argument != "name" {
                Err(syn::Error::new_spanned(argument, "unexpected argument"))
            } else {
                input.parse::<Token![=]>()?;
                Ok(DeriveEventArguments {
                    event_name: Some(input.parse()?),
                })
            }
        }
    }
}

impl TryFrom<&[Attribute]> for DeriveEventArguments {
    type Error = syn::Error;

    fn try_from(attributes: &[Attribute]) -> Result<Self, Self::Error> {
        for attribute in attributes {
            if has_name(attribute, "presage") {
                return attribute.parse_args();
            }
        }
        Ok(DeriveEventArguments::default())
    }
}

struct EventInfo {
    type_name: Ident,
    event_name: LitStr,
}

impl TryFrom<Item> for EventInfo {
    type Error = TokenStream;

    fn try_from(item: Item) -> Result<Self, Self::Error> {
        match item {
            Item::Struct(item) => item.try_into(),
            Item::Enum(item) => item.try_into(),
            _ => Err(error(
                item,
                "Event can only be derived for a struct or an enum",
            )),
        }
    }
}

impl TryFrom<ItemStruct> for EventInfo {
    type Error = TokenStream;

    fn try_from(item: ItemStruct) -> Result<Self, Self::Error> {
        let arguments = DeriveEventArguments::try_from(item.attrs.as_slice())
            .map_err(syn::Error::into_compile_error)?;

        let event_name = arguments
            .event_name
            .unwrap_or_else(|| create_str_literal_from_ident(&item.ident));

        Ok(EventInfo {
            type_name: item.ident,
            event_name,
        })
    }
}

impl TryFrom<ItemEnum> for EventInfo {
    type Error = TokenStream;

    fn try_from(item: ItemEnum) -> Result<Self, Self::Error> {
        let arguments = DeriveEventArguments::try_from(item.attrs.as_slice())
            .map_err(syn::Error::into_compile_error)?;

        let event_name = arguments
            .event_name
            .unwrap_or_else(|| create_str_literal_from_ident(&item.ident));

        Ok(EventInfo {
            type_name: item.ident,
            event_name,
        })
    }
}
