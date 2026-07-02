use anyhow::Result;
use std::process::Stdio;
use tokio::process::{Child, Command};

pub struct PtyProcess {
    child: Child,
}

impl PtyProcess {
    pub fn new(command: &str, working_dir: &str) -> Result<Self> {
        let child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(PtyProcess { child })
    }

    pub async fn wait(&mut self) -> Result<i32> {
        let status = self.child.wait().await?;
        Ok(status.code().unwrap_or(-1))
    }
}
