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
    let mut glob_len = 0u16;
    let mut glob_set = TokenStream::new();
    let mut close = TokenStream::new();
    let mut index = TokenStream::new();

    for i in &mut item.items {
        // Check if function.
        let f = match i {
            ImplItem::Fn(v) => v,
            i => return Err(Error::new_spanned(i, "unsupported item")),
        };

        // Parse attributes.
        let mut i = 0;
        let mut ty = if f.sig.asyncness.is_some() {
            FnType::AsyncMethod
        } else {
            FnType::Method
        };

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

                ty = FnType::ClassMethod;
            } else if a.path().is_ident("close") {
                if !close.is_empty() {
                    return Err(Error::new_spanned(
                        a,
                        "multiple close method is not supported",
                    ));
                }

                ty = FnType::Close;
            } else if a.path().is_ident("prop") {
                if f.sig.asyncness.is_some() {
                    return Err(Error::new_spanned(a, "async property is not supported"));
                }

                ty = FnType::Property;
            } else {
                i += 1;
                continue;
            }

            f.attrs.remove(i);
            break;
        }

        // Get function name.
        let ident = &f.sig.ident;
        let span = Span::call_site();
        let name = ident.to_string().replace('_', "");

        match ty {
            FnType::Method => index.extend(quote_spanned! {span=>
                #name => drop(cx.push_fn(|cx| cx.to_ud::<Self>(1).#ident(cx))),
            }),
            FnType::AsyncMethod => index.extend(quote_spanned! {span=>
                #name => drop(cx.push_async(async |cx| cx.to_ud::<Self>(1).#ident(cx).await)),
            }),
            FnType::Property => index.extend(quote_spanned! {span=>
                #name => return v.#ident(cx),
            }),
            FnType::ClassMethod => {
                let name = CString::new(name).unwrap();
                let name = LitCStr::new(&name, Span::call_site());

                glob_len += 1;
                glob_set.extend(quote_spanned! {span=>
                    t.set(#name).push_fn(Self::#ident);
                });
            }
            FnType::Close => {
                close = quote! {
                    t.set(c"__close").push_fn(|cx| cx.to_ud::<Self>(1).#ident(cx));
                }
            }
        }
    }

    // Generate setup_metatable.
    let mut meta = TokenStream::new();

    meta.extend(close);

    if !index.is_empty() {
        meta.extend(quote! {
            t.set(c"__index").push_fn(|cx| {
                let v = cx.to_ud::<Self>(1);
                let n = cx.to_str(2);

                match n {
                    #index
                    _ => drop(cx.push_nil()),
                }

                Ok(())
            });
        });
    }

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
    if !meta.is_empty() {
        meta = quote! {
            fn setup_metatable<P: ::zl::Frame>(t: &mut ::zl::Table<P>) {
                use ::zl::Frame;
                #meta
            }
        }
    }

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

enum FnType {
    Method,
    AsyncMethod,
    Property,
    ClassMethod,
    Close,
}
