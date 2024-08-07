mod default;
mod prefix;
mod util;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn prefix(_args: TokenStream, input: TokenStream) -> TokenStream {
    util::syn_result_to_token_stream(prefix::prefix(input.into()))
}

#[proc_macro_derive(ClapDefault)]
pub fn derive_default(input: TokenStream) -> TokenStream {
    util::syn_result_to_token_stream(default::derive_default(input.into()))
}
