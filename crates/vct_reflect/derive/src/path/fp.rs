use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

/// Full Path (FP) for [`core::any::Any`]
pub(crate) struct AnyFP;
/// Full Path (FP) for [`Clone`]
pub(crate) struct CloneFP;
/// Full Path (FP) for [`Default`]
pub(crate) struct DefaultFP;
/// Full Path (FP) for [`Option`]
pub(crate) struct OptionFP;
/// Full Path (FP) for [`Result`]
pub(crate) struct ResultFP;
/// Full Path (FP) for [`Send`]
pub(crate) struct SendFP;
/// Full Path (FP) for [`Sync`]
pub(crate) struct SyncFP;

impl ToTokens for AnyFP {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(::core::any::Any).to_tokens(tokens);
    }
}

impl ToTokens for CloneFP {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(::core::clone::Clone).to_tokens(tokens);
    }
}

impl ToTokens for DefaultFP {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(::core::default::Default).to_tokens(tokens);
    }
}

impl ToTokens for OptionFP {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(::core::option::Option).to_tokens(tokens);
    }
}

impl ToTokens for ResultFP {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(::core::result::Result).to_tokens(tokens);
    }
}

impl ToTokens for SendFP {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(::core::marker::Send).to_tokens(tokens);
    }
}

impl ToTokens for SyncFP {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(::core::marker::Sync).to_tokens(tokens);
    }
}
