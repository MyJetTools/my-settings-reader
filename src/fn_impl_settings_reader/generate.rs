use proc_macro::TokenStream;

pub fn generate(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let struct_name = name.to_string();

    let mut result = String::new();

    result.push_str("impl ");
    result.push_str(struct_name.as_str());
    result.push_str(" {\n");

    let file_name = ".text";

    let implementation = format!(
        r#"pub async fn load() -> Self {{
        if let Some(result) = Self::read_from_file() {{
            return result;
        }}
    
        Self::read_from_url().await
    }}
    
    fn read_from_file() -> Option<Self> {{
        let home = format!("{{}}/{file_name}", std::env::var("HOME").unwrap());
    
        let mut file_result = std::fs::File::open(home);
    
        if file_result.is_err() {{
            return None;
        }}
    
        let mut result = Vec::new();
        file_result.unwrap().read_to_end(&mut result).unwrap();
        Some(serde_yaml::from_slice(&result).unwrap())
    }}
    
    async fn read_from_url() -> Self {{
        let url = std::env::var("SETTINGS_URL");
    
        if url.is_err() {{
            panic!("Environmant variable SETTINGS_URL is not set");
        }}
    
        let url = url.unwrap();
    
        let mut result = FlUrl::new(url.as_str()).get().await.unwrap();
    
        let body = result.get_body().await.unwrap();
    
        serde_yaml::from_slice(body).unwrap()
    }}
    "#,
        file_name = file_name
    );

    result.push_str(implementation.as_str());

    result.push_str("}\n");

    result.parse().unwrap()
}
