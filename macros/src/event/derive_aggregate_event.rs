use proc_macro::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseStream};
use syn::{
    parse_macro_input, Attribute, Fields, FieldsNamed, FieldsUnnamed, Ident, Item, ItemEnum,
    ItemStruct, LitStr, Path, Token, Variant,
};

use crate::utils::{create_str_literal_from_ident, error, has_name};

pub fn derive_aggregate_event(event: TokenStream) -> TokenStream {
    let item = parse_macro_input!(event as Item);

    let AggregateEventInfo {
        type_name,
        aggregate,
        id_spec,
        event_name,
    } = match item.try_into() {
        Ok(info) => info,
        Err(error) => return error,
    };

    TokenStream::from(quote! {
        impl presage::Event for #type_name {
            const NAME: &'static str = #event_name;
        }

        impl presage::AggregateEvent for #type_name {
            type Aggregate = #aggregate;

            fn id(&self) -> Id<Self::Aggregate> {
                #id_spec
            }
        }
    })
}

#[derive(Default)]
struct DeriveAggregateEventArguments {
    aggregate: Option<Path>,
    id: Option<Ident>,
    event_name: Option<LitStr>,
}

impl Parse for DeriveAggregateEventArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ahead = input.fork();
        if let Ok(path) = ahead.parse::<Path>() {
            if ahead.is_empty() {
                input.advance_to(&ahead);
                return Ok(DeriveAggregateEventArguments {
                    aggregate: Some(path),
                    ..Default::default()
                });
            }
        }

        let mut aggregate = None;
        let mut id = None;
        let mut event_name = None;

        while !input.is_empty() {
            let ident = input.parse::<Ident>()?;
            match ident.to_string().as_str() {
                "aggregate" => {
                    input.parse::<Token![=]>()?;
                    aggregate = Some(input.parse()?);
                }
                "id" => {
                    input.parse::<Token![=]>()?;
                    id = Some(input.parse()?);
                }
                "name" => {
                    input.parse::<Token![=]>()?;
                    let value = input.parse::<LitStr>()?;
                    if value.value().trim().is_empty() {
                        return Err(syn::Error::new_spanned(
                            ident,
                            "the event name must not be empty",
                        ));
                    }
                    event_name = Some(value);
                }
                _ => return Err(syn::Error::new_spanned(ident, "unknown argument")),
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(DeriveAggregateEventArguments {
            aggregate,
            id,
            event_name,
        })
    }
}

impl TryFrom<&[Attribute]> for DeriveAggregateEventArguments {
    type Error = syn::Error;

    fn try_from(attributes: &[Attribute]) -> Result<Self, Self::Error> {
        for attribute in attributes {
            if has_name(attribute, "presage") {
                return attribute.parse_args();
            }
        }
        Ok(DeriveAggregateEventArguments::default())
    }
}

struct AggregateEventInfo {
    type_name: Ident,
    aggregate: Path,
    id_spec: IdSpec,
    event_name: LitStr,
}

impl TryFrom<Item> for AggregateEventInfo {
    type Error = TokenStream;

    fn try_from(item: Item) -> Result<Self, Self::Error> {
        match item {
            Item::Struct(item) => item.try_into(),
            Item::Enum(item) => item.try_into(),
            _ => Err(error(
                item,
                "AggregateEvent can only be derived for a struct or an enum",
            )),
        }
    }
}

impl TryFrom<ItemStruct> for AggregateEventInfo {
    type Error = TokenStream;

    fn try_from(item: ItemStruct) -> Result<Self, Self::Error> {
        let arguments = DeriveAggregateEventArguments::try_from(item.attrs.as_slice())
            .map_err(syn::Error::into_compile_error)?;
        let aggregate = arguments
            .aggregate
            .ok_or_else(|| error(item.clone(), MISSING_AGGREGATE_ERROR))?;

        let id_spec = if let Some(id) = arguments.id {
            IdSpec::Struct(IdField::Named(id))
        } else {
            IdSpec::Struct(item.fields.try_into()?)
        };

        let event_name = arguments
            .event_name
            .unwrap_or_else(|| create_str_literal_from_ident(&item.ident));

        Ok(AggregateEventInfo {
            type_name: item.ident,
            aggregate,
            id_spec,
            event_name,
        })
    }
}

impl TryFrom<ItemEnum> for AggregateEventInfo {
    type Error = TokenStream;

