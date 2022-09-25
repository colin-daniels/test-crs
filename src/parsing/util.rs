use std::path::{Path, PathBuf};
use std::{fs, io};

pub fn get_rule_configs<P: AsRef<Path>>(dir: P) -> io::Result<Vec<PathBuf>> {
    if dir.as_ref().is_dir() {
        let mut config_paths = vec![];
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            // check if it looks like a config file
            let is_conf = path.is_file()
                && entry
                    .file_name()
                    .into_string()
                    .unwrap_or_default()
                    .ends_with(".conf");

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
