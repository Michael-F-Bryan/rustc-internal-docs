use std::process::Output;

error_chain! {
    foreign_links {
        Io(::std::io::Error) #[doc = "Wrapper around `std::io::Error`"];
    }

    errors {
        CommandFailed(cmd: String, output: Output) {
            description("Command Execution Failed")
            display("Command failed{}, \"{}\"",
                ::helpers::pretty_print_return_code(output.status.code()),
                cmd)
        }
        DocGeneration(errors: Vec<(String, Error)>) {
            description("Documentation Generation Failed")
            display("Documentation generation failed with {} errors", errors.len())
        }
    }
}
