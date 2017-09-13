use std::path::Path;
use std::process::{Command, Output};
use std::io::Read;
use std::fs::File;
use copy_dir;
use shlex;

use errors::*;

/// Run a command, possibly in a specific directory. Using `format!(...)` to
/// turn the input tokens into a command string.
///
/// # Examples
///
/// ```rust,ignore
/// # use std::path::Path;
/// # #[macro_use]
/// # extern crate rustc_internal_docs;
/// let root = Path::new("/foo/bar/baz"):
/// cmd!(in root, "git pull {} {} --ff-only", "origin", "master")?;
/// ```
#[macro_export]
macro_rules! cmd {
    (in $dir:expr, $( $arg:expr ),+) => {
        $crate::helpers::execute_command(format!($( $arg ),*), Some($dir.as_ref()))
    };
    ($( $arg:expr ),+) => {
        $crate::helpers::execute_command(format!($( $arg ),*), None)
    };
}


/// Iterate through all the errors in an `error-chain`, printing them to the
/// screen and then bailing with an exit status of 1.
#[macro_export]
macro_rules! backtrace {
    ($maybe_err:expr) => {
        match $maybe_err {
            Ok(val) => val,
            Err(e) => {
                $crate::helpers::print_backtrace(&e, 0);
                ::std::process::exit(1);
            }
        }
    }
}

pub fn print_backtrace(e: &Error, indent: usize) {
    error!("{}Error: {}", "\t".repeat(indent), e);

    for cause in e.iter().skip(1) {
        error!("{}Caused By: {}", "\t".repeat(indent + 1), cause);
    }
}

/// Print either "" or " with return code 1", depending on whether we got a
/// return code.
pub fn pretty_print_return_code(code: Option<i32>) -> String {
    match code {
        None => String::new(),
        Some(i) => format!(" with return code {}", i),
    }
}


/// Print the stdout and stderr of a `std::process::Output` at the specified
/// logging level. The default is to print with `debug!()`.
macro_rules! print_output {
    ($output:expr) => { print_output!(debug, $output) };
    ($level:ident, $output:expr) => {{
        let output: &::std::process::Output = &$output;
        if !output.stdout.is_empty() {
            $level!("Stdout:");
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                $level!("{}", line);
            }
        } 
        if !output.stderr.is_empty() {
            $level!("Stderr:");
            for line in String::from_utf8_lossy(&output.stderr).lines() {
                $level!("{}", line);
            }
        }
    }}
}

/// Pretty much just `cp -r $from $to`.
pub fn recursive_copy<F: AsRef<Path>, T: AsRef<Path>>(from: F, to: T) -> Result<()> {
    let mut errors = copy_dir::copy_dir(from, to)?;
    if errors.is_empty() {
        Ok(())
    } else {
        // if there were any errors we'll blow up and pass it to the user
        let an_error = errors.pop().unwrap();
        bail!(an_error)
    }
}


pub fn execute_command<S: AsRef<str>>(cmd: S, cd_dir: Option<&Path>) -> Result<Output> {
    let cmd = cmd.as_ref();
    let parsed = match shlex::split(cmd) {
        Some(bits) => bits,
        None => bail!("Invalid command"),
    };

    if parsed.is_empty() {
        bail!("Can't execute an empty command");
    }

    let mut command_builder = Command::new(&parsed[0]);

    for arg in &parsed[1..] {
        command_builder.arg(arg);
    }

    if let Some(cd) = cd_dir {
        command_builder.current_dir(cd);
    }

    debug!(r#"Executing "{}""#, cmd);
    let output = command_builder
        .output()
        .chain_err(|| format!("Command not found, {}", parsed[0]))?;

    debug!(
        r#""{}" completed{}"#,
        cmd,
        pretty_print_return_code(output.status.code())
    );

    if !output.status.success() {
        warn!("Command failed: {}", cmd);
        print_output!(output);
        bail!(ErrorKind::CommandFailed(cmd.to_string(), output));
    }

    Ok(output)
}

/// Read a file's contents into memory.
pub fn read_file<P: AsRef<Path>>(filename: P) -> Result<String> {
    let filename = filename.as_ref();

    let mut buffer = String::new();
    File::open(filename)
        .chain_err(|| format!("Couldn't open {}", filename.display()))?
        .read_to_string(&mut buffer)
        .chain_err(|| "Couldn't read from the file")?;

    Ok((buffer))
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn run_a_passing_command() {
        assert!(execute_command("true", None).is_ok());
    }

    #[test]
    fn run_a_failing_command() {
        assert!(execute_command("false", None).is_err());
    }

    #[test]
    fn detect_running_nonexistent_commands() {
        assert!(execute_command("foo bar baz", None).is_err());
    }

    #[test]
    fn macro_works_correctly_and_executes_in_a_directory() {
        let temp = TempDir::new("rustc-internal-docs").unwrap();
        let dummy_file = temp.path().join("dummy.txt");
        File::create(&dummy_file).unwrap();

        assert!(dummy_file.exists());
        cmd!(in temp.path(), "rm {}", dummy_file.display()).unwrap();
        assert!(!dummy_file.exists());
    }

    #[test]
    fn command_with_spaces_in_it() {
        let output = cmd!(r#"echo "Hello World""#).unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Hello World"));
        assert!(!stdout.contains('"'));
    }
}
