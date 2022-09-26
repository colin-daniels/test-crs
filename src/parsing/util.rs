use std::path::{Path, PathBuf};
use std::{fs, io};

#[macro_export]
macro_rules! enum_token {
    (pub enum $name:ident {
        $($variant:ident = $val:expr),*,
    }) => {
        #[derive(Debug, Copy, Clone)]
        pub enum $name {
            $($variant),*
        }

        impl $name {
            pub fn name(&self) -> &'static str {
                match self {
                    $($name::$variant => $val,)*
                }
            }

            pub fn from_name(s: &str) -> Option<Self> {
                match s {
                    $($val => Some($name::$variant),)*
                    _ => None,
                }
            }
        }
    };
}

#[macro_export]
macro_rules! replace_expr {
    ($_t:tt $sub:tt) => {
        $sub
    };
}

#[macro_export]
macro_rules! token_with_args {
    (pub enum $name:ident {
        $($variant:ident$(($typ:ty))? = $val:expr),*,
    }) => {
        #[derive(Debug, Clone)]
        pub enum $name {
            $($variant$(($typ))?),*
        }

        impl $name {
            pub fn name(&self) -> &'static str {
                match self {
                    $($name::$variant$(($crate::replace_expr!($typ _)))? => $val,)*
                }
            }

            pub fn from_arg(s: &str, a: Option<&str>) -> Option<Self> {
                match s {
                    $($val => Some($name::$variant$((<$typ>::from(a?)))?),)*
                    _ => None,
                }
            }
        }
    };
}

pub fn get_rule_configs<P: AsRef<Path>>(dir: P) -> io::Result<Vec<PathBuf>> {
    if dir.as_ref().is_dir() {
        let mut config_paths = vec![];
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let filename = entry.file_name().into_string().unwrap_or_default();

            // check if it looks like a config file
            let is_conf = path.is_file()
                && filename.ends_with(".conf")
                // skip install-specific exclusions
                && !filename.contains("EXCLUSION-RULES");

            if is_conf {
                config_paths.push(path);
            }
        }

        // return a sorted list due to the fact that CRS expects these to be loaded in
        // a certain order
        config_paths.sort();
        Ok(config_paths)
    } else {
        Ok(Default::default())
    }
}
