use proc_macro::TokenStream;

pub fn generate(ast: &syn::DeriveInput) -> TokenStream {
    let struct_name = &ast.ident;

    let mut main_separator = String::new();

    main_separator.push('{');
    main_separator.push('}');
    if std::path::MAIN_SEPARATOR == '/' {
        main_separator.push(std::path::MAIN_SEPARATOR);
    } else {
        main_separator.push(std::path::MAIN_SEPARATOR);
        main_separator.push(std::path::MAIN_SEPARATOR);
    };

    main_separator.push('{');
    main_separator.push('}');

    #[cfg(not(feature = "background-reader"))]
    let br: Option<proc_macro2::TokenStream> = None;

    #[cfg(feature = "background-reader")]
    let br: proc_macro2::TokenStream = quote::quote! {
        pub struct SettingsReader {
            settings: std::sync::Arc<tokio::sync::RwLock<#struct_name>>,
        }
        impl SettingsReader {
            pub async fn new(file_name: &str) -> Self {
                match #struct_name::first_load(file_name).await {
                    FirstLoadResult::FromFile(settings) => {
                        let settings = std::sync::Arc::new(tokio::sync::RwLock::new(settings));
                        tokio::spawn(update_settings_in_a_background(
                            settings.clone(),
                            Some(file_name.to_string()),
                        ));
                        Self { settings }
                    }
                    FirstLoadResult::FromUrl(settings) => {
                        let settings = std::sync::Arc::new(tokio::sync::RwLock::new(settings));
                        tokio::spawn(update_settings_in_a_background(settings.clone(), None));
                        Self { settings }
                    }
                }
            }
            pub async fn get_settings(&self) -> #struct_name {
                self.settings.read().await.clone()
            }
        }
        async fn update_settings_in_a_background(
            settings: std::sync::Arc<tokio::sync::RwLock<#struct_name>>,
            file_name: Option<String>,
        ) {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                let settings = settings.clone();
                let file_name = file_name.clone();
        
                if let Some(file_name) = &file_name {
                    let file_name = file_name.clone();
        
                    tokio::spawn(async move {
                        match #struct_name::read_from_file(file_name).await {
                            Ok(settings_model) => {
                                let mut write_access = settings.write().await;
                                *write_access = settings_model;
                            }
                            Err(err) => {
                                println!("Can not read settings from file. Err: {:?}", err);
                            }
                        }
                    });
                } else {
                    tokio::spawn(async move {
                        match #struct_name::read_from_url().await {
                            Ok(settings_model) => {
                                let mut write_access = settings.write().await;
                                *write_access = settings_model;
                            }
                            Err(err) => {
                                println!("Can not read settings from url. Err: {:?}", err);
                            }
                        }
                    });
                }
            }
        }
    }.into();

    quote::quote! {
        #[derive(Debug, Clone)]
        pub enum LoadSettingsError{
            FileError(String),
            YamlError(String)
        }

        pub enum FirstLoadResult {
            FromFile(#struct_name),
            FromUrl(#struct_name),
        }

        impl #struct_name {
            pub async fn first_load(file_name: &str) -> FirstLoadResult {
                match Self::read_from_file(file_name.to_string()).await {
                    Ok(settings) => return FirstLoadResult::FromFile(settings),
                    Err(err) => {
                        match err {
                            LoadSettingsError::FileError(err) => {
                                println!("Can not load settings from file. {:?}", err);

                            }
                            LoadSettingsError::YamlError(err) => {
                                panic!("{}", err);
                            }
                        }
            
                    }
                }
                let result = Self::read_from_url().await.unwrap();
                FirstLoadResult::FromUrl(result)
            }
            async fn read_from_file(file_name: String) -> Result<Self, LoadSettingsError> {
                let file_name = format!(#main_separator, std::env::var("HOME").unwrap(), file_name);
                let file_result = tokio::fs::File::open(file_name.as_str()).await;
                if file_result.is_err() {
                    return Err(LoadSettingsError::FileError(format!("Can not read settings from file: {}", file_name)));
                }
                let mut result = Vec::new();
                match tokio::io::AsyncReadExt::read_to_end(&mut file_result.unwrap(), &mut result).await {
                    Ok(_) => match serde_yaml::from_slice(&result) {
                        Ok(result) => Ok(result),
                        Err(err) => Err(LoadSettingsError::YamlError(format!(
                            "Invalid yaml format of file: {}. Err: {}",
                            file_name, err
                        ))),
                    },
                    Err(_) => Err(LoadSettingsError::FileError(format!("Can not read settings from file: {}", file_name))),
                }
            }
            async fn read_from_url() -> Result<Self, String> {
                let url = std::env::var("SETTINGS_URL");
                if url.is_err() {
                    return Err(format!("Environment variable SETTINGS_URL is not set"));
                }
                let url = url.unwrap();
                let result = flurl::FlUrl::new(url.as_str()).get().await;
        
                if let Err(err) = &result {
                    return Err(format!(
                        "Can not read settings from url: {}. Err: {:?}",
                        url, err
                    ));
                }
        
                let mut result = result.unwrap();
        
                let body = result.get_body().await;
        
                if let Err(err) = &body {
                    return Err(format!(
                        "Can not extract body from http request settings from url: {}. Err: {:?}",
                        url, err
                    ));
                }
        
                let body = body.unwrap();
        
                match serde_yaml::from_slice(body) {
                    Ok(result) => Ok(result),
                    Err(err) => Err(format!(
                        "Invalid yaml format of file: {}. Err: {}",
                        url, err
                    )),
                }
            }
        }

        #br
    }
    .into()
}
