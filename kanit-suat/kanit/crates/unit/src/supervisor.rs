use async_process::{Child, Command};

use kanit_common::error::{Context, ErrorKind, Result};
use kanit_supervisor::{RestartPolicy, Supervisor};

pub struct SupervisorBuilder(Supervisor);

#[allow(dead_code)]
impl SupervisorBuilder {
    pub fn new<S: ToString, I: IntoIterator<Item = S>>(cmd: S, args: I) -> Self {
        Self(Supervisor {
            cmd: cmd.to_string(),
            args: args.into_iter().map(|s| s.to_string()).collect(),
            restart_delay: None,
            restart_attempts: None,
            restart_policy: None,
            pwd: None,
            root: None,
            env: vec![],
            group: None,
            user: None,
            stdout: None,
            stderr: None,
        })
    }

    pub fn from_supervisor(supervisor: Supervisor) -> Self {
        Self(supervisor)
    }

    pub fn build(self) -> Supervisor {
        self.0
    }

    pub fn spawn(self) -> Result<Child> {
        let mut args = vec![];

        if let Some(delay) = self.0.restart_delay {
            args.push("-d".to_string());
            args.push(delay.to_string());
        }

        if let Some(attempts) = self.0.restart_attempts {
            args.push("-a".to_string());
            args.push(attempts.to_string());
        }

        if let Some(policy) = self.0.restart_policy {
            args.push("-P".to_string());
            args.push(policy.to_string());
        }

        if let Some(pwd) = self.0.pwd {
            args.push("-p".to_string());
            args.push(pwd);
        }

        if let Some(root) = self.0.root {
            args.push("-r".to_string());
            args.push(root);
        }

        for pair in self.0.env {
            args.push("-e".to_string());
            args.push(pair);
        }

        if let Some(group) = self.0.group {
            args.push("-g".to_string());
            args.push(group);
        }

        if let Some(user) = self.0.user {
            args.push("-u".to_string());
            args.push(user);
        }

        if let Some(stdout) = self.0.stdout {
            args.push("--stdout".to_string());
            args.push(stdout);
        }

        if let Some(stderr) = self.0.stderr {
            args.push("--stderr".to_string());
            args.push(stderr);
        }

        args.push("--".to_string());

        args.push(self.0.cmd);

        args.extend_from_slice(&self.0.args);

        Command::new("kanit-supervisor")
            .args(args)
            .spawn()
            .context_kind("failed to spawn supervisor", ErrorKind::Recoverable)
    }

    pub fn restart_delay(mut self, delay: u64) -> Self {
        self.0.restart_delay = Some(delay);
        self
    }

    pub fn restart_attempts(mut self, attempts: u64) -> Self {
        self.0.restart_attempts = Some(attempts);
        self
    }

    pub fn restart_policy(mut self, policy: RestartPolicy) -> Self {
        self.0.restart_policy = Some(policy);
        self
    }

    pub fn pwd(mut self, pwd: String) -> Self {
        self.0.pwd = Some(pwd);
        self
    }

    pub fn root(mut self, root: String) -> Self {
        self.0.root = Some(root);
        self
    }

    pub fn env(mut self, key: String, value: String) -> Self {
        self.0.env.push(format!("{}={}", key, value));
        self
    }

    pub fn group(mut self, group: String) -> Self {
        self.0.group = Some(group);
        self
    }

    pub fn user(mut self, user: String) -> Self {
        self.0.user = Some(user);
        self
    }

    pub fn stdout(mut self, stdout: String) -> Self {
        self.0.stdout = Some(stdout);
        self
    }

    pub fn stderr(mut self, stderr: String) -> Self {
        self.0.stderr = Some(stderr);
        self
    }
}
