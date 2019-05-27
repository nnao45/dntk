pub extern crate subprocess;

use super::meta;
use std::path::PathBuf;
use subprocess::{Exec, PopenError, Redirection, ExitStatus};

#[derive(Debug)]
pub enum BcError {
    PopenError(PopenError),
    NoResult,
    Timeout,
    /// Maybe it is a syntax error.
    Error(String),
}

#[derive(Debug, PartialEq)]
pub struct BcExecuter {
    bc_path: PathBuf,
}

impl BcExecuter {
    pub fn new() -> Self {
        let mut path = PathBuf::new();
        path.push(meta::build_cli().get_matches().value_of("bc-path").unwrap());
        BcExecuter {
            bc_path: path,
        }
    }

    fn handle_output(&self, output: String) -> String {
        let len = output.len();

        let mut output = output.into_bytes();

        let output = unsafe {
            output.set_len(len - 1);

            String::from_utf8_unchecked(output)
        };

        match output.find("\\\n") {
            Some(index) => {
                let mut s = String::from(&output[..index]);

                s.push_str(&output[(index + 2)..].replace("\\\n", ""));

                s
            }
            None => output
        }
    }

    pub fn exec(&self, statement: &str) -> Result<String, BcError> {
        let mut stdin = "".to_string();
        if let Some(v) = meta::build_cli().get_matches().value_of("scale") {
            stdin += &format!("{}{}{}","scale=", v, ";");
        }
        stdin += &format!("{}\n", &statement);
        let process = Exec::cmd(&self.bc_path.as_os_str())
            .arg("-l")
            .arg("-q")
            .stdin(stdin.as_str())
            .stdout(Redirection::Pipe)
            .stderr(Redirection::Pipe);

        let capture = process.capture().map_err(|err| BcError::PopenError(err))?;

        if let ExitStatus::Exited(status) = capture.exit_status {
            if status == 124 {
                return Err(BcError::Timeout);
            }
        }

        let stderr = capture.stderr_str();

        if stderr.is_empty() {
            let stdout = capture.stdout_str();

            if stdout.is_empty() {
                Err(BcError::NoResult)
            } else {
                Ok(self.handle_output(stdout))
            }
        } else {
            Err(BcError::Error(self.handle_output(stderr)))
        }
    }
}

#[cfg(test)]
mod bc_tests {
    use super::{BcExecuter};

    #[test]
    fn test_handle_output(){
        let b = BcExecuter::new();
        let input1 = "1";
        let input1_newline = format!("{}{}", input1, "\n");
        assert_eq!(b.handle_output(input1_newline.to_string()), input1.to_string());
        let input2 = "0.12";
        let input2_newline = format!("{}{}", input2, "\n");
        assert_eq!(b.handle_output(input2_newline.to_string()), input2.to_string());
        let input3 = "211";
        let input3_newline = format!("{}{}", input3, "\\\n\n");
        assert_eq!(b.handle_output(input3_newline.to_string()), input3.to_string());
    }

    #[test]
    fn test_exec(){
        let b = BcExecuter::new();
        let input1 = "1+2";
        let output1= "3";
        assert_eq!(b.exec(input1).unwrap(), output1);
        let input2 = "0.12/a(123)";
        let output2 = ".07679182076851013335";
        assert_eq!(b.exec(input2).unwrap(), output2);
        let input3 = "2^2^2^2";
        let output3 = "65536";
        assert_eq!(b.exec(input3).unwrap(), output3);
        let input4 = "3x4x";
        #[cfg(target_os = "macos")]
        let output4 = "Error(\"(standard_in) 1: parse error\")";
        #[cfg(target_os = "linux")]
        let output4 = "Error(\"(standard_in) 1: syntax error\")";
        assert_eq!(format!("{:?}", b.exec(input4).err().unwrap()), output4);
    }
}