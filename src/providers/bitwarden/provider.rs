use super::session::{Error as SessionError, Session};
use crate::item::{Field, Item};
use crate::providers::Provider;
use crate::rofi::RofiWindow;
use anyhow::Result;
use keyring::Keyring;
use std::collections::HashMap;

// TODO: how to pass config??
pub struct BitwardenProvider {
    session: Option<Session>,
}

impl BitwardenProvider {
    pub fn new() -> Self {
        Self { session: None }
    }

    fn get_session(&mut self) -> Result<&Session> {
        if self.session.is_some() {
            Ok(self.session.as_ref().unwrap())
        } else {
            self.open_session()
        }
    }

    fn open_session(&mut self) -> Result<&Session> {
        let keyring = Keyring::new("bitwarden_rofi", "BW_SESSION");

        let session = match keyring.get_password() {
            Ok(key) => {
                let session = Session::open(&key);
                match session.is_unlocked() {
                    Ok(true) => Some(session),
                    Ok(false) => {
                        eprintln!("bitwarden: Session key is not valid");
                        None
                    }
                    Err(err) => match err.downcast_ref::<SessionError>() {
                        Some(SessionError::FailedToDecrypt) => {
                            eprintln!("bitwarden: Failed to decrypt");
                            None
                        }
                        _ => return Err(err),
                    },
                }
            }
            Err(_) => None,
        };

        session
            .map(Ok)
            .unwrap_or_else(|| {
                let password = RofiWindow::new("Enter master password")
                    .add_args(vec!["-dmenu"])
                    .password(true)
                    .lines(0)
                    .show(vec![])?
                    .entry()?;

                let session = Session::unlock(&password)?;

                keyring.set_password(&session.key).unwrap_or_else(|err| {
                    eprintln!("bitwarden: Failed to put session key in keyring: {}", err);
                });

                Ok(session)
            })
            .map(move |session| {
                self.session = Some(session);
                self.session.as_ref().unwrap()
            })
    }
}

impl Provider for BitwardenProvider {
    fn is_blocking() -> bool {
        true
    }

    fn get_items(&mut self) -> Result<Vec<Item>> {
        let mut folders = HashMap::new();

        let session = self.get_session()?;

        for f in session.list_folders()?.into_iter() {
            folders.insert(f.id.clone(), f);
        }

        let mut items: Vec<Item> = vec![];

        for i in session.list_items()?.into_iter() {
            let mut path = match folders.get(&i.folder_id) {
                None => vec![],
                _ if i.folder_id.is_none() => vec![],
                Some(folder) => folder.name.split("/").map(|s| s.to_string()).collect(),
            };

            path.push(i.name);
            let title = path.join("/");

            let mut fields = vec![];
            if let Some(login) = i.login {
                if login.username.is_some() {
                    fields.push(Field::Username);
                }
                if login.password.is_some() {
                    fields.push(Field::Password);
                }
                if login.totp.is_some() {
                    fields.push(Field::Totp);
                }
            }

            let item = Item {
                id: i.id,
                title,
                fields,
            };

            items.push(item);
        }

        Ok(items)
    }

    fn read_field(&mut self, id: &str, field: &Field) -> Result<String> {
        let field_name = match field {
            Field::Username => "username",
            Field::Password => "password",
            Field::Totp => "totp",
            Field::Other(name) => &name,
        };
        let session = self.get_session()?;
        session.read_field(id, field_name)
    }

    fn lock(&mut self) -> Result<()> {
        // TODO: this can open a session if it wasn't...
        self.get_session()?.lock()?;
        let keyring = Keyring::new("bitwarden_rofi", "BW_SESSION");
        keyring.delete_password().unwrap_or_else(|err| {
            eprintln!("Deleting entry from keyring failed: {}", err);
        });
        Ok(())
    }

    fn sync(&mut self) -> Result<()> {
        self.get_session()?.sync()
    }
}
