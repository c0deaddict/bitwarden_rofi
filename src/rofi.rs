//! https://github.com/krruzic/rustofi/blob/master/src/window.rs

use anyhow::Result;
use std::error::Error;
use std::fmt;
use std::io::Write;
use std::process::{Command, Stdio};
use std::str;

pub struct RofiWindow {
    pub prompt: String,
    pub message: Option<String>,
    pub width: Option<i32>,
    pub lines: i32,
    pub password: bool,
    pub extra_args: Vec<String>,
}

pub enum RofiResponse {
    Entry(String),
    Cancel,
    CustomKey(i32),
}

#[derive(Debug)]
pub struct RofiError {
    pub details: String,
}

impl fmt::Display for RofiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for RofiError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl RofiWindow {
    fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        args.extend(vec!["-p".to_string(), self.prompt.clone()]);
        if let Some(msg) = self.message.as_ref() {
            args.extend(vec!["-mesg".to_string(), msg.to_string()]);
        }
        if let Some(width) = self.width {
            args.extend(vec!["-width".to_string(), width.to_string()]);
        }
        args.extend(vec!["-lines".to_string(), self.lines.to_string()]);
        if self.password {
            args.push("-password".to_string());
        }
        args.extend(self.extra_args.clone());
        args
    }

    pub fn new(prompt: &str) -> RofiWindow {
        RofiWindow {
            prompt: prompt.to_string(),
            message: None,
            width: None,
            lines: 10,
            password: false,
            extra_args: vec![],
        }
    }

    pub fn prompt(mut self, prompt: &str) -> Self {
        self.prompt = prompt.to_string();
        self
    }

    pub fn message(mut self, msg: &str) -> Self {
        self.message = Some(msg.to_string());
        self
    }

    pub fn width(mut self, width: i32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn lines(mut self, lines: i32) -> Self {
        self.lines = lines;
        self
    }

    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    pub fn add_args(mut self, args: Vec<&str>) -> Self {
        self.extra_args.extend(args.iter().map(|s| s.to_string()));
        self
    }

    pub fn kb_custom(mut self, idx: i32, key: &str) -> Self {
        let arg = format!("-kb-custom-{}", idx);
        self.extra_args.extend(vec![arg, key.to_string()]);
        self
    }

    pub fn matching(mut self, algo: &str) -> Self {
        self.extra_args
            .extend(vec!["-matching".to_string(), algo.to_string()]);
        self
    }

    pub fn show(self, options: Vec<String>) -> Result<RofiResponse> {
        let args = self.to_args();

        println!("args: {}", args.join(" "));

        let mut proc = Command::new("rofi")
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        {
            let options = options.join("\n");
            let input = options.as_bytes();
            let stdin = proc.stdin.as_mut().unwrap();
            stdin.write_all(input)?;
        }

        let output = proc.wait_with_output()?;
        if let Some(code) = output.status.code() {
            if code >= 10 && code <= 28 {
                return Ok(RofiResponse::CustomKey(code - 9));
            } else if code == 1 {
                return Ok(RofiResponse::Cancel);
            } else if code != 0 {
                let e = RofiError {
                    details: format!("unexpected exit code: {}", code),
                };
                return Err(e.into());
            }
        }
        let selection = str::from_utf8(&output.stdout)?.trim().to_string();

        Ok(RofiResponse::Entry(selection))
    }
}

impl RofiResponse {
    pub fn entry(&self) -> Result<String> {
        match self {
            RofiResponse::Entry(s) => Ok(s.to_string()),
            _ => {
                let e = RofiError {
                    details: "expected an entry as response".to_string(),
                };
                Err(e.into())
            }
        }
    }
}
