use rust_extensions::StrOrString;

pub struct SettingsReader<T: Send + Sync + 'static>
where
    T: serde::de::DeserializeOwned,
{
    settings: tokio::sync::Mutex<Option<std::sync::Arc<T>>>,
    file_name: StrOrString<'static>,
}

impl<T: Send + Sync + 'static> SettingsReader<T>
where
    T: serde::de::DeserializeOwned,
{
    pub fn new(file_name: impl Into<StrOrString<'static>>) -> Self {
        Self {
            settings: tokio::sync::Mutex::new(None),
            file_name: file_name.into(),
        }
    }

    async fn read_settings_model(&self) -> std::sync::Arc<T> {
        let file_name = rust_extensions::file_utils::format_path(self.file_name.as_str());

        let content = tokio::fs::read_to_string(file_name.as_str()).await;

        if let Err(err) = &content {
            panic!(
                "Can not read settings file '{}'. Err:{}",
                file_name.as_str(),
                err
            );
        }

        let content = content.unwrap();

        let model: T = serde_yaml::from_str(content.as_str()).unwrap();

        std::sync::Arc::new(model)
    }

    pub async fn get_settings(&self) -> std::sync::Arc<T> {
        let mut settings_access = self.settings.lock().await;

        loop {
            if let Some(settings_access) = settings_access.clone() {
                return settings_access;
            }

            let model = self.read_settings_model().await;
            *settings_access = Some(model);
        }
    }

    pub async fn get<TResult>(&self, convert: impl Fn(&T) -> TResult) -> TResult {
        let mut settings_access = self.settings.lock().await;

        loop {
            if let Some(settings_access) = settings_access.as_ref() {
                return convert(settings_access);
            }

            let model = self.read_settings_model().await;
            *settings_access = Some(model);
        }
    }
}
