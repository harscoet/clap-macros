use proc_macro2::{Span, TokenStream};
use syn::{
    AngleBracketedGenericArguments, Expr, ExprLit, GenericArgument, Ident, Lit, LitStr, Meta,
    MetaNameValue, PathArguments, PathSegment, Token, Type,
};

pub fn get_field_type(field_type: &Type) -> (Option<String>, Option<String>) {
    if let Type::Path(type_path) = field_type {
        if let Some(segment) = type_path.path.segments.last() {
            (
                Some(segment.ident.to_string()),
                get_argument_type_from_path_segment(segment),
            )
        } else {
            (None, None)
        }
    } else {
        (None, None)
    }
}

fn get_argument_type_from_path_segment(segment: &PathSegment) -> Option<String> {
    if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
        &segment.arguments
    {
        if let Some(GenericArgument::Type(kind)) = args.last() {
            return get_field_type(kind).0;
        }
    }

    None
}

pub fn new_meta_name_str_value(ident: &str, value: &str, span: Span) -> Meta {
    Meta::NameValue(MetaNameValue {
        path: Ident::new(ident, span).into(),
        eq_token: Token![=]([span]),
        value: Expr::Lit(ExprLit {
            attrs: Vec::new(),
            lit: Lit::Str(LitStr::new(value, span)),
        }),
    })
}

pub fn syn_result_to_token_stream(result: syn::Result<TokenStream>) -> proc_macro::TokenStream {
    match result {
        Ok(token_stream) => token_stream.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
