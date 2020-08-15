use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Write;

#[serde(rename_all = "camelCase")]
#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    name: String,
    path: Vec<String>,
    organization: Option<String>,
    has_username: bool,
    has_password: bool,
    has_totp: bool,
}

pub struct Cache {
    filename: String,
    items: Vec<Item>,
}

impl Cache {
    pub fn try_load(filename: &str) -> Cache {
        let items = match fs::read_to_string(filename) {
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
            filename: filename.to_string(),
        }
    }

    pub fn replace(&mut self, items: Vec<Item>) {
        self.items = items;

        let mut file = match File::create(&self.filename) {
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
}
