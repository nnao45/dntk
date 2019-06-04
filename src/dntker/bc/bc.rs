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
        path.push(&util::DNTK_OPT.bc_path);
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
        if util::DNTK_OPT.scale != 0 {
            stdin += &format!("{}{}{}","scale=", util::DNTK_OPT.scale, ";");
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

        let stderr = capture.stderr_str().replace("\r", "");

        if stderr.is_empty() {
            let stdout = capture.stdout_str().replace("\r", "");

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