pub const IMPLEMENTATION: &str = r#"""
pub async fn load() -> Self {
    if let Some(result) = Self::read_from_file() {
        return result;
    }

    Self::read_from_url().await
}

fn read_from_file() -> Option<Self> {
    let home = format!("{}/.reachpay", std::env::var("HOME").unwrap());

    let mut file_result = std::fs::File::open(home);

    if file_result.is_err() {
        return None;
    }

    let mut result = Vec::new();
    file_result.unwrap().read_to_end(&mut result).unwrap();
    Some(serde_yaml::from_slice(&result).unwrap())
}

async fn read_from_url() -> Self {
    let url = std::env::var("SETTINGS_URL");

    if url.is_err() {
        panic!("Environmant variable SETTINGS_URL is not set");
    }

    let url = url.unwrap();

    let mut result = FlUrl::new(url.as_str()).get().await.unwrap();

    let body = result.get_body().await.unwrap();

    serde_yaml::from_slice(body).unwrap()
}
"""#;
