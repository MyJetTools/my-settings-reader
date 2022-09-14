use proc_macro::TokenStream;

pub fn generate(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let struct_name = name.to_string();

    let mut result = String::new();

    result.push_str("impl ");
    result.push_str(struct_name.as_str());
    result.push_str(" {\n");
    result.push_str(
        r#"pub async fn load(file_name: &str) -> Self {{
        if let Some(result) = Self::read_from_file(file_name) {{
            return result;
        }}
    
        Self::read_from_url().await
    }}
    
    fn read_from_file(file_name: &str) -> Option<Self> {{
        let home = format!("{}/{}", std::env::var("HOME").unwrap(), file_name);


    
        let mut file_result = std::fs::File::open(home.as_str());
    
        if file_result.is_err() {{
            println!("Can not read settings from file: {}", home);
            return None;
        }}
    
        let mut result = Vec::new();
        std::io::Read::read_to_end(&mut file_result.unwrap(), &mut result).unwrap();
        Some(serde_yaml::from_slice(&result).unwrap())
    }}
    
    async fn read_from_url() -> Self {{
        let url = std::env::var("SETTINGS_URL");
    
        if url.is_err() {{
            panic!("Environmant variable SETTINGS_URL is not set");
        }}
    
        let url = url.unwrap();
    
        let mut result = flurl::FlUrl::new(url.as_str(), None).get().await.unwrap();
    
        let body = result.get_body().await.unwrap();
    
        serde_yaml::from_slice(body).unwrap()
    }}"#,
    );

    result.push_str("}\n");

    #[cfg(feature = "background-reader")]
    {
        result.push_str(
            r#"

    pub struct SettingsReader {
        settings: Arc<RwLock<"#,
        );

        result.push_str(struct_name.as_str());

        result.push_str(
            r#">>,
    }
    
    impl SettingsReader {
        pub async fn new(file_name: &str) -> Self {
            let settings = "#,
        );

        result.push_str(struct_name.as_str());

        result.push_str(
            r#"::load(file_name).await;
    
            let settings = Arc::new(RwLock::new(settings));
    
            tokio::spawn(update_settings_in_a_background(settings.clone(),file_name.to_string(),));
    
            Self { settings }
        }
    
        pub async fn get_settings(&self) -> "#,
        );
        result.push_str(struct_name.as_str());
        result.push_str(
            r#"{
            self.settings.read().await.clone()
        }
    }
    
    async fn update_settings_in_a_background(settings: Arc<RwLock<"#,
        );
        result.push_str(struct_name.as_str());
        result.push_str(
            r#">>, file_name: String) {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    
            let settings = settings.clone();
            let file_name = file_name.clone();
            let _ = tokio::spawn(async move {
                let settings_model = "#,
        );
        result.push_str(struct_name.as_str());
        result.push_str(
            r#"::load(file_name.as_str()).await;
                let mut write_access = settings.write().await;
                *write_access = settings_model;
            })
            .await;
        }
    }
    "#,
        );
    }
    result.parse().unwrap()
}
