use std::path::PathBuf;
use std::env;


/// How should we deal with errors?
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ErrorHandling {
    /// Abort at the first error.
    pub fail_fast: bool,
    /// If there were errors generating docs for some of the crates, do we
    /// still upload to GitHub pages?
    pub upload_with_errors: bool,
}

impl Default for ErrorHandling {
    fn default() -> ErrorHandling {
        ErrorHandling {
            fail_fast: false,
            upload_with_errors: false,
        }
    }
}

/// The overall configuration for `rustc-internal-docs`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub rust_dir: PathBuf,
    pub git_repo: String,
    pub error_handling: ErrorHandling,
}

impl Config {
    pub fn default_config_file() -> PathBuf {
        let mut default_config = PathBuf::new();
        if let Some(home) = env::home_dir() {
            default_config.push(home);
        }
        default_config.push(".rustc-internal-docs.toml");

        default_config
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            rust_dir: PathBuf::from("/srv/github-backups/rust-lang/rust"),
            git_repo: String::from("git@github.com:Michael-F-Bryan/rustc-internal-docs.git"),
            error_handling: ErrorHandling::default(),
        }
    }
}
