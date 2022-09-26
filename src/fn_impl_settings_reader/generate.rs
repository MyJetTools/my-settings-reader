use proc_macro::TokenStream;

pub fn generate(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let struct_name = name.to_string();

    let mut result = String::new();

    result.push_str("impl ");
    result.push_str(struct_name.as_str());
    result.push_str(" {\n");

    result.push_str(
        r#"pub async fn load(file_name: &str) -> Self {
            match Self::read_from_file(file_name.to_string()).await {
                Ok(settings) => return settings,
                Err(_) => {
                    println!("Unable to read settings from file: {}.", file_name);
                }
            }
    
            Self::read_from_url().await
        }
    
    async fn read_from_file(file_name: String) -> Option<Self> {
        let file_name = format!("{}"#,
    );

    if std::path::MAIN_SEPARATOR == '/' {
        result.push(std::path::MAIN_SEPARATOR);
    } else {
        result.push(std::path::MAIN_SEPARATOR);
        result.push(std::path::MAIN_SEPARATOR);
    }

    result.push_str(
        r#"{}", std::env::var("HOME").unwrap(), file_name);

        let file_result = tokio::fs::File::open(file_name.as_str()).await; // Here
        if file_result.is_err() {
            return Err(format!("Can not read settings from file: {}", file_name));
        }
        let mut result = Vec::new();
        match tokio::io::AsyncReadExt::read_to_end(&mut file_result.unwrap(), &mut result).await {
            Ok(_) => match serde_yaml::from_slice(&result) {
                Ok(result) => Ok(result),
                Err(err) => Err(format!(
                    "Invalid yaml format of file: {}. Err: {}",
                    file_name, err
                )),
            },
            Err(_) => Err(format!("Can not read settings from file: {}", file_name)),
        }
    }
    
    async fn read_from_url() -> Self {
            let url = std::env::var("SETTINGS_URL");
            if url.is_err() {
                    panic!("Environmant variable SETTINGS_URL is not set");
            }
            let url = url.unwrap();
            let mut result = flurl::FlUrl::new(url.as_str(), None).get().await.unwrap();
            let body = result.get_body().await.unwrap();
            serde_yaml::from_slice(body).unwrap()
    }"#,
    );

    result.push_str("}\n");

    #[cfg(feature = "background-reader")]
    {
        result.push_str(
            r#"

    pub struct SettingsReader {
        settings: std::sync::Arc<tokio::sync::RwLock<"#,
        );

        result.push_str(struct_name.as_str());

        result.push_str(
            r#">>,
    }
    
    impl SettingsReader {
        pub async fn new(file_name: &str) -> Self {
            if let Ok(settings) = "#,
        );

        result.push_str(struct_name.as_str());

        result.push_str(
            r#"::read_from_file(file_name.to_string()).await {
                let settings = std::sync::Arc::new(tokio::sync::RwLock::new(settings));
                tokio::spawn(update_settings_in_a_background(
                    settings.clone(),
                    Some(file_name.to_string()),
                ));
                return Self { settings };
            }
            let settings = "#,
        );

        result.push_str(struct_name.as_str());

        result.push_str(
            r#"::read_from_url().await;
            let settings = std::sync::Arc::new(tokio::sync::RwLock::new(settings));
            tokio::spawn(update_settings_in_a_background(settings.clone(), None));
            Self { settings }
        }
        pub async fn get_settings(&self) -> "#,
        );

        result.push_str(struct_name.as_str());

        result.push_str(
            r#" {
            self.settings.read().await.clone()
        }
    }
    
    async fn update_settings_in_a_background(
        settings: std::sync::Arc<tokio::sync::RwLock<"#,
        );

        result.push_str(struct_name.as_str());

        result.push_str(
            r#">>,
        file_name: Option<String>,
    ) {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            let settings = settings.clone();
            let file_name = file_name.clone();
            let _ = tokio::spawn(async move {
                let result = if let Some(file_name) = &file_name {
                    let file_name = file_name.clone();
                    match tokio::spawn("#,
        );

        result.push_str(struct_name.as_str());

        result.push_str(
            r#"::read_from_file(file_name)).await {
                        Ok(result) => result,
                        Err(err) => Err(format!("Can not read settings from file. Err: {}", err)),
                    }
                } else {
                    match tokio::spawn("#,
        );

        result.push_str(struct_name.as_str());

        result.push_str(
            r#"::read_from_url()).await {
                        Ok(result) => Ok(result),
                        Err(err) => Err(format!("Can not read settings from url. Err: {}", err)),
                    }
                };
    
                if let Ok(settings_model) = result {
                    let mut write_access = settings.write().await;
                    *write_access = settings_model;
                }
            })
            .await;
        }
    }
    "#,
        );
    }
    result.parse().unwrap()
}
