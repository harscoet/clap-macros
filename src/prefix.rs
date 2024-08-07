use proc_macro2::TokenStream;
use quote::ToTokens;
use std::{collections::HashSet, str::FromStr};
use syn::{punctuated::Punctuated, spanned::Spanned, Field, ItemStruct, Meta, Result, Token};

use crate::util;

#[derive(PartialEq, Eq, Hash, Clone)]
enum ClapArgIdent {
    Id,
    Long,
    Env,
    ValueName,
}

impl FromStr for ClapArgIdent {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "id" => Ok(Self::Id),
            "long" => Ok(Self::Long),
            "env" => Ok(Self::Env),
            "value_name" => Ok(Self::ValueName),
            _ => Err(()),
        }
    }
}

pub fn prefix(input: TokenStream) -> Result<TokenStream> {
    let mut item: ItemStruct = syn::parse2(input)?;
    let prefix = item.ident.to_string().to_lowercase();

    for field in item.fields.iter_mut() {
        update_field(field, &prefix)?;
    }

    Ok(item.into_token_stream())
}

fn update_field(field: &mut Field, prefix: &str) -> Result<()> {
    let field_ident = field.ident.as_ref().ok_or(syn::Error::new(
        field.span(),
        "Expected field to have an identifier",
    ))?;

    let snake_case_value = format!("{prefix}_{field_ident}");
    let screaming_case_value = snake_case_value.to_uppercase();
    let kebab_case_value = snake_case_value.replace('_', "-");

    for attr in field.attrs.iter_mut() {
        if !attr.path().is_ident("arg") {
            continue;
        }

        let span = attr.span();

        if let Meta::List(ref mut list) = &mut attr.meta {
            let mut visited: HashSet<ClapArgIdent> = HashSet::new();
            let mut new_meta_list: Punctuated<Meta, Token![,]> = Punctuated::new();

            for meta in list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)? {
                let meta_ident: String = meta
                    .path()
                    .get_ident()
                    .map(|x| x.to_string())
                    .unwrap_or_default();

                if let Ok(clap_arg_ident) = ClapArgIdent::from_str(meta_ident.as_str()) {
                    visited.insert(clap_arg_ident.clone());

                    if let Meta::NameValue(_) = meta {
                        new_meta_list.push(meta);
                    } else {
                        new_meta_list.push(util::new_meta_name_str_value(
                            &meta_ident,
                            match clap_arg_ident {
                                ClapArgIdent::Id => &snake_case_value,
                                ClapArgIdent::Long => &kebab_case_value,
                                ClapArgIdent::Env => &screaming_case_value,
                                ClapArgIdent::ValueName => &screaming_case_value,
                            },
                            span,
                        ));
                    }
                } else {
                    new_meta_list.push(meta);
                }
            }

            if !visited.contains(&ClapArgIdent::Id) {
                new_meta_list.push(util::new_meta_name_str_value("id", &snake_case_value, span));
            }

            if !visited.contains(&ClapArgIdent::Long) {
                new_meta_list.push(util::new_meta_name_str_value(
                    "long",
                    &kebab_case_value,
                    span,
                ));
            }

            if !visited.contains(&ClapArgIdent::Env) {
                new_meta_list.push(util::new_meta_name_str_value(
                    "env",
                    &screaming_case_value,
                    span,
                ));
            }

            if !visited.contains(&ClapArgIdent::ValueName) {
                new_meta_list.push(util::new_meta_name_str_value(
                    "value_name",
                    &screaming_case_value,
                    span,
                ));
            }

            list.tokens = new_meta_list.to_token_stream();
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_prefix() -> syn::Result<()> {
        let input = quote! {
            pub struct Etcd {
                #[arg(long, env, default_value = DEFAULT_ETCD_ENDPOINT)]
                pub endpoints: Vec<String>,
                #[arg(long, env)]
                pub auth_password: Option<String>,
                #[arg(long = "custom-init-batch-size", env, default_value_t = 100)]
                pub init_batch_size: i64,
                #[arg(long, env, value_name = "CUSTOM_MAX_DECODING_MESSAGE_SIZE", default_value_t = DEFAULT_ETCD_MAX_DECODING_MESSAGE_SIZE)]
                pub max_decoding_message_size: usize,
                #[arg(long, env = "CUSTOM_ETCD_ENV_NAMESPACE", value_delimiter = ',')]
                pub namespace: Vec<u8>,
            }
        };

        assert_eq!(
            prefix(input.into())?.to_string(),
            quote! {
                pub struct Etcd {
                    #[arg(long = "etcd-endpoints", env = "ETCD_ENDPOINTS", default_value = DEFAULT_ETCD_ENDPOINT, id = "etcd_endpoints", value_name = "ETCD_ENDPOINTS")]
                    pub endpoints: Vec<String>,
                    #[arg(long = "etcd-auth-password", env = "ETCD_AUTH_PASSWORD", id = "etcd_auth_password", value_name = "ETCD_AUTH_PASSWORD")]
                    pub auth_password: Option<String>,
                    #[arg(long = "custom-init-batch-size", env = "ETCD_INIT_BATCH_SIZE", default_value_t = 100, id = "etcd_init_batch_size", value_name = "ETCD_INIT_BATCH_SIZE")]
                    pub init_batch_size: i64,
                    #[arg(long = "etcd-max-decoding-message-size", env = "ETCD_MAX_DECODING_MESSAGE_SIZE", value_name = "CUSTOM_MAX_DECODING_MESSAGE_SIZE", default_value_t = DEFAULT_ETCD_MAX_DECODING_MESSAGE_SIZE, id = "etcd_max_decoding_message_size")]
                    pub max_decoding_message_size: usize,
                    #[arg(long = "etcd-namespace", env = "CUSTOM_ETCD_ENV_NAMESPACE", value_delimiter = ',', id = "etcd_namespace", value_name = "ETCD_NAMESPACE")]
                    pub namespace: Vec<u8>,
                }
            }
            .to_string()
        );

        Ok(())
    }
}
