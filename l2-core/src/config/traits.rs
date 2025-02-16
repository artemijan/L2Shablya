use serde::de::DeserializeOwned;
use std::any::type_name;
use std::fmt::Debug;
use std::path::PathBuf;
use std::{env, fs};

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
        let dir_entries = fs::read_dir(Self::DATA_DIR).unwrap_or_else(|e| {
            panic!(
                "Can't open {} to read config directory. {e}",
                Self::DATA_DIR
            )
        });
        let mut config = Self::default();
        for dir_entry in dir_entries {
            let path = dir_entry.expect("Can not read dir entry").path();
            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("yaml") {
                let file = fs::read_to_string(path).expect("Can not read file");
                let inst: Self::TargetConfigType =
                    serde_yaml::from_str(&file).expect("Can't read experience table");
                config.for_each(inst);
            }
        }
        config.post_load();
        config
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate as l2_core;
    use macro_common::config_file;
    use serde::Deserialize;

    #[derive(Debug, Clone, Deserialize)]
    #[config_file(path = "../test_data/test.yaml", msg = "Loaded")]
    struct TestConf {
        name: String,
    }

    #[test]
    fn test_config_file_loader() {
        let conf = <TestConf as ConfigFileLoader>::load();
        assert_eq!(conf.name, "test config");
    }
}
