use crate::app::App;
use crate::item::{Action, Field, Item};
use crate::provider::Provider;
use anyhow::Result;
use serde::Deserialize;

// TODO: for password-store provider: fields are NOT known before hand.
//       all files need to be decrypted to discover the fields, and that takes too long.
//       change cache to optionally store fields?

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
pub struct Config {
    path: String,
}

pub struct PasswordStore {}

impl PasswordStore {
    pub fn new(app: &App, id: &str, config: serde_json::Value) -> Box<dyn Provider> {
        let config: Config = serde_json::from_value(config).unwrap();
        println!("pass config = {:?}", config);
        Box::new(PasswordStore {})
    }
}

impl Provider for PasswordStore {
    fn list_items(&mut self) -> Result<Vec<Item>> {
        Ok(vec![])
    }

    fn read_field(&mut self, item: &Item, field: &Field) -> Result<String> {
        Ok("".to_owned())
    }

    fn list_actions(&mut self) -> Result<Vec<Action>> {
        Ok(vec![])
    }

    fn do_action(&mut self, action: &Action) {}
}
