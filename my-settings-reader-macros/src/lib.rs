extern crate proc_macro;
use proc_macro::TokenStream;

mod fn_impl_settings_reader;

#[proc_macro_derive(SettingsModel, attributes(file_name))]
pub fn settings_model(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    crate::fn_impl_settings_reader::generate(&ast)
}

#[proc_macro]
pub fn render_settings_reader(input: TokenStream) -> TokenStream {
    let file_name: proc_macro2::TokenStream = input.into();
    let result = quote::quote! {
        pub struct SettingsReader {
            settings: tokio::sync::Mutex<Option<std::sync::Arc<SettingsModel>>>,
        }

        impl SettingsReader {
            pub fn new() -> Self {
                Self {
                    settings: tokio::sync::Mutex::new(None),
                }
            }

            pub async fn get_settings(&self) -> std::sync::Arc<SettingsModel> {
                let mut settings_access = self.settings.lock().await;

                loop {
                    if let Some(settings_access) = settings_access.clone() {
                        return settings_access;
                    }

                    let file_name = rust_extensions::file_utils::format_path(#file_name);

                    let content = tokio::fs::read_to_string(file_name.as_str()).await;

                    if let Err(err) = &content {
                        panic!(
                            "Can not read settings file '{}'. Err:{}",
                            file_name.as_str(),
                            err
                        );
                    }

                    let content = content.unwrap();

                    let model: SettingsModel = serde_yaml::from_str(content.as_str()).unwrap();

                    let model = std::sync::Arc::new(model);

                    *settings_access = Some(model);
                }
            }
        }
    };

    result.into()
}
