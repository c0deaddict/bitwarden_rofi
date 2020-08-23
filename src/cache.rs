use crate::item::Item;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct Cache {
    path: PathBuf,
    items: Vec<Item>,
}

impl Cache {
    pub fn try_load(path: &Path) -> Cache {
        let items = match fs::read_to_string(path) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(items) => items,
                Err(err) => {
                    eprintln!("Could not deserialize cache: {}", err);
                    vec![]
                }
            },
            Err(err) => {
                eprintln!("Could not read cache: {}", err);
                vec![]
            }
        };

        Cache {
            items,
            path: path.to_owned(),
        }
    }

    pub fn replace(&mut self, items: Vec<Item>) {
        self.items = items;

        let mut file = match File::create(&self.path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Could not create cache file: {}", err);
                return;
            }
        };

        let contents = serde_json::to_string(&self.items).unwrap();
        match file.write_all(contents.as_bytes()) {
            Err(err) => eprintln!("Writing cache file failed: {}", err),
            Ok(_) => eprintln!("Cache updated"),
        }
    }

    pub fn items(&self) -> &Vec<Item> {
        &self.items
    }
}
