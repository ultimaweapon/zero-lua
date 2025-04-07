use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use std::ffi::CString;
use syn::meta::ParseNestedMeta;
use syn::{AttrStyle, Error, ImplItem, ItemImpl, LitCStr, Type};

pub fn transform(mut item: ItemImpl, opts: Options) -> syn::Result<TokenStream> {
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
    let mut meta_len = 0u16;
    let mut meta_set = TokenStream::new();
    let mut glob_len = 0u16;
    let mut glob_set = TokenStream::new();

    for i in &mut item.items {
        // Check if function.
        let f = match i {
            ImplItem::Fn(v) => v,
            i => return Err(Error::new_spanned(i, "unsupported item")),
        };

        // Parse attributes.
        let mut class = None;
        let mut i = 0;

        while i < f.attrs.len() {
            // Skip inner.
            let a = &f.attrs[i];

            if !matches!(a.style, AttrStyle::Outer) {
                i += 1;
                continue;
            }

            // Check name.
            if a.path().is_ident("class") {
                if !opts.global {
                    return Err(Error::new_spanned(
                        a,
                        "class method on non-global class is not supported",
                    ));
                }

                f.attrs.remove(i);
                class = Some(());
                continue;
            }

            i += 1;
        }

        // Get function name.
        let ident = &f.sig.ident;
        let span = Span::call_site();
        let name = CString::new(ident.to_string().replace('_', "")).unwrap();
        let name = LitCStr::new(&name, Span::call_site());

        match class {
            Some(_) => {
                glob_len += 1;
                glob_set.extend(quote_spanned! {span=>
                    t.set(#name).push_fn(Self::#ident);
                });
            }
            None => {
                meta_len += 1;
                meta_set.extend(quote_spanned! {span=>
                    t.set(#name).push_fn(|cx| cx.to_ud::<Self>(1).#ident(cx));
                });
            }
        }
    }

    // Generate setup_metatable.
    let meta = match meta_len {
        0 => TokenStream::new(),
        n => quote! {
            fn setup_metatable<P: ::zl::Frame>(t: &mut ::zl::Table<P>) {
                use ::zl::Frame;

                let mut s = t.set(c"__index");
                let mut t = s.push_table(0, #n);

                #meta_set
            }
        },
    };

    // Generate setup_global.
    let glob = match glob_len {
        0 => TokenStream::new(),
        n => quote! {
            fn setup_global<P: ::zl::Frame>(mut g: ::zl::GlobalSetter<P, &::core::ffi::CStr>) {
                use ::zl::Frame;

                let mut t = g.push_table(0, #n);

                #glob_set
            }
        },
    };

    // Generate name.
    let ident = ty.path.require_ident()?;
    let name = match opts.global {
        true => {
            let name = CString::new(ident.to_string()).unwrap();
            let name = LitCStr::new(&name, Span::call_site());

            quote! {
                fn name() -> &'static ::core::ffi::CStr {
                    #name
                }
            }
        }
        false => quote! {
            fn name() -> &'static ::core::ffi::CStr {
                static NAME: ::std::sync::LazyLock<::std::ffi::CString> = ::std::sync::LazyLock::new(|| ::std::ffi::CString::new(::std::any::type_name::<#ident>()).unwrap());
                NAME.as_c_str()
            }
        },
    };

    // Compose.
    Ok(quote! {
        #item

        impl ::zl::UserData for #ident {
            #name
            #meta
            #glob
        }
    })
}

#[derive(Default)]
pub struct Options {
    global: bool,
}

impl Options {
    pub fn parse(&mut self, m: ParseNestedMeta) -> syn::Result<()> {
        if m.path.is_ident("global") {
            self.global = true;
        } else {
            return Err(m.error("unknown option"));
        }

        Ok(())
    }
}
