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

impl Default for BcExecuter {
    fn default() -> Self {
        let mut path = PathBuf::new();
        path.push(&util::DNTK_OPT.bc_path);
        BcExecuter {
            bc_path: path,
        }
    }
}

impl BcExecuter {
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

    fn handle(&self, capture: CaptureData) -> Result<String, BcError> {
        if let ExitStatus::Exited(status) = capture.exit_status {
            if status == 124 {
                return Err(BcError::Timeout);
            }
        }

        let stderr = capture.stderr_str().replace("\r", "");

        if stderr.is_empty() {
            let stdout = capture.stdout_str().replace("\r", "");

            if stdout.is_empty() || stdout.contains("syntax error") {
                Err(BcError::NoResult)
            } else {
                Ok(self.handle_output(stdout))
            }
        } else {
            Err(BcError::Error(self.handle_output(stderr)))
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

        match process.capture() {
            Ok(capture) => self.handle(capture),
            Err(e) => Err(BcError::PopenError(e)),
        }
    }
}