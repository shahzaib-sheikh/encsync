use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub sources: Vec<String>,
    pub version: String,
    pub destination: String,
}

pub fn get_config(path: impl AsRef<std::path::Path>) -> Option<Config> {
    let result = std::fs::File::open(path);

    if let Ok(mut file) = result {
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Could not read config file");

        let config: Config = toml::from_str(&contents).unwrap();
        Some(config)
    } else {
        None
    }
}

pub fn write_config(path: impl AsRef<std::path::Path>, config: &Config) {
    let mut file = std::fs::File::create(path).unwrap();

    let toml = toml::to_string(&config).unwrap();

    file.write_all(toml.as_bytes()).unwrap();
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn test_get_config_fail() {
        let config = get_config("test/config.toml");

        assert!(config.is_none());
    }

    #[test]
    fn test_get_config_success() {
        let tempfile = tempfile::NamedTempFile::new().unwrap();

        let mut file = std::fs::File::create(tempfile.path()).unwrap();

        file.write_all(
            r#"
            sources = ["test"]
            version = "0.1.0"
            destination = "test"
        "#
            .as_bytes(),
        )
        .unwrap();

        let config = get_config(tempfile.path());

        assert!(config.is_some());

        if let Some(c) = config {
            assert_eq!(c.version, "0.1.0");
            assert_eq!(c.destination, "test");
            assert_eq!(c.sources, vec!["test"]);
        }
    }

    #[test]
    fn test_write_config() {
        let tempfile = tempfile::NamedTempFile::new().unwrap();

        let config = Config {
            sources: vec!["test".to_string()],
            version: "0.1.0".to_string(),
            destination: "test".to_string(),
        };

        write_config(tempfile.path(), &config);

        let mut file = std::fs::File::open(tempfile.path()).unwrap();

        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        assert_eq!(contents, toml::to_string(&config).unwrap());
    }

    #[test]
    fn test_write_config_and_read() {
        let tempfile = tempfile::NamedTempFile::new().unwrap();

        let config = Config {
            sources: vec!["test".to_string()],
            version: "0.1.0".to_string(),
            destination: "test".to_string(),
        };

        write_config(tempfile.path(), &config);

        let config = get_config(tempfile.path());

        assert!(config.is_some());

        if let Some(c) = config {
            assert_eq!(c.version, "0.1.0");
            assert_eq!(c.destination, "test");
            assert_eq!(c.sources, vec!["test"]);
        }
    }
}
