use proc_macro::TokenStream;
use syn::{Error, Item, ItemEnum, ItemImpl, parse_macro_input};

mod class;
mod derive;

#[proc_macro_attribute]
pub fn class(arg: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemImpl);
    let mut opts = self::class::Options::default();
    let parser = syn::meta::parser(|m| opts.parse(m));

    parse_macro_input!(arg with parser);

    self::class::transform(item, opts)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

#[proc_macro_derive(FromOption)]
pub fn derive_from_option(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemEnum);

    self::derive::from_option(item)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

#[proc_macro_derive(UserData)]
pub fn derive_user_data(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as Item);

    self::derive::user_data(item)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
