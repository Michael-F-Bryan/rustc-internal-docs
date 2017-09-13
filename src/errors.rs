use std::process::Output;

error_chain! {
    foreign_links {
        Io(::std::io::Error) #[doc = "Wrapper around `std::io::Error`"];
    }

    errors {
        CommandFailed(cmd: String, output: Output) {
            description("Command Execution Failed")
            display("Command failed{}, \"{}\"",
                match output.status.code() {
                    None => String::new(),
                    Some(i) => format!(" with return code {}", i)
                },
                cmd)
        }
        DocGeneration(errors: Vec<Error>) {
            description("Documentation Generation Failed")
            display("Documentation generation failed with {} errors", errors.len())
        }
    }
}
