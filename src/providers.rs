use crate::item::{Field, Item};
use anyhow::Result;

// TODO: add custom actions (like: sync/lock for bitwarden).
// TODO: for password-store provider: fields are NOT known before hand.
//       all files need to be decrypted to discover the fields, and that takes too long.
//       change cache to optionally store fields?

pub trait Provider {
    fn is_blocking() -> bool;
    fn lock(&mut self) -> Result<()>;
    fn sync(&mut self) -> Result<()>;
    fn get_items(&mut self) -> Result<Vec<Item>>;
    // fn get_fields(&mut self, id: &str) -> Result<Vec<Field>>;
    fn read_field(&mut self, id: &str, field: &Field) -> Result<String>;
}

pub mod bitwarden;
