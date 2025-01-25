use serde::de::DeserializeOwned;
use std::any::type_name;
use std::fmt::Debug;
use std::fs;

pub trait Loadable: Sized + Debug + Clone {
    fn load(){}
    fn post_load(&self) {}
}

pub trait ConfigFileLoader: DeserializeOwned + Loadable {
    const DATA_FILE: &'static str;
    #[must_use]
    fn load() -> Self {
        let file_content = fs::read_to_string(Self::DATA_FILE).unwrap_or_else(|e| {
            panic!(
                "Can't read config file {}, because of: {e}",
                Self::DATA_FILE
            )
        });
        let config_name = type_name::<Self>();
        let inst: Self = serde_yaml::from_str(&file_content)
            .unwrap_or_else(|e| panic!("Can't read {config_name}, because of: {e}"));
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
