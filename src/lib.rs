extern crate chrono;
extern crate copy_dir;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate regex;
#[macro_use]
extern crate serde_derive;
extern crate shlex;
extern crate tempdir;

#[cfg(test)]
extern crate toml;

pub mod errors;
mod config;
#[macro_use]
pub mod helpers;

use std::path::Path;
use std::fs::{self, File};
use std::io::Write;
use tempdir::TempDir;
use regex::Regex;
use chrono::Local;

use errors::*;
pub use config::Config;

const RUST_GITHUB_REPO: &'static str = "https://github.com/rust-lang/rust";


/// The main entrypoint of the program.
///
/// This will:
///
/// 1. Update the rust repo to the latest version
/// 2. Tell rustbuild to build the docs by tweaking its `config.toml`
/// 3. Iterate through each crate in `src/` and build its docs with
///    `./x.py doc src/libfoo`
/// 4. Upload the generated documentation to github pages if there were
///    no errors or if `config.error_handling.upload_with_errors` is set.
pub fn run(cfg: Config) -> Result<()> {
    info!("Starting documentation generation");

    if !cfg.stages.skip_git_update {
        update_rust_repo(&cfg.rust_dir).chain_err(|| "Failed to update the rust repo")?;
    }
    setup_rustbuild_config_file(&cfg.rust_dir)
        .chain_err(|| "Couldn't make sure the config.toml is set properly")?;

    info!("Generating documentation");
    generate_docs(&cfg.rust_dir)?;

    if !cfg.stages.skip_upload {
        upload_docs(&cfg.rust_dir, &cfg.git_repo).chain_err(|| "Uploading docs failed")?;
    }

    info!("Documentation generation completed successfully");
    Ok(())
}

/// Generate the documentation for the specified library in a local checkout
/// of the rust repo.
fn generate_docs(root: &Path) -> Result<()> {
    cmd!(in root, "./x.py doc -v")?;
    Ok(())
}

fn upload_docs(root: &Path, git_repo: &str) -> Result<()> {
    // FIXME: should probably not hard-code this...
    let target = "x86_64-unknown-linux-gnu";

    let docs_dir = root.join("build").join(target).join("crate-docs");

    if !docs_dir.exists() {
        bail!(
            "Couldn't find {} ... were any docs even generated?",
            docs_dir.display()
        );
    }

    let temp = TempDir::new("rustc-internal-docs")?;
    cmd!("git clone {} {}", git_repo, temp.path().display())?;
    cmd!(in temp.path(), "git checkout gh-pages")?;

    debug!("Copying generated docs to {}", temp.path().display());
    cmd!(
        "rsync -a {}/ {}/",
        docs_dir.display(),
        temp.path().display()
    )?;

    // Make a page to redirect people to rustc/index.html if it doesn't
    // already exist
    let index = temp.path().join("index.html");
    if !index.exists() {
        let redirect = r#"<html><meta http-equiv="refresh" content="0; URL=rustc/index.html"></html>"#;
        File::create(index)?.write_all(redirect.as_bytes())?;
    }

    debug!("Pushing to GitHub pages");
    cmd!(in temp.path(), "git add .")?;
    cmd!(in temp.path(), r#"git commit -m "updated documentation at {}""#, Local::now())?;
    cmd!(in temp.path(), "git push origin gh-pages")?;
    debug!("Docs uploaded");

    Ok(())
}

/// Do a `git clone` or `git pull` to make sure the Rust repo is up to date.
fn update_rust_repo(root: &Path) -> Result<()> {
    if !root.exists() {
        info!("Rust directory not found, cloning into {}", root.display());
        cmd!("git clone {} {}", RUST_GITHUB_REPO, root.display())?;
    } else {
        info!("Updating rust checkout");
        cmd!(in root, "git pull origin master --ff-only")?;
    }

    info!("Rust directory is up to date");
    Ok(())
}

/// Copy across the template config file from `src/bootstrap/config.toml.example`
/// if it doesn't already exist, and make sure we tell rustbuild to build *all*
/// docs.
fn setup_rustbuild_config_file(root: &Path) -> Result<()> {
    debug!("Making sure config.toml is set up correctly");

    let config_file = root.join("config.toml");
    let template = root.join("config.toml.example");

    if !config_file.exists() {
        fs::copy(&template, &config_file).chain_err(|| {
            format!("Couldn't copy the template config ({})", template.display())
        })?;
    }

    let mut config_contents = helpers::read_file(&config_file)?;

    let patterns = vec![
        (
            Regex::new("^#?compiler-docs .*$").unwrap(),
            "compiler-docs = true",
        ),
        (Regex::new("^#?docs .*$").unwrap(), "docs = true"),
    ];

    for (pattern, replace) in patterns {
        config_contents = pattern.replace(&config_contents, replace).into_owned();
    }

    debug!("Config file: {}", config_file.display());

    File::create(&config_file)
        .expect("Config file should already exist")
        .write_all(config_contents.as_bytes())
        .chain_err(|| "Couldn't write to config.toml")?;

    Ok(())
}

