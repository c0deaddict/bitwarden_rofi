use crate::app::App;
use crate::item::{Action, Field, Item};
use crate::provider::Provider;
use anyhow::Result;
use serde::Deserialize;

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
pub struct Config {
    path: String,
    wrapper: Vec<String>,
}

pub struct Terraform {}

// TODO: somehow redirect stdin to rofi entrybox'es
// TODO: capture output and when stdin is asked (can we determine this?) show
// the captured output of the last line as prompt for the entrybox.

impl Terraform {
    pub fn new(app: &App, id: &str, config: serde_json::Value) -> Box<dyn Provider> {
        let config: Config = serde_json::from_value(config).unwrap();
        println!("terraform config = {:?}", config);
        Box::new(Terraform {})
    }
}

impl Provider for Terraform {
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
