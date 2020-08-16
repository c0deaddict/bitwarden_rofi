use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::error::Error as StdError;
use std::fmt;
use std::io::Write;
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
    pub object: String,
    pub id: Option<String>, // None is the root folder.
    pub name: String,
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
pub struct Uri {
    pub uri: String,
    #[serde(rename = "match")]
    pub match_: Option<String>,
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
pub struct Login {
    pub username: Option<String>,
    pub password: Option<String>,
    pub totp: Option<String>,
    pub password_revision_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub uris: Vec<Uri>,
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
pub struct Item {
    pub object: String,
    pub id: String,
    pub name: String,
    pub folder_id: Option<String>,
    pub organization_id: Option<String>,
    #[serde(rename = "type")]
    pub type_: i32,
    pub notes: Option<String>,
    pub favorite: bool,
    pub login: Option<Login>,
    pub collection_ids: Vec<String>,
    pub revision_date: DateTime<Utc>,
}

pub struct Session {
    pub key: String,
}

#[derive(Debug, Clone)]
pub enum Error {
    UnlockFailed,
    FailedToDecrypt,
    UnexpectedResponse(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::UnlockFailed => f.write_str("UnlockFailed"),
            Error::FailedToDecrypt => f.write_str("FailedToDecrypt"),
            Error::UnexpectedResponse(_) => f.write_str("UnexpectedResponse"),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::UnlockFailed => "UnlockFailed",
            Error::FailedToDecrypt => "FailedToDecrypt",
            Error::UnexpectedResponse(_) => "UnexpectedResponse",
        }
    }
}

impl Session {
    pub fn open(key: &str) -> Session {
        Session {
            key: key.to_string(),
        }
    }

    pub fn unlock(password: &str) -> Result<Session> {
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
        let key = str::from_utf8(&output)?.to_string();

        if key == "" {
            return Err(anyhow::Error::from(Error::UnlockFailed));
        }

        Ok(Session { key })
    }

    pub fn is_unlocked(&self) -> Result<bool> {
        Ok(self.status()?.status == "unlocked")
    }

    pub fn lock(&self) -> Result<()> {
        let resp = self.call_str(&["lock"])?;
        match &resp[..] {
            "Your vault is locked." => Ok(()),
            _ => Err(anyhow::Error::from(Error::UnexpectedResponse(resp))),
        }
    }

    pub fn status(&self) -> Result<Status> {
        self.call_json(&["status"])
    }

    pub fn list_folders(&self) -> Result<Vec<Folder>> {
        self.call_json(&["list", "folders"])
    }

    pub fn list_items(&self) -> Result<Vec<Item>> {
        self.call_json(&["list", "items"])
    }

    pub fn sync(&self) -> Result<()> {
        self.call_json(&["sync"])
    }

    fn call_str(&self, args: &[&str]) -> Result<String> {
        let p = Command::new("bw")
            .args(args)
            .env("BW_SESSION", &self.key)
            .stdin(Stdio::null())
            .output()?;

        let output = str::from_utf8(&p.stdout)?;
        self.check_for_errors(&output)?;
        Ok(output.to_string())
    }

    fn call_json<T>(&self, args: &[&str]) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let output = self.call_str(args)?;
        Ok(serde_json::from_str(&output)?)
    }

    fn check_for_errors(&self, output: &str) -> std::result::Result<(), Error> {
        if output.starts_with("Failed to decrypt.") {
            Err(Error::FailedToDecrypt)
        } else {
            Ok(())
        }
    }
}
