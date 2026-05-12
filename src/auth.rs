use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::{fs, path::PathBuf};

use anyhow::anyhow;
use crossterm::{
    event::{Event, KeyCode, KeyModifiers, read},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use directories::ProjectDirs;
use keyring_core::Entry;

const SERVICE: &str = "bb-dash";
const USERNAME_KEY: &str = "bitbucket_username";
const TOKEN_KEY: &str = "bitbucket_api_token";

pub struct Credentials {
    pub username: String,
    pub api_token: String,
}

pub fn prompt_and_store() -> anyhow::Result<()> {
    print!("Bitbucket username: ");
    std::io::stdout().flush()?;
    let mut username = String::new();
    std::io::stdin().read_line(&mut username)?;
    let username = username.trim().to_string();

    if username.is_empty() {
        return Err(anyhow!("username cannot be empty"));
    }

    let api_token = prompt_password("Bitbucket API token: ")?;

    if api_token.is_empty() {
        return Err(anyhow!("API token cannot be empty"));
    }

    let in_keychain = store(&username, &api_token)?;
    if in_keychain {
        println!("Credentials stored in system keychain.");
    } else {
        println!("Credentials stored in {}.", credentials_path()?.display());
    }

    Ok(())
}

pub fn load_stored() -> anyhow::Result<Option<Credentials>> {
    init_store();
    if let Ok(entry) = Entry::new(SERVICE, USERNAME_KEY) {
        if let Ok(username) = entry.get_password() {
            if let Ok(api_token) = Entry::new(SERVICE, TOKEN_KEY).and_then(|e| e.get_password()) {
                return Ok(Some(Credentials {
                    username,
                    api_token,
                }));
            }
        }
    }

    load_from_file()
}

fn store(username: &str, api_token: &str) -> anyhow::Result<bool> {
    init_store();
    let keychain_ok = (|| -> keyring_core::Result<()> {
        Entry::new(SERVICE, USERNAME_KEY)?.set_password(username)?;
        Entry::new(SERVICE, TOKEN_KEY)?.set_password(api_token)?;
        Ok(())
    })()
    .is_ok();

    if !keychain_ok {
        store_to_file(username, api_token)?;
    }
    Ok(keychain_ok)
}

fn init_store() {
    let _ = keyring::use_native_store(true);
}

fn store_to_file(username: &str, api_token: &str) -> anyhow::Result<()> {
    let path = credentials_path()?;
    let content = format!("username={username}\napi_token={api_token}\n");
    fs::write(&path, &content)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

fn load_from_file() -> anyhow::Result<Option<Credentials>> {
    let path = credentials_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path)?;
    let mut username = None;
    let mut api_token = None;

    for line in content.lines() {
        if let Some(v) = line.strip_prefix("username=") {
            username = Some(v.to_string());
        } else if let Some(v) = line.strip_prefix("api_token=") {
            api_token = Some(v.to_string());
        }
    }

    match (username, api_token) {
        (Some(u), Some(t)) => Ok(Some(Credentials {
            username: u,
            api_token: t,
        })),
        _ => Ok(None),
    }
}

fn credentials_path() -> anyhow::Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", "bb-dash")
        .ok_or_else(|| anyhow!("Could not determine config directory"))?;
    let config_dir = dirs.config_dir().to_owned();
    fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join("credentials"))
}

fn prompt_password(prompt: &str) -> anyhow::Result<String> {
    print!("{prompt}");
    std::io::stdout().flush()?;

    enable_raw_mode()?;
    let result = read_masked();
    let _ = disable_raw_mode();
    println!();

    result
}

fn read_masked() -> anyhow::Result<String> {
    let mut buf = String::new();

    loop {
        match read()? {
            Event::Key(key) => match key.code {
                KeyCode::Enter => return Ok(buf),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Err(anyhow!("Cancelled"));
                }
                KeyCode::Char(c) => {
                    buf.push(c);
                    print!("*");
                    std::io::stdout().flush()?;
                }
                KeyCode::Backspace if !buf.is_empty() => {
                    buf.pop();
                    print!("\x08 \x08");
                    std::io::stdout().flush()?;
                }
                _ => {}
            },
            _ => {}
        }
    }
}
