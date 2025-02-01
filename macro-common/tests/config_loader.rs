#[cfg(test)]
pub mod l2_core {
    pub mod config {
        pub mod traits {
            pub trait Loadable {}
            pub trait ConfigFileLoader: Loadable {
                const DATA_FILE: &'static str;
                fn load(&self) {}
            }
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::l2_core::config::traits::ConfigFileLoader;
    use macro_common::config_file;

    #[config_file(path = "different_path")]
    struct TestLoader {
        h: i32,
    }
    impl TestLoader {
        fn test(&self) -> i32 {
            self.h
        }
    }

    #[config_file(path = "enum_path")]
    enum ConfigLoader {
        One,
        Two,
    }

    #[test]
    fn case_struct() {
        let k = TestLoader { h: 1 };
        k.load();
        assert_eq!(k.test(), 1);
        assert_eq!(TestLoader::DATA_FILE, "different_path");
    }
    #[test]
    fn case_enum() {
        let k = ConfigLoader::One;
        let l = ConfigLoader::Two;
        k.load();
        l.load();
    }
}
