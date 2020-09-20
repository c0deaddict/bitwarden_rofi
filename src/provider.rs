use crate::item::{Action, Field, Item};
use anyhow::Result;

pub type NewProvider = dyn Fn(&str, serde_json::Value) -> Box<dyn Provider>;

pub trait Provider {
    fn list_items(&mut self) -> Result<Vec<Item>>;
    fn read_field(&mut self, item: &Item, field: &Field) -> Result<String>;
    fn list_actions(&mut self) -> Result<Vec<Action>>;
    fn do_action(&mut self, action: &Action);
}
