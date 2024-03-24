#![allow(dead_code, unused_variables, unused_features)]
use proc_macro::TokenStream;

use quote::{format_ident, quote_spanned, ToTokens};
use syn::{parse::Parse, parse_macro_input, spanned::Spanned, Attribute, DeriveInput};

/// A convenience macro which derives a lotta things to make your type work in all supported/enabled scripting languages, and provide static typing where possible.
///
/// This macro is used extensively in `bevy_mod_scripting/src/generated.rs`, for extensive usage examples see those macro invocations.
///
/// Right now the macro supports:
/// - primitive types surrounded in `Raw()`
///   - usize
///   - isize
///   - f32
///   - f64
///   - u128
///   - u64
///   - u32
///   - u16
///   - u8
///   - i128
///   - i64
///   - i32
///   - i16
///   - i8
///   - String
///   - bool
/// - other wrapper types generated by this macro surrounded in `Wrapper()`
/// - Both mutable and immutable references to any of the above (apart from on fields)
/// - the self type and receiver (self, &self or &mut self), if used in method must be followed by `:` to differentiate it from other self arguments  
/// Currently more complex types like: Option<T> and LuaWrapper<T> are not yet supported (although they have Proxy implementations which can be manually implemented).
///  
/// # Example
/// ```rust,ignore
/// use bevy_mod_scripting_derive::impl_script_newtype;
///
/// pub struct MyStruct{
///     my_field: bool
/// }
///
/// impl MyStruct {
///     pub fn do_something(&self) -> bool {
///         self.my_field
///     }
/// }
/// impl_script_newtype!(
///     #[languages(lua,rhai)]
///     MyStruct:
///       Fields(
///         my_field: Raw(bool)
///       ) + Methods(
///         do_something(&self:) -> Raw(bool)
///       )
/// );
/// // a new type is created
/// // with all neccessary traits implemented
/// println!("{:?}", std::any::TypeId::Of::<LuaMyStruct>);
/// ```
#[proc_macro]
#[deprecated(
    note = "this macro will be removed in the next release, please use the new derive macros",
    since = "0.3.0"
)]
pub fn impl_script_newtype(input: TokenStream) -> TokenStream {
    // let invocation = parse_macro_input!(input as MacroInvocation);
    // let inner = invocation.inner;
    return input.into();
    // let mut output: proc_macro2::TokenStream = Default::default();
    // // find the language implementor macro id's
    // match invocation.languages.parse_meta() {
    //     Ok(syn::Meta::List(list)) => {
    //         if !list.path.is_ident("languages") {
    //             return syn::Error::new_spanned(list, "Expected `langauges(..)` meta list")
    //                 .to_compile_error()
    //                 .into();
    //         }

    //         // now create an invocation per language specified
    //         for language in &list.nested {
    //             let mut feature_gate = false;
    //             let mut inner_language = None;
    //             if let syn::NestedMeta::Meta(syn::Meta::List(sub_list)) = language {
    //                 if sub_list.path.is_ident("on_feature") {
    //                     if let Some(syn::NestedMeta::Meta(syn::Meta::Path(path))) =
    //                         sub_list.nested.first()
    //                     {
    //                         if let Some(ident) = path.get_ident() {
    //                             inner_language = Some(ident);
    //                             feature_gate = true;
    //                         }
    //                     }
    //                 }
    //             } else if let syn::NestedMeta::Meta(syn::Meta::Path(path)) = language {
    //                 if let Some(ident) = path.get_ident() {
    //                     inner_language = Some(ident)
    //                 }
    //             }

    //             let inner_language =
    //                 match inner_language {
    //                     Some(v) => v,
    //                     None => return syn::Error::new_spanned(
    //                         language,
    //                         "Expected `on_feature(x)` or `x` attribute where x is a valid language",
    //                     )
    //                     .to_compile_error()
    //                     .into(),
    //                 };

    //             let lang_str = inner_language.to_string();
    //             let macro_ident = format_ident!("impl_{}_newtype", inner_language);
    //             let inner = invocation.inner.clone();
    //             let feature_gate = feature_gate.then_some(quote::quote!(#[cfg(feature=#lang_str)]));
    //             output.extend(quote_spanned! {language.span()=>
    //                 #feature_gate
    //                 #macro_ident!{
    //                     #inner
    //                 }
    //             });
    //         }
    //     }
    //     _ => {
    //         return syn::Error::new(
    //             invocation.span(),
    //             "Expected attribute of the form #[languages(..)]",
    //         )
    //         .to_compile_error()
    //         .into()
    //     }
    // };

    // output.into()
}

pub(crate) struct MacroInvocation {
    pub languages: Attribute,
    pub inner: proc_macro2::TokenStream,
}

impl Parse for MacroInvocation {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            languages: Attribute::parse_outer(input)?
                .into_iter()
                .next()
                .ok_or_else(|| {
                    syn::Error::new(
                        input.span(),
                        "Expected meta attribute selecting language implementors",
                    )
                })?,
            inner: input.parse()?,
        })
    }
}

impl ToTokens for MacroInvocation {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let languages = &self.languages;
        let inner = &self.inner;
        tokens.extend(quote::quote! {
            #languages
            #inner
        });
    }
}