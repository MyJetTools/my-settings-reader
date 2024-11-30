use std::sync::Arc;

use flurl::FlUrl;
use rust_extensions::StrOrString;

pub struct SettingsReaderInner<T: Send + Sync + 'static>
where
    T: serde::de::DeserializeOwned,
{
    instance: tokio::sync::Mutex<Option<std::sync::Arc<T>>>,
    file_name: StrOrString<'static>,
}

impl<T: Send + Sync + 'static> SettingsReaderInner<T>
where
    T: serde::de::DeserializeOwned,
{
    pub fn new(file_name: StrOrString<'static>) -> Self {
        Self {
            instance: tokio::sync::Mutex::new(None),
            file_name,
        }
    }
    async fn try_read_from_url(&self) -> Option<T> {
        let url = std::env::var("SETTINGS_URL");
        if url.is_err() {
            return None;
        }

        let url = url.unwrap();
        let response = match FlUrl::new(url.as_str()).get().await {
            Ok(response) => response,
            Err(err) => {
                println!("Can not read settings from url {}. Err: {:?}", url, err);
                return None;
            }
        };

        let content = response.receive_body().await;

        let content = match content {
            Ok(content) => content,
            Err(err) => {
                println!("Can not get settings body from url {}. Err: {:?}", url, err);
                return None;
            }
        };

        match serde_yaml::from_slice(content.as_slice()) {
            Ok(settings) => Some(settings),
            Err(err) => {
                println!(
                    "Can not deserialize settings from url {}. Err: {:?}",
                    url, err
                );
                None
            }
        }
    }

    async fn read_settings_model(&self) -> Result<std::sync::Arc<T>, String> {
        if let Some(result) = self.try_read_from_url().await {
            return Ok(std::sync::Arc::new(result));
        }

        let file_name = rust_extensions::file_utils::format_path(self.file_name.as_str());

        let content = tokio::fs::read_to_string(file_name.as_str()).await;

        let content = match content {
            Ok(content) => content,
            Err(err) => {
                return Err(format!(
                    "Can not read settings file '{}'. Err:{}",
                    file_name.as_str(),
                    err
                ));
            }
        };

        let model: T = serde_yaml::from_str(content.as_str()).unwrap();

        Ok(std::sync::Arc::new(model))
    }
}

pub struct SettingsReader<T: Send + Sync + 'static>
where
    T: serde::de::DeserializeOwned,
{
    inner: Arc<SettingsReaderInner<T>>,
}

impl<T: Send + Sync + 'static> SettingsReader<T>
where
    T: serde::de::DeserializeOwned,
{
    pub fn new(file_name: impl Into<StrOrString<'static>>) -> Self {
        let result = Self {
            inner: SettingsReaderInner::new(file_name.into()).into(),
        };

        tokio::spawn(background_refresh(result.inner.clone()));

        result
    }

    pub fn new_without_background_refresh(file_name: impl Into<StrOrString<'static>>) -> Self {
        Self {
            inner: SettingsReaderInner::new(file_name.into()).into(),
        }
    }

    pub async fn get<TResult>(&self, convert: impl Fn(&T) -> TResult) -> TResult {
        let mut settings_access = self.inner.instance.lock().await;

        loop {
            if let Some(settings_access) = settings_access.as_ref() {
                return convert(settings_access);
            }

            let model = self.inner.read_settings_model().await.unwrap();
            *settings_access = Some(model);
        }
    }
}

async fn background_refresh<T: Send + Sync + 'static>(inner: Arc<SettingsReaderInner<T>>)
where
    T: serde::de::DeserializeOwned,
{
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;

        match inner.read_settings_model().await {
            Ok(settings_model) => {
                let mut write_access = inner.instance.lock().await;
                *write_access = Some(settings_model);
            }
            Err(err) => {
                println!("Error reading settings from file. Err: {:?}", err);
            }
        }
    }
}
