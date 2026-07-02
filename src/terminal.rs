use crossterm::event::KeyEvent;

#[derive(Clone)]
pub struct Pane {
    pub name: String,
    pub content: String,
    pub input: String,
    pub history: Vec<String>,
}

impl Pane {
    pub fn new(name: &str) -> Self {
        Pane {
            name: name.to_string(),
            content: String::new(),
            input: String::new(),
            history: Vec::new(),
        }
    }

    pub fn execute(&mut self, command: &str) {
        let cmd = command.trim().to_string();
        if cmd.is_empty() {
            return;
        }

        self.history.push(cmd.clone());

        let output = match cmd.as_str() {
            "help" => "Available commands:\n  help    - Show this help\n  clear   - Clear screen\n  ls      - List files\n  pwd     - Print working directory\n  echo    - Echo text\n  calc    - Calculator (e.g. calc 2+2)\n  date    - Show current date/time\n  whoami  - Show current user\n  uname   - Show system info".to_string(),
            "clear" => {
                self.content.clear();
                return;
            }
            "ls" => {
                let output = std::process::Command::new("ls")
                    .arg("-la")
                    .output()
                    .unwrap_or_else(|_| std::process::Output {
                        stdout: b"ls: command not found\n".to_vec(),
                        stderr: Vec::new(),
                        status: std::process::ExitStatus::default(),
                    });
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            "pwd" => std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "/".to_string()),
            "date" => {
                let output = std::process::Command::new("date")
                    .output()
                    .unwrap_or_else(|_| std::process::Output {
                        stdout: b"date: command not found\n".to_vec(),
                        stderr: Vec::new(),
                        status: std::process::ExitStatus::default(),
                    });
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            "whoami" => {
                let output = std::process::Command::new("whoami")
                    .output()
                    .unwrap_or_else(|_| std::process::Output {
                        stdout: b"whoami: command not found\n".to_vec(),
                        stderr: Vec::new(),
                        status: std::process::ExitStatus::default(),
                    });
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            "uname" => {
                let output = std::process::Command::new("uname")
                    .arg("-a")
                    .output()
                    .unwrap_or_else(|_| std::process::Output {
                        stdout: b"uname: command not found\n".to_vec(),
                        stderr: Vec::new(),
                        status: std::process::ExitStatus::default(),
                    });
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            other if other.starts_with("echo ") => other[5..].to_string(),
            other if other.starts_with("calc ") => {
                let expr = &other[5..];
                simple_calc(expr)
            }
            other => {
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(other)
                    .output()
                    .unwrap_or_else(|_| std::process::Output {
                        stdout: Vec::new(),
                        stderr: format!("Command not found: {}\n", other).into_bytes(),
                        status: std::process::ExitStatus::default(),
                    });
                let mut result = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                if !stderr.is_empty() {
                    result.push_str(&stderr);
                }
                result
            }
        };

        self.content.push_str(&format!("$ {}\n{}\n", cmd, output));
    }
}

fn simple_calc(expr: &str) -> String {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() != 3 {
        return "Usage: calc <num> <op> <num>".to_string();
    }
    let a: f64 = match parts[0].parse() {
        Ok(v) => v,
        Err(_) => return format!("Invalid number: {}", parts[0]),
    };
    let b: f64 = match parts[2].parse() {
        Ok(v) => v,
        Err(_) => return format!("Invalid number: {}", parts[2]),
    };
    match parts[1] {
        "+" => format!("{}", a + b),
        "-" => format!("{}", a - b),
        "*" => format!("{}", a * b),
        "/" => {
            if b == 0.0 {
                "Error: division by zero".to_string()
            } else {
                format!("{}", a / b)
            }
        }
        op => format!("Unknown operator: {}", op),
    }
}

pub struct Tab {
    pub name: String,
    pub panes: Vec<Pane>,
    pub active_pane: usize,
}

impl Tab {
    pub fn new(name: &str) -> Self {
        Tab {
            name: name.to_string(),
            panes: vec![Pane::new("Terminal")],
            active_pane: 0,
        }
    }

    pub fn split_horizontal(&mut self) {
        let idx = self.panes.len() + 1;
        self.panes.push(Pane::new(&format!("Pane {}", idx)));
        self.active_pane = self.panes.len() - 1;
    }

    pub fn split_vertical(&mut self) {
        let idx = self.panes.len() + 1;
        self.panes.push(Pane::new(&format!("Pane {}", idx)));
        self.active_pane = self.panes.len() - 1;
    }

    pub fn close_pane(&mut self) {
        if self.panes.len() > 1 {
            self.panes.remove(self.active_pane);
            if self.active_pane >= self.panes.len() {
                self.active_pane = self.panes.len() - 1;
            }
        }
    }
}
