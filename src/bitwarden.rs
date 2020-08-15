use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::str;

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
pub struct Status {
    server_url: String,
    last_sync: DateTime<Utc>,
    user_email: String,
    user_id: String,
    status: String,
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
pub struct Folder {
    object: String,
    id: Option<String>, // None is the root folder.
    name: String,
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
pub struct Uri {
    uri: String,
    #[serde(rename = "match")]
    match_: Option<String>,
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
pub struct Login {
    username: Option<String>,
    password: Option<String>,
    totp: Option<String>,
    password_revision_date: Option<DateTime<Utc>>,
    #[serde(default)]
    uris: Vec<Uri>,
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
pub struct Item {
    object: String,
    id: String,
    name: String,
    folder_id: Option<String>,
    organization_id: Option<String>,
    #[serde(rename = "type")]
    type_: i32,
    notes: Option<String>,
    favorite: bool,
    login: Option<Login>,
    collection_ids: Vec<String>,
    revision_date: DateTime<Utc>,
}

pub struct Session {
    pub key: String,
}

impl Session {
    pub fn open(key: &str) -> Session {
        Session {
            key: key.to_string(),
        }
    }

    pub fn unlock(password: &str) -> io::Result<Session> {
        let mut p = Command::new("bw")
            .args(&["unlock", "--raw"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        {
            let input = password.as_bytes();
            let stdin = p.stdin.as_mut().unwrap();
            stdin.write_all(input)?;
        }

        let output = p.wait_with_output().unwrap().stdout;
        let key = str::from_utf8(&output).unwrap().to_string();

        if key == "" {
            // TODO return a custom error for this.
            eprintln!("Unlock failed. Is the password correct?");
        }

        Ok(Session { key })
    }

    pub fn is_unlocked(&self) -> io::Result<bool> {
        Ok(self.status()?.status == "unlocked")
    }

    pub fn lock(&self) -> io::Result<()> {
        self.call(&["lock"])
    }

    pub fn status(&self) -> io::Result<Status> {
        self.call(&["status"])
    }

    pub fn list_folders(&self) -> io::Result<Vec<Folder>> {
        self.call(&["list", "folders"])
    }

    pub fn list_items(&self) -> io::Result<Vec<Item>> {
        self.call(&["list", "items"])
    }

    pub fn sync(&self) -> io::Result<()> {
        self.call(&["sync"])
    }

    fn call<T>(&self, args: &[&str]) -> io::Result<T>
    where
        T: DeserializeOwned,
    {
        let p = Command::new("bw")
            .args(args)
            .env("BW_SESSION", &self.key)
            .stdin(Stdio::null())
            .output()?;

        let output = str::from_utf8(&p.stdout).expect("invalid utf-8 data");
        Ok(serde_json::from_str(output).map_err(|err| {
            eprintln!("Deserialize to JSON failed: {} ...", &output[0..1000]);
            err
        })?)
    }
}
