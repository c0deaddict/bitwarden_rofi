use bitwarden_rofi::bitwarden::Session;
use bitwarden_rofi::cache::Cache;
use keyring::Keyring;
use rustofi::components::EntryBox;
use std::process;

fn main() -> std::io::Result<()> {
    let keyring = Keyring::new("bitwarden_rofi", "BW_SESSION");

    let session = match keyring.get_password() {
        Ok(key) => {
            let session = Session::open(&key);
            if session.is_unlocked()? {
                Some(session)
            } else {
                eprintln!("Session key is not valid");
                None
            }
        }
        Err(_) => None,
    };

    let session = session.unwrap_or_else(|| {
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
    });

    let mut cache = Cache::try_load("/home/jos/.local/share/bitwarden_rofi/cache.json");

    cache.replace(vec![]);

    // read all items and folders
    // cache name/folder structure on disk
    // show menu:
    // - list all items and folders
    // - when one is chosen; show options for getting:
    //   * username
    //   * password
    //   * otp
    //   * more?
    // - sync (and update cache)
    // - lock

    // println!("{}", session.key);

    // session.lock().unwrap_or_else(|err| {
    //     eprintln!("Error locking session: {}", err);
    // });

    println!("{:?}", session.status());

    // println!("{:?}", session.list_items());

    Ok(())
}
