use anyhow::Result;
use bitwarden_rofi::bitwarden::{self, Session};
use bitwarden_rofi::cache::{self, Cache};
use keyring::Keyring;
use rustofi::components::EntryBox;
use rustofi::window::*;
use std::collections::HashMap;
use std::process;

// https://github.com/mattydebie/bitwarden-rofi/blob/master/bwmenu

fn main() -> Result<()> {
    let mut cache = Cache::try_load("/home/jos/.local/share/bitwarden_rofi/cache.json");

    let session = load_session()?;
    cache.replace(get_items(&session)?);

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
        let mut title = i.path.join("/");
        if title.len() != 0 {
            title += "/";
        };
        title += &i.name;
        entries.push(title);
    }

    entries.sort();

    let res = Window::new("Select an entry")
        .format('s')
        .location(Location::MiddleCentre)
        .add_args(
            vec![
                "-dmenu",
                "-markup-rows",
                "-matching",
                "fuzzy",
                "-kb-custom-1",
                "Alt+r",
                "-kb-custom-2",
                "Alt+l",
                "-mesg",
                "<b>Alt+r</b>: sync | <b>Alt+l</b>: lock",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        )
        .lines(15)
        .show(entries.clone())
        .expect("Creating rofi window failed");

    if res == "" {
        println!("You chose cancel. Bye then.");
        return Ok(());
    }

    let idx = entries.iter().position(|x| x == &res);

    println!("You chose entry: {:?}", idx);

    Ok(())
}

fn load_session() -> Result<Session> {
    let keyring = Keyring::new("bitwarden_rofi", "BW_SESSION");

    let session = match keyring.get_password() {
        Ok(key) => {
            let session = Session::open(&key);
            match session.is_unlocked() {
                Ok(true) => Some(session),
                Ok(false) => {
                    eprintln!("Session key is not valid");
                    None
                }
                Err(err) => match err.downcast_ref::<bitwarden::Error>() {
                    Some(bitwarden::Error::FailedToDecrypt) => {
                        eprintln!("Failed to decrypt");
                        None
                    }
                    _ => return Err(err),
                },
            }
        }
        Err(_) => None,
    };

    Ok(session.unwrap_or_else(|| {
        let password = EntryBox::create_window()
            .add_args(vec!["-password".to_string()])
            .prompt("Enter master password".to_string())
            .show(vec![])
            .expect("password entry failed");

        let session = Session::unlock(&password).unwrap_or_else(|err| {
            eprintln!("Problem unlocking session: {}", err);
            process::exit(1)
        });

        keyring.set_password(&session.key).unwrap_or_else(|err| {
            eprintln!("Failed to put session key in keyring: {}", err);
        });

        session
    }))
}

fn get_items(session: &Session) -> Result<Vec<cache::Item>> {
    let mut folders = HashMap::new();

    for f in session.list_folders()?.into_iter() {
        folders.insert(f.id.clone(), f);
    }

    let mut items: Vec<cache::Item> = vec![];

    for i in session.list_items()?.into_iter() {
        let path = match folders.get(&i.folder_id) {
            None => vec![],
            _ if i.folder_id.is_none() => vec![],
            Some(folder) => folder.name.split("/").map(|s| s.to_string()).collect(),
        };

        let login = i.login.as_ref();

        let item = cache::Item {
            name: i.name,
            path: path,
            organization: None,
            has_username: login.map(|l| l.username.is_some()).unwrap_or(false),
            has_password: login.map(|l| l.password.is_some()).unwrap_or(false),
            has_totp: login.map(|l| l.totp.is_some()).unwrap_or(false),
        };
        items.push(item);
    }

    Ok(items)
}
