use crate::cache::Cache;
use crate::provider::{NewProvider, Provider};
use crate::providers::bitwarden::Bitwarden;
use crate::providers::keyhub::Keyhub;
use crate::providers::password_store::PasswordStore;
use crate::providers::terraform::Terraform;
use crate::rofi::{RofiResponse, RofiWindow};
use anyhow::Result;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use xdg;

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
struct ProviderConfig {
    #[serde(rename = "type")]
    type_: String,
    shortcut: Option<String>,
    config: serde_json::Value,
}

// TODO: shared rofi args
#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
struct Config {
    providers: HashMap<String, ProviderConfig>,
}

pub struct App {
    config: Config,
    providers: HashMap<String, RefCell<Box<dyn Provider>>>,
    xdg_dirs: xdg::BaseDirectories,
}

impl App {
    pub fn new() -> Result<App> {
        let mut providers: HashMap<String, Box<NewProvider>> = HashMap::new();
        providers.insert("bitwarden".to_owned(), Box::new(Bitwarden::new));
        providers.insert("password_store".to_owned(), Box::new(PasswordStore::new));
        providers.insert("terraform".to_owned(), Box::new(Terraform::new));
        providers.insert("keyhub".to_owned(), Box::new(Keyhub::new));

        let xdg_dirs = xdg::BaseDirectories::with_prefix("bitwarden_rofi")?;

        let config_file = xdg_dirs.find_config_file("config.json").unwrap();
        let contents = fs::read_to_string(config_file)?;
        let config: Config = serde_json::from_str(&contents)?;

        let providers = config
            .providers
            .iter()
            .map(|(key, provider)| match providers.get(&provider.type_) {
                Some(new_fn) => (
                    key.to_owned(),
                    RefCell::new(new_fn(key, provider.config.to_owned())),
                ),
                None => panic!("Provider {} does not exist", key),
            })
            .collect();

        Ok(App {
            config,
            providers,
            xdg_dirs,
        })
    }

    pub fn show(&self) -> Result<()> {
        // TODO: deterministic sort order
        let (key, provider) = self.providers.iter().next().unwrap();

        eprintln!("First provider = {}", key);

        // TODO: provider needs to know:
        // - the shortcuts to the rest
        // - xdg dirs (with a custom prefix?)

        // TODO: notify-send when doing a long op, (eg. bw first time initialize)

        // show menu:
        // - list all items and folders
        // - when one is chosen; show options for getting:
        //   * username
        //   * password
        //   * otp
        //   * more?
        // - sync (and update cache)
        // - lock

        let mut entries: Vec<String> = vec![];
        for i in provider.borrow_mut().list_items()?.iter() {
            entries.push(i.title.clone());
        }

        entries.sort();

        // TODO: let rofi return an integer of the index selected.
        let res = RofiWindow::new("Select an entry")
            .matching("fuzzy")
            .kb_custom(1, "Alt+r")
            .kb_custom(2, "Alt+l")
            .message("<b>Alt+r</b>: sync | <b>Alt+l</b>: lock")
            .add_args(vec!["-dmenu", "-markup-rows"])
            .lines(15)
            .show(entries.clone())
            .expect("Creating rofi window failed");

        match res {
            RofiResponse::Entry(s) => {
                println!("Entry = {}", s);
                let idx = entries.iter().position(|x| x == &s);
                println!("You chose entry: {:?}", idx);
            }
            RofiResponse::Cancel => println!("Bye."),
            RofiResponse::CustomKey(key) => println!("Custom key {}", key),
        }

        Ok(())
    }

    pub fn get_cache_file(&self, name: &str) -> io::Result<PathBuf> {
        self.xdg_dirs.place_cache_file(name)
    }
}
