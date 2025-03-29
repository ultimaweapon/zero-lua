use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use std::ffi::CString;
use syn::{Error, ImplItem, ItemImpl, LitCStr, Type};

pub fn transform(item: ItemImpl) -> syn::Result<TokenStream> {
    // Check impl block.
    if let Some(v) = &item.defaultness {
        return Err(Error::new_spanned(
            v,
            "default implementation is not supported",
        ));
    }

    if let Some(v) = &item.unsafety {
        return Err(Error::new_spanned(
            v,
            "unsafe implementation is not supported",
        ));
    }

    if item.generics.lt_token.is_some() {
        return Err(Error::new_spanned(
            item.generics,
            "generic implementation is not supported",
        ));
    }

    if let Some((_, v, _)) = &item.trait_ {
        return Err(Error::new_spanned(
            v,
            "trait implementation cannot be used here",
        ));
    }

    // Get type.
    let ty = match item.self_ty.as_ref() {
        Type::Path(v) => v,
        v => return Err(Error::new_spanned(v, "unsupported type")),
    };

    // Parse items.
    let mut count = 0u16;
    let mut setters = TokenStream::new();

    for i in &item.items {
        // Check if function.
        let f = match i {
            ImplItem::Fn(v) => v,
            i => return Err(Error::new_spanned(i, "unsupported item")),
        };

        // Get function name.
        let ident = &f.sig.ident;
        let span = Span::mixed_site();
        let name = CString::new(ident.to_string().replace('_', "")).unwrap();
        let name = LitCStr::new(&name, Span::call_site());

        count += 1;
        setters.extend(quote_spanned! {span=>
            t.set(#name).push_fn(|cx| cx.to_ud::<Self>().#ident(cx));
        });
    }

    // Compose.
    let ident = ty.path.require_ident()?;
    let name = CString::new(ident.to_string()).unwrap();
    let name = LitCStr::new(&name, Span::call_site());

    Ok(quote! {
        #item

        impl ::zl::UserData for #ident {
            fn name() -> &'static ::core::ffi::CStr {
                #name
            }

            fn setup_metatable<P: ::zl::Frame>(t: &mut ::zl::Table<P>) {
                use ::zl::Frame;

                let mut s = t.set(c"__index");
                let mut t = s.push_table(0, #count);

                #setters
            }
        }
    })
}
