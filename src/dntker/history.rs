use std::env;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct History {
    entries: Vec<String>,
    cursor: Option<usize>,
    path: Option<PathBuf>,
}

impl History {
    pub(crate) fn load() -> Self {
        let path = Self::history_file_path();
        let mut entries = Vec::new();
        if let Some(history_path) = &path {
            if let Ok(file) = fs::File::open(history_path) {
                let reader = BufReader::new(file);
                for line in reader.lines().map_while(Result::ok) {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        entries.push(trimmed.to_string());
                    }
                }
            }
        }

        History {
            entries,
            cursor: None,
            path,
        }
    }

    pub(crate) fn push(&mut self, entry: &str) {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            self.cursor = None;
            return;
        }

        if self.entries.last().is_some_and(|last| last == trimmed) {
            self.cursor = None;
            return;
        }

        self.entries.push(trimmed.to_string());
        self.cursor = None;

        if let Some(path) = &self.path {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                let _ = writeln!(file, "{trimmed}");
            }
        }
    }

    pub(crate) fn previous(&mut self) -> Option<String> {
        if self.entries.is_empty() {
            return None;
        }

        let new_index = match self.cursor {
            None => self.entries.len().saturating_sub(1),
            Some(0) => 0,
            Some(idx) => idx.saturating_sub(1),
        };
        self.cursor = Some(new_index);
        self.entries.get(new_index).cloned()
    }

    pub(crate) fn next(&mut self) -> Option<String> {
        match self.cursor {
            None => None,
            Some(idx) => {
                if idx + 1 >= self.entries.len() {
                    self.cursor = None;
                    Some(String::new())
                } else {
                    let new_index = idx + 1;
                    self.cursor = Some(new_index);
                    self.entries.get(new_index).cloned()
                }
            }
        }
    }

    pub(crate) fn reset_navigation(&mut self) {
        self.cursor = None;
    }

    fn history_file_path() -> Option<PathBuf> {
        if let Some(custom) = env::var_os("DNTK_HISTORY_FILE") {
            return Some(PathBuf::from(custom));
        }

        #[cfg(target_os = "windows")]
        {
            env::var_os("APPDATA").map(|base| PathBuf::from(base).join("dntk").join("history"))
        }

        #[cfg(not(target_os = "windows"))]
        {
            if let Some(dir) = env::var_os("XDG_CONFIG_HOME") {
                Some(PathBuf::from(dir).join("dntk").join("history"))
            } else {
                env::var_os("HOME").map(|home| {
                    PathBuf::from(home)
                        .join(".config")
                        .join("dntk")
                        .join("history")
                })
            }
        }
    }
}
