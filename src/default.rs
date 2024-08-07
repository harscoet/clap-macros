use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    punctuated::Punctuated, Attribute, Expr, ExprLit, Field, ItemStruct, Lit, Meta, MetaNameValue,
    Result, Token,
};

use crate::util;

const LIST_DELIMITER: char = ',';

pub fn derive_default(input: TokenStream) -> Result<TokenStream> {
    let item: ItemStruct = syn::parse2(input)?;
    let mut default_values = Vec::with_capacity(item.fields.len());
    let struct_name = item.ident;

    for field in item.fields.iter() {
        default_values.push(parse_field(field)?);
    }

    let expanded = quote! {
        impl Default for #struct_name {
            fn default() -> Self {
                Self {
                    #(#default_values),*
                }
            }
        }
    };

    Ok(expanded)
}

fn parse_field(field: &Field) -> Result<TokenStream> {
    let mut default_value: Option<TokenStream> = None;

    let field_name = &field.ident;
    let field_type = &field.ty;

    if let (Some(field_type_string), field_type_argument) = util::get_field_type(field_type) {
        for attr in &field.attrs {
            if !attr.path().is_ident("arg") {
                continue;
            }

            default_value =
                parse_field_attribute(attr, &field_type_string, field_type_argument.as_deref())?;
        }
    }

    Ok(if default_value.is_none() {
        quote! {
            #field_name: <#field_type>::default()
        }
    } else {
        quote! {
            #field_name: #default_value
        }
    })
}

fn parse_field_attribute(
    attribute: &Attribute,
    field_type_string: &str,
    field_type_argument: Option<&str>,
) -> Result<Option<TokenStream>> {
    for meta in attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)? {
        if let Some(meta_ident) = meta.path().get_ident() {
            if meta_ident == "default_value" || meta_ident == "default_value_t" {
                if let Meta::NameValue(MetaNameValue { value, .. }) = meta {
                    return Ok(Some(parse_field_attribute_default_value(
                        field_type_string,
                        field_type_argument,
                        value,
                    )?));
                }
            }
        }
    }

    Ok(None)
}

fn parse_field_attribute_default_value(
    field_type_string: &str,
    field_type_argument: Option<&str>,
    value: Expr,
) -> Result<TokenStream> {
    match value {
        Expr::Lit(ExprLit {
            attrs: _,
            lit: Lit::Str(value),
        }) => match (field_type_string, field_type_argument) {
            ("String", _) => Ok(syn::parse_quote!(#value.into())),
            ("Url", _) => Ok(syn::parse_quote!(#value.parse().unwrap())),
            ("Vec", Some(vec_type)) => {
                let value = value.value();

                let token_stream = match vec_type {
                    "String" => value
                        .split(LIST_DELIMITER)
                        .map(|x| x.trim())
                        .map(|x| Ok(quote! { #x.to_string() }))
                        .collect::<Result<Vec<_>>>(),
                    _ => value
                        .split(LIST_DELIMITER)
                        .map(|x| Ok(syn::parse_str::<syn::Expr>(x.trim())?.to_token_stream()))
                        .collect::<Result<Vec<_>>>(),
                }?;

                Ok(syn::parse_quote!(vec![#(#token_stream),*]))
            }
            _ => Ok(value.to_token_stream()),
        },
        _ => Ok(value.to_token_stream()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_derive_default() -> syn::Result<()> {
        let input = quote! {
            #[derive(DefaultFromClap)]
            struct Parameters {
                #[arg(long, default_value_t = 8080)]
                port: u16,
                #[arg(long, default_value = "localhost")]
                host: String,
                #[arg(long, default_value = "aa,bb, cc")]
                domains: Vec<String>,
                #[arg(long, default_value = "1,2,3")]
                numbers: Vec<u16>,
                #[arg(long, default_value = "true, false")]
                booleans: Vec<bool>,
                #[arg(long, default_value = "https://www.google.com")]
                url: Url,
                #[arg(long = "parameters-mode",env = "PARAMETERS_MODE",value_enum,default_value_t = Mode::Release,id = "parameters_mode",value_name = "PARAMETERS_MODE")]
                mode: Mode,
                #[arg(long)]
                tls: bool,
                #[arg(long)]
                option: Option<String>,
            }
        };

        assert_eq!(
            derive_default(input.into())?.to_string(),
            quote! {
                impl Default for Parameters {
                    fn default() -> Self {
                        Self {
                            port: 8080,
                            host: "localhost".into(),
                            domains: vec![
                                "aa".to_string(),
                                "bb".to_string(),
                                "cc".to_string()
                            ],
                            numbers: vec![1, 2, 3],
                            booleans: vec![true, false],
                            url: "https://www.google.com".parse().unwrap(),
                            mode: Mode::Release,
                            tls: <bool>::default(),
                            option: < Option<String> >::default()
                        }
                    }
                }
            }
            .to_string()
        );

        Ok(())
    }
}
