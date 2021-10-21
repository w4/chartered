use serde::Serialize;

/// The `config.json` file to write to the repository.
#[derive(Serialize, Debug, Clone)]
pub struct CargoConfig {
    pub dl: String,
    pub api: String,
}

impl CargoConfig {
    pub fn new(base: &url::Url, api_key: &str, organisation: &str) -> Self {
        let base = format!("{}a/{}/o/{}", base, api_key, organisation);

        Self {
            dl: format!("{}/api/v1/crates", base),
            api: base,
        }
    }
}

#[cfg(test)]
mod test {
    use super::CargoConfig;

    #[test]
    fn test_cargo_config() {
        let conf = CargoConfig::new(
            &url::Url::parse("https://127.0.0.1:1234").unwrap(),
            "my-api-key",
            "my-organisation",
        );
        assert_eq!(
            conf.dl,
            "https://127.0.0.1:1234/a/my-api-key/o/my-organisation/api/v1/crates"
        );
        assert_eq!(
            conf.api,
            "https://127.0.0.1:1234/a/my-api-key/o/my-organisation"
        );
    }
}
