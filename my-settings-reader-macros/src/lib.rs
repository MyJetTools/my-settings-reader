extern crate proc_macro;
use proc_macro::TokenStream;

mod fn_impl_settings_reader;

#[proc_macro_derive(SettingsModel, attributes(file_name))]
pub fn settings_model(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    crate::fn_impl_settings_reader::generate(&ast)
}

#[proc_macro]
pub fn dioxus_settings_reader(_input: TokenStream) -> TokenStream {
    let file_name = _input.to_string();
    let result = quote::quote! {
        pub struct SettingsReader {
            settings: Mutex<Option<Arc<SettingsModel>>>,
        }

        impl SettingsReader {
            pub fn new() -> Self {
                Self {
                    settings: Mutex::new(None),
                }
            }

            pub async fn get_settings(&self) -> Arc<SettingsModel> {
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

                    let model = Arc::new(model);

                    *settings_access = Some(model);
                }
            }
        }
    };

    result.into()
}
