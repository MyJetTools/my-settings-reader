use proc_macro::TokenStream;

pub fn generate(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let struct_name = name.to_string();

    let mut result = String::new();

    result.push_str("impl ");
    result.push_str(struct_name.as_str());
    result.push_str(" {\n");

    let implementation = r#"pub async fn load(file_name: &str) -> Self {{
        if let Some(result) = Self::read_from_file(file_name) {{
            return result;
        }}
    
        Self::read_from_url().await
    }}
    
    fn read_from_file(file_name: &str) -> Option<Self> {{
        let home = format!("{}/{}", std::env::var("HOME").unwrap(), file_name);
    
        let mut file_result = std::fs::File::open(home);
    
        if file_result.is_err() {{
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
    
        let mut result = flurl::FlUrl::new(url.as_str()).get().await.unwrap();
    
        let body = result.get_body().await.unwrap();
    
        serde_yaml::from_slice(body).unwrap()
    }}
    "#;

    result.push_str(implementation);

    result.push_str("}\n");

    result.parse().unwrap()
}
