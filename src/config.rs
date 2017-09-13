use std::path::PathBuf;
use std::env;


/// The overall configuration for `rustc-internal-docs`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// Location of your local checkout of the rust-lang/rust repository.
    pub rust_dir: PathBuf,
    /// The git repository who's GitHub Pages to push to.
    pub git_repo: String,
    /// How errors are handled.
    pub error_handling: ErrorHandling,
    /// Tweak which steps are run.
    pub stages: Stages,
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
            rust_dir: PathBuf::from("/tmp/rust-lang/rust"),
            git_repo: String::from("git@github.com:Michael-F-Bryan/rustc-internal-docs.git"),
            error_handling: ErrorHandling::default(),
            stages: Stages::default(),
        }
    }
}

/// Options for tweaking which stages of the documentation generation to execute.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
#[serde(rename = "stages")]
#[serde(rename_all = "kebab-case")]
pub struct Stages {
    pub skip_git_update: bool,
    pub skip_upload: bool,
}

/// How should we deal with errors?
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
#[serde(rename = "error-handling")]
#[serde(rename_all = "kebab-case")]
pub struct ErrorHandling {
    /// Abort at the first error.
    pub fail_fast: bool,
    /// If there were errors generating docs for some of the crates, do we
    /// still upload to GitHub pages?
    pub upload_with_errors: bool,
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use toml;
    use helpers;

    #[test]
    fn make_sure_we_can_parse_the_default_config_file() {
        let default_file = Path::new(env!("CARGO_MANIFEST_DIR")).join("rustc-internal-docs.toml");
        let should_be = Config {
            rust_dir: PathBuf::from("/srv/github-backups/rust-lang/rust"),
            error_handling: ErrorHandling {
                upload_with_errors: true,
                ..Default::default()
            },
            stages: Stages {
                ..Default::default()
            },
            ..Default::default()
        };

        let contents = helpers::read_file(&default_file).unwrap();
        let got: Config = toml::from_str(&contents).unwrap();

        assert_eq!(got, should_be);
    }
}
