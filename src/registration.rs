//! Append-only dashboard configuration writes. Raw existing settings are never serialized.
use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::Mutex,
};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NewApi {
    pub name: String,
    pub url: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_body_contains: Option<String>,
    pub interval_secs: u64,
    pub timeout_secs: u64,
}
#[derive(Debug)]
pub enum AddError {
    Invalid(&'static str, &'static str),
    Duplicate,
    Changed,
    Busy,
    Disabled,
    Storage,
}
impl AddError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::Invalid(_, message) => message,
            Self::Duplicate => "That target name already exists. Choose a different name.",
            Self::Changed => "The configuration changed outside this dashboard. Restart the monitor before adding an API.",
            Self::Busy => "Another API is being saved. Try again shortly.",
            Self::Disabled => "Adding APIs is unavailable for this monitor configuration.",
            Self::Storage => "The API could not be saved. Check configuration and database permissions, then try again.",
        }
    }
    pub fn field(&self) -> Option<&'static str> {
        match self {
            Self::Invalid(field, _) => Some(field),
            Self::Duplicate => Some("name"),
            _ => None,
        }
    }
}
impl NewApi {
    pub fn validate(&self) -> Result<(), AddError> {
        if self.name.is_empty()
            || self.name.len() > 80
            || !self
                .name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        {
            return Err(AddError::Invalid(
                "name",
                "Use 1–80 letters, numbers, dots, hyphens, or underscores for the name.",
            ));
        }
        let url = url::Url::parse(&self.url)
            .map_err(|_| AddError::Invalid("url", "Enter a complete HTTP or HTTPS URL."))?;
        if self.url.len() > 2048
            || self.url.contains("${")
            || !["http", "https"].contains(&url.scheme())
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err(AddError::Invalid("url", "Use an HTTP or HTTPS URL without embedded credentials or environment placeholders."));
        }
        if !["GET", "HEAD"].contains(&self.method.as_str()) {
            return Err(AddError::Invalid(
                "method",
                "Choose GET or HEAD. Advanced methods are configured in TOML.",
            ));
        }
        if self
            .expected_status
            .is_some_and(|status| !(100..=599).contains(&status))
        {
            return Err(AddError::Invalid(
                "expected_status",
                "Use an HTTP status from 100 to 599, or leave it blank for any 2xx.",
            ));
        }
        if self.interval_secs < 5 || self.interval_secs > 31_536_000 {
            return Err(AddError::Invalid(
                "interval_secs",
                "The interval must be between 5 seconds and one year.",
            ));
        }
        if self.timeout_secs == 0 || self.timeout_secs >= self.interval_secs {
            return Err(AddError::Invalid(
                "timeout_secs",
                "The timeout must be positive and shorter than the interval.",
            ));
        }
        if self
            .expected_body_contains
            .as_ref()
            .is_some_and(|text| text.len() > 4096 || text.contains("${"))
        {
            return Err(AddError::Invalid("expected_body_contains", "Use up to 1,024 characters of literal expected text, without environment placeholders."));
        }
        if self.method == "HEAD"
            && self
                .expected_body_contains
                .as_ref()
                .is_some_and(|text| !text.is_empty())
        {
            return Err(AddError::Invalid(
                "expected_body_contains",
                "HEAD responses have no body. Remove the expected text or choose GET.",
            ));
        }
        Ok(())
    }
    pub fn block(&self) -> Result<String, AddError> {
        self.validate()?;
        // TOML's serializer escapes quotes/newlines; raw browser input is never concatenated as syntax.
        toml::to_string(self)
            .map(|fields| format!("\n\n[[target]]\n{fields}"))
            .map_err(|_| AddError::Storage)
    }
}
pub struct ConfigEditor {
    path: PathBuf,
    expected: Mutex<String>,
}
impl ConfigEditor {
    pub fn open(path: PathBuf, running: &Config) -> Result<Self, AddError> {
        let metadata = fs::symlink_metadata(&path).map_err(|_| AddError::Disabled)?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.permissions().readonly()
            || metadata.len() > 1_048_576
        {
            return Err(AddError::Disabled);
        }
        let expected = fs::read_to_string(&path).map_err(|_| AddError::Disabled)?;
        if &Config::parse(&expected).map_err(|_| AddError::Changed)? != running {
            return Err(AddError::Changed);
        }
        let path = fs::canonicalize(path).map_err(|_| AddError::Disabled)?;
        Ok(Self {
            path,
            expected: Mutex::new(expected),
        })
    }
    pub fn append(&self, block: &str) -> Result<(), AddError> {
        let mut expected = self.expected.lock().map_err(|_| AddError::Storage)?;
        let mut lock_name = self.path.as_os_str().to_os_string();
        lock_name.push(".edit.lock");
        let lock_path = PathBuf::from(lock_name);
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let lock = options.open(lock_path).map_err(|_| AddError::Storage)?;
        fs2::FileExt::try_lock_exclusive(&lock).map_err(|_| AddError::Busy)?;
        let metadata = fs::symlink_metadata(&self.path).map_err(|_| AddError::Storage)?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.permissions().readonly()
        {
            return Err(AddError::Storage);
        }
        let current = fs::read_to_string(&self.path).map_err(|_| AddError::Storage)?;
        if current != *expected {
            return Err(AddError::Changed);
        }
        let candidate = format!("{current}{block}");
        if candidate.len() > 1_048_576 {
            return Err(AddError::Storage);
        }
        Config::parse(&candidate).map_err(|_| AddError::Changed)?;
        let parent = self.path.parent().ok_or(AddError::Storage)?;
        let mut temporary =
            tempfile::NamedTempFile::new_in(parent).map_err(|_| AddError::Storage)?;
        temporary
            .write_all(candidate.as_bytes())
            .map_err(|_| AddError::Storage)?;
        temporary
            .as_file()
            .set_permissions(metadata.permissions())
            .map_err(|_| AddError::Storage)?;
        temporary
            .as_file()
            .sync_all()
            .map_err(|_| AddError::Storage)?;
        if fs::read_to_string(&self.path).map_err(|_| AddError::Storage)? != current {
            return Err(AddError::Changed);
        }
        temporary
            .persist(&self.path)
            .map_err(|_| AddError::Storage)?;
        *expected = candidate;
        // Once replacement succeeded, it is saved even if a directory fsync is unsupported.
        #[cfg(unix)]
        {
            if let Ok(directory) = std::fs::File::open(parent) {
                let _ = directory.sync_all();
            }
        }
        Ok(())
    }
}
