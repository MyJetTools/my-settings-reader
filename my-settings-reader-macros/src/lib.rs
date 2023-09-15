extern crate proc_macro;
use proc_macro::TokenStream;

mod fn_impl_settings_reader;

#[proc_macro_derive(SettingsModel, attributes(file_name))]
pub fn settings_model(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    crate::fn_impl_settings_reader::generate(&ast)
}