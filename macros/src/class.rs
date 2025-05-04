use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use std::ffi::CString;
use std::num::NonZero;
use syn::meta::ParseNestedMeta;
use syn::{
    AttrStyle, Error, Expr, ImplItem, ImplItemConst, ImplItemFn, ItemImpl, Lit, LitCStr, Type,
    Visibility, parse_quote,
};

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
    let mut globs = Vec::new();
    let mut close = TokenStream::new();
    let mut index = TokenStream::new();
    let mut setters = TokenStream::new();
    let mut next_uv = 1;

    for i in &mut item.items {
        match i {
            ImplItem::Const(v) => parse_const(&mut index, &mut setters, &mut next_uv, v)?,
            ImplItem::Fn(v) => parse_fn(&opts, &mut globs, &mut close, &mut index, v)?,
            i => return Err(Error::new_spanned(i, "unsupported item")),
        }
    }

    // Generate setup_metatable.
    let mut meta = TokenStream::new();

    meta.extend(close);

    if !index.is_empty() {
        meta.extend(quote! {
            meta.set(c"__index").push_fn(|cx| {
                let n = cx.to_str(::zl::PositiveInt::TWO);

                match n {
                    #index
                    _ => drop(cx.push_nil()),
                }

                Ok(())
            });
        });
    }

    // Generate register.
    let glob = match globs.len() {
        0 => TokenStream::new(),
        n => {
            let n = u16::try_from(n).unwrap();
            let mut t = quote! {
                fn register<P: ::zl::Frame>(mut g: ::zl::GlobalSetter<P, &::core::ffi::CStr>) {
                    use ::zl::Frame;

                    let mut t = g.push_table(0, #n);
                }
            };

            for g in globs {
                t.extend(g);
            }

            t
        }
    };

    // Generate name.
    let ident = ty.path.require_ident()?;
    let name = match opts.global {
        true => {
            let name = CString::new(ident.to_string()).unwrap();
            let name = LitCStr::new(&name, Span::call_site());

            quote! {
                #[inline(always)]
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

    // Generate setter implementation.
    if !setters.is_empty() {
        setters = quote! {
            impl #ident {
                #setters
            }
        }
    }

    // Compose.
    let uv = u16::try_from(next_uv - 1).unwrap();

    if !meta.is_empty() {
        meta = quote! {
            fn setup<P: ::zl::Frame>(meta: &mut ::zl::Table<P>) {
                use ::zl::Frame;
                #meta
            }
        }
    }

    Ok(quote! {
        #item

        impl ::zl::UserType for #ident {
            #name

            #[inline(always)]
            fn user_values() -> Option<::std::num::NonZero<u16>> {
                :std::num::NonZero<u16>::new(#uv)
            }

            #meta
            #glob
        }

        #setters
    })
}

fn parse_const(
    index: &mut TokenStream,
    setters: &mut TokenStream,
    next: &mut u32,
    c: &mut ImplItemConst,
) -> syn::Result<()> {
    let ident = &c.ident;

    if let Some(t) = &c.defaultness {
        return Err(Error::new_spanned(t, "default const is not supported"));
    }

    if c.generics.lt_token.is_some() {
        return Err(Error::new_spanned(ident, "generic const is not supported"));
    }

    // Replace type and value.
    let ty = std::mem::replace(&mut c.ty, parse_quote!(::std::num::NonZero<u16>));
    let uv: u16 = match &c.expr {
        Expr::Infer(_) => {
            let v = next
                .clone()
                .try_into()
                .map_err(|_| Error::new_spanned(ident, "the index for property out of range"))?;

            *next += 1;

            v
        }
        Expr::Lit(v) => match &v.lit {
            Lit::Int(l) => {
                let v = l.base10_parse::<NonZero<u16>>()?.get();

                if u32::from(v) < *next {
                    return Err(Error::new_spanned(
                        l,
                        "the value is lower than the previous property",
                    ));
                }

                *next = u32::from(v) + 1;

                v
            }
            v => return Err(Error::new_spanned(v, "unsupported literal")),
        },
        v => return Err(Error::new_spanned(v, "unsupported expression")),
    };

    c.expr = parse_quote!(::std::num::NonZero<u16>::new(#uv).unwrap());

    // Generate Lua accessor.
    let name = ident.to_string().to_ascii_lowercase();
    let span = Span::call_site();

    index.extend(quote_spanned! {span=>
        #name => cx.push_uv::<Self>(::zl::PositiveInt::ONE, #uv.try_into().unwrap()),
    });

    // Generate Rust setter.
    let vis = std::mem::replace(&mut c.vis, Visibility::Inherited);
    let setter = format_ident!("set_{name}");

    setters.extend(quote_spanned! {span=>
        #vis fn #setter<T: ::zl::TypedUd<Type = Self>>(ud: &mut T, v: #ty) {
            ::zl::IntoLua::into_lua(v, &mut ud.set_uv(#uv.try_into().unwrap()).unwrap());
        }
    });

    Ok(())
}

fn parse_fn(
    opts: &Options,
    globs: &mut Vec<TokenStream>,
    close: &mut TokenStream,
    index: &mut TokenStream,
    f: &mut ImplItemFn,
) -> syn::Result<()> {
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
    let name = ident.to_string();

    match ty {
        FnType::Method => index.extend(quote_spanned! {span=>
            #name => drop(cx.push_fn(Self::#ident)),
        }),
        FnType::AsyncMethod => index.extend(quote_spanned! {span=>
            #name => drop(cx.push_async(Self::#ident)),
        }),
        FnType::Property => index.extend(quote_spanned! {span=>
            #name => return Self::#ident(cx),
        }),
        FnType::ClassMethod => {
            let name = CString::new(name).unwrap();
            let name = LitCStr::new(&name, Span::call_site());

            globs.push(quote_spanned! {span=>
                t.set(#name).push_fn(Self::#ident);
            });
        }
        FnType::Close => {
            *close = quote! {
                meta.set(c"__close").push_fn(Self::#ident);
            }
        }
    }

    Ok(())
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
