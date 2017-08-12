#!/bin/bash
#
# This is a small bash script which I use to generate a copy of `rustc`'s 
# internal documentation and upload them to GitHub pages.
#
# Use at your own risk
#
# I typically have it set up as a cron job.


set -e

TARGET_TRIPLE=$(rustup show | sed -e 's/Default host: \([^\s]*\)/\1/p' --quiet)
RUST_DIR=/srv/github-backups/rust-lang/rust
GIT_REPO="git@github.com:Michael-F-Bryan/rustc-internal-docs.git"


log() {
  if [ -z "$SYSTEM_LOG" ]; then
    echo "[$(date --rfc-3339=seconds)] $@" 1>&2
  else
    logger --tag rustc-internal-docs "$@"
  fi
}


log "Starting documentation generation at $(date)"
log "Target Triple: $TARGET_TRIPLE"

if [ ! -d "$RUST_DIR" ]; then
  log "Rust directory not found. Cloning from github..."
  git clone "https://github.com/rust-lang/rust" "$RUST_DIR"
  cd $RUST_DIR
else
  cd $RUST_DIR
  log "Fetching latest changes"
  git pull origin master
fi

if [ ! -f "./config.toml" ]; then
  cp "src/bootstrap/config.toml.example" "./config.toml"
  sed -e "s/#compiler-docs = true/compiler-docs = true/" -i ./config.toml
fi

log "Generating documentation"
./x.py doc --incremental
log "Documentation generated"

TEMP_DIR=$(mktemp -d)
CRATE_DOCS_DIR="$RUST_DIR/build/$TARGET_TRIPLE/crate-docs/"

log "Copying crate docs to a temporary directory"
cd "$TEMP_DIR"
cp -r $CRATE_DOCS_DIR/* "$TEMP_DIR"

# Make sure going to the root index.html redirects to rustc's internal docs
echo '<html><meta http-equiv="refresh" content="0; URL=rustc/index.html"></html>' > index.html

git init .
git remote add origin "$GIT_REPO"
git checkout -b gh-pages
git add .
git commit -m "Updated docs"

log "Pushing new docs up to GitHub"
git push origin gh-pages --force

rm -r -f "$TEMP_DIR"
log "Documentation generation and uploading finished"

