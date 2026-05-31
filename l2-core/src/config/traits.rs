use serde::de::DeserializeOwned;
use std::any::type_name;
use std::fmt::Debug;
use std::path::PathBuf;
use std::{env, fs};
use walkdir::WalkDir;

pub trait Loadable: Sized + Debug + Clone {
    fn load() {}
    fn post_load(&self) {}
}

pub trait ConfigFileLoader: DeserializeOwned + Loadable {
    const DATA_FILE: &'static str;
    #[must_use]
    fn load() -> Self {
        // Get config path from L2_CONFIG env variable or use default "./"
        let config_base = env::var("L2_CONFIG").unwrap_or_else(|_| "./".to_string());
        let config_path = PathBuf::from(config_base).join(Self::DATA_FILE);

        let file_content = fs::read_to_string(&config_path).unwrap_or_else(|e| {
            panic!(
                "Can't read config file {} (resolved path: {}), because of: {e}",
                Self::DATA_FILE,
                config_path.display()
            )
        });

        let inst: Self = serde_yaml::from_str(&file_content).unwrap_or_else(|e| {
            panic!(
                "Can't parse {} (resolved path: {}), because of: {e}",
                type_name::<Self>(),
                config_path.display()
            )
        });

        inst.post_load();
        inst
    }
}

pub trait LoadFileHandler {
    type TargetConfigType: DeserializeOwned + Debug;
    fn for_each(&mut self, item: Self::TargetConfigType);
}

pub trait ConfigDirLoader: Loadable + Default + LoadFileHandler {
    const DATA_DIR: &'static str;
    #[must_use]
    fn load() -> Self {
        // Get config path from L2_CONFIG env variable or use default "./"
        let config_base = env::var("L2_CONFIG").unwrap_or_else(|_| "./".to_string());
        let config_dir = PathBuf::from(config_base).join(Self::DATA_DIR);

        let mut config = Self::default();

        for entry in WalkDir::new(config_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                e.path().is_file()
                    && e.path().extension().and_then(|ext| ext.to_str()) == Some("yaml")
            })
        {
            let path = entry.path();
            let file = fs::read_to_string(path).unwrap_or_else(|e| {
                panic!("Cannot read file {}, because of: {e}", path.display())
            });
            let inst: Self::TargetConfigType = serde_yaml::from_str(&file).unwrap_or_else(|e| {
                panic!("Can't parse YAML file {}, because of: {e}", path.display())
            });
            config.for_each(inst);
        }

        config.post_load();
        config
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[allow(unused_imports)]
    use crate as l2_core;
    //hack to test l2_core crate
    use macro_common::config_file;
    use serde::Deserialize;

    #[derive(Debug, Clone, Deserialize)]
    #[config_file(path = "test_data/test.yaml", msg = "Loaded")]
    struct TestConf {
        name: String,
    }
    #[derive(Debug, Clone, Deserialize)]
    #[config_file(path = "config/game.yaml", msg = "Loaded")]
    struct TestConf2 {
        name: String,
    }

    #[test]
    fn test_config_file_loader() {
        let conf = <TestConf as ConfigFileLoader>::load();
        assert_eq!(conf.name, "test config");
    }
    #[test]
    fn test_config_file_load() {
        let conf = <TestConf2 as ConfigFileLoader>::load();
        assert_eq!(conf.name, "Game server");
    }
}