    fn try_from(item: ItemEnum) -> Result<Self, Self::Error> {
        let arguments = DeriveAggregateEventArguments::try_from(item.attrs.as_slice())
            .map_err(syn::Error::into_compile_error)?;

        let aggregate = arguments
            .aggregate
            .ok_or_else(|| error(item.clone(), MISSING_AGGREGATE_ERROR))?;

        let id_spec = IdSpec::Enum(
            item.variants
                .into_iter()
                .map(|variant| get_variant_spec(variant, &arguments.id))
                .collect::<Result<_, _>>()?,
        );

        let event_name = arguments
            .event_name
            .unwrap_or_else(|| create_str_literal_from_ident(&item.ident));

        Ok(AggregateEventInfo {
            type_name: item.ident,
            aggregate,
            id_spec,
            event_name,
        })
    }
}

fn get_variant_spec(
    variant: Variant,
    default_id: &Option<Ident>,
) -> Result<VariantSpec, TokenStream> {
    let arguments = DeriveAggregateEventArguments::try_from(variant.attrs.as_slice())
        .map_err(syn::Error::into_compile_error)?;
    let id = variant.fields.try_into().or_else(|error| {
        if let Some(id) = arguments.id {
            Ok(IdField::Named(id))
        } else if let Some(id) = default_id {
            Ok(IdField::Named(id.clone()))
        } else {
            Err(error)
        }
    })?;
    Ok(VariantSpec {
        variant: variant.ident,
        id,
    })
}

enum IdSpec {
    Struct(IdField),
    Enum(Vec<VariantSpec>),
}

impl ToTokens for IdSpec {
    fn to_tokens(&self, tokens: &mut quote::__private::TokenStream) {
        match self {
            Self::Struct(id) => id.to_tokens(tokens),
            Self::Enum(variants) => {
                if variants.is_empty() {
                    tokens.append_all(quote! {unreachable!()})
                } else {
                    tokens.append_all(quote! {
                        match self {
                            #(#variants)*
                        }
                    })
                }
            }
        }
    }
}

struct VariantSpec {
    variant: Ident,
    id: IdField,
}

impl ToTokens for VariantSpec {
    fn to_tokens(&self, tokens: &mut quote::__private::TokenStream) {
        let variant = &self.variant;
        match &self.id {
            IdField::Named(id) => {
                tokens.append_all(quote! {Self::#variant{ #id, .. } => #id.clone(),})
            }
            IdField::Unnamed(index) => {
                let underscores: Vec<_> = (0..*index).map(|_| quote! {_,}).collect();
                tokens.append_all(quote! {Self::#variant(#(#underscores)*id, ..) => id.clone(),})
            }
        }
    }
}

enum IdField {
    Named(Ident),
    Unnamed(usize),
}

impl ToTokens for IdField {
    fn to_tokens(&self, tokens: &mut quote::__private::TokenStream) {
        match self {
            IdField::Named(id) => tokens.append_all(quote! {self.#id.clone()}),
            IdField::Unnamed(index) => {
                let index = syn::Index::from(*index);
                tokens.append_all(quote! {self.#index.clone()})
            }
        }
    }
}

impl TryFrom<Fields> for IdField {
    type Error = TokenStream;

    fn try_from(fields: Fields) -> Result<Self, Self::Error> {
        match fields {
            Fields::Named(fields) => fields.try_into(),
            Fields::Unnamed(fields) => fields.try_into(),
            Fields::Unit => Err(error(
                fields,
                "An id field is required when deriving AggregateEvent",
            )),
        }
    }
}

impl TryFrom<FieldsNamed> for IdField {
    type Error = TokenStream;

    fn try_from(fields: FieldsNamed) -> Result<Self, Self::Error> {
        fields
            .named
            .iter()
            .find_map(|field| {
                field
                    .attrs
                    .iter()
                    .find(|attribute| is_id(attribute))
                    .and(field.ident.clone())
                    .map(IdField::Named)
            })
            .ok_or_else(|| missing_id(fields))
    }
}

impl TryFrom<FieldsUnnamed> for IdField {
    type Error = TokenStream;

    fn try_from(fields: FieldsUnnamed) -> Result<Self, Self::Error> {
        fields
            .unnamed
            .iter()
            .enumerate()
            .find_map(|(index, field)| {
                field
                    .attrs
                    .iter()
                    .any(is_id)
                    .then_some(IdField::Unnamed(index))
            })
            .ok_or_else(|| missing_id(fields))
    }
}

fn is_id(attribute: &Attribute) -> bool {
    has_name(attribute, "id")
}

fn missing_id(span: impl ToTokens) -> TokenStream {
    error(span, MISSING_ID_ATTRIBUTE_ERROR)
}

const MISSING_AGGREGATE_ERROR: &str = r"When deriving AggregateEvent, the aggregate type must be specified using the #[presage] attribute.

help: use `#[presage(<path>)]` or `#[presage(aggregate = <path>)]`";

const MISSING_ID_ATTRIBUTE_ERROR: &str = r"When deriving AggregateEvent, an id field must be specified.

help: use the `#[id]` on the field or `#[presage(id = <ident>)]` on the container";
