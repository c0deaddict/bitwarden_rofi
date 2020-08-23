use crate::cache::Cache;
use crate::providers::bitwarden::BitwardenProvider;
use crate::providers::Provider;
use crate::rofi::{RofiResponse, RofiWindow};
use anyhow::Result;
use xdg;

pub struct App {
    // config file with providers
}

impl App {
    pub fn new() -> App {
        App {}
    }

    pub fn show(&self) -> Result<()> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("bitwarden_rofi")?;
        let cache_file = xdg_dirs.place_cache_file("cache.json")?;
        eprintln!("Cache file is {:?}", cache_file);
        let mut cache = Cache::try_load(&cache_file);

        let mut bw = BitwardenProvider::new();
        cache.replace(bw.get_items()?);

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
        for i in cache.items().iter() {
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
}
