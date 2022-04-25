use proc_macro::TokenStream;

pub fn generate(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let struct_name = name.to_string();

    let mut result = String::new();

    result.push_str("impl ");
    result.push_str(struct_name.as_str());
    result.push_str(" {\n");

    result.push_str("pub async fn read(&self) -> Self {");

    result.push_str(super::fn_settings::IMPLEMENTATION);

    result.push_str("}\n");

    result.push_str("}\n");

    result.parse().unwrap()
}
