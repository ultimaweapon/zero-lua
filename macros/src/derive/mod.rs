use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Error, Fields, Item, ItemEnum, LitByteStr};

pub fn user_data(item: Item) -> syn::Result<TokenStream> {
    // Get type identifier.
    let ident = match item {
        Item::Struct(v) => {
            if v.generics.lt_token.is_some() {
                return Err(Error::new_spanned(
                    v.ident,
                    "generic struct is not supported",
                ));
            }

            v.ident
        }
        v => return Err(Error::new_spanned(v, "unsupported item")),
    };

    // Compose.
    Ok(quote! {
        impl ::zl::UserData for #ident {
            fn name() -> &'static ::core::ffi::CStr {
                static NAME: ::std::sync::LazyLock<::std::ffi::CString> = ::std::sync::LazyLock::new(|| ::std::ffi::CString::new(::std::any::type_name::<#ident>()).unwrap());
                NAME.as_c_str()
            }
        }
    })
}

pub fn from_option(item: ItemEnum) -> syn::Result<TokenStream> {
    // Parse variants.
    let mut arms = TokenStream::new();

    for v in item.variants {
        // Check type.
        if !matches!(v.fields, Fields::Unit) {
            return Err(Error::new_spanned(v, "unsupported variant kind"));
        }

        // Check if ASCII.
        let ident = v.ident;
        let name = ident.to_string();

        if !name.is_ascii() {
            return Err(Error::new_spanned(ident, "non-ASCII name is not supported"));
        }

        // Render match arm.
        let pattern = LitByteStr::new(name.to_ascii_lowercase().as_bytes(), Span::call_site());

        arms.extend(quote! {
            #pattern => ::core::option::Option::Some(Self::#ident),
        });
    }

    // Compose.
    let ident = item.ident;

    Ok(quote! {
        impl ::zl::FromOption for #ident {
            fn from_option(v: &[u8]) -> ::core::option::Option<Self> {
                match v {
                    #arms
                    _ => ::core::option::Option::None,
                }
            }
        }
    })
}
