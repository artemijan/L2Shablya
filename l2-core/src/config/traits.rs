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
        let mut full_path = PathBuf::from(Self::DATA_FILE);
        let mut visited_paths = Vec::new(); // Track visited symlinks to prevent loops

        // Resolve symlinks manually to avoid infinite loops
        while full_path.is_symlink() {
            assert!(
                !visited_paths.contains(&full_path),
                "Symlink loop detected: {full_path:?}"
            );
            visited_paths.push(full_path.clone());

            full_path = fs::read_link(&full_path).unwrap_or_else(|e| {
                panic!("Failed to resolve symlink {}: {e}", full_path.display())
            });
        }

        // Ensure we get an absolute path
        full_path = full_path
            .canonicalize()
            .unwrap_or_else(|_| env::current_dir().unwrap().join(Self::DATA_FILE));
        let file_content = fs::read_to_string(&full_path).unwrap_or_else(|e| {
            panic!(
                "Can't read config file {} (resolved path: {}), because of: {e}",
                Self::DATA_FILE,
                full_path.display()
            )
        });

        let config_name = type_name::<Self>();
        let inst: Self = serde_yaml::from_str(&file_content)
            .unwrap_or_else(|e| panic!("Can't parse {config_name}, because of: {e}"));

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
        let mut config = Self::default();

        for entry in WalkDir::new(Self::DATA_DIR)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().is_file() && e.path().extension().and_then(|ext| ext.to_str()) == Some("yaml"))
        {
            let path = entry.path();
            let file = fs::read_to_string(path).expect("Cannot read file");
            let inst: Self::TargetConfigType =
                serde_yaml::from_str(&file).expect("Can't parse YAML file");
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
    use crate as l2_core; //hack to test l2_core crate
    use macro_common::config_file;
    use serde::Deserialize;

    #[derive(Debug, Clone, Deserialize)]
    #[config_file(path = "../test_data/test.yaml", msg = "Loaded")]
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
