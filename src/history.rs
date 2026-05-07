use std::collections::vec_deque;
use std::fs::{read_to_string, OpenOptions};
use std::io::Write;
use std::ops::Index;

use rustyline::history::{History, MemHistory};
use rustyline::Config;

#[derive(Default)]
pub struct MyHistory {
    mem: MemHistory,
    new_entries: usize,
}

impl MyHistory {
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(&Config::default())
    }

    #[must_use]
    pub fn with_config(config: &Config) -> Self {
        Self {
            mem: MemHistory::with_config(config),
            new_entries: 0,
        }
    }
}

impl History for MyHistory {
    fn get(
        &self,
        index: usize,
        dir: rustyline::history::SearchDirection,
    ) -> rustyline::Result<Option<rustyline::history::SearchResult<'_>>> {
        self.mem.get(index, dir)
    }

    fn add(&mut self, line: &str) -> rustyline::Result<bool> {
        if self.mem.add(line)? {
            self.new_entries = self.new_entries.saturating_add(1).min(self.len());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn add_owned(&mut self, line: String) -> rustyline::Result<bool> {
        if self.mem.add_owned(line)? {
            self.new_entries = self.new_entries.saturating_add(1).min(self.len());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn len(&self) -> usize {
        self.mem.len()
    }

    fn is_empty(&self) -> bool {
        self.mem.is_empty()
    }

    fn set_max_len(&mut self, len: usize) -> rustyline::Result<()> {
        self.mem.set_max_len(len)?;
        self.new_entries = self.new_entries.min(len);
        Ok(())
    }

    fn ignore_dups(&mut self, yes: bool) -> rustyline::Result<()> {
        self.mem.ignore_dups(yes)
    }

    fn ignore_space(&mut self, yes: bool) {
        self.mem.ignore_space(yes)
    }

    fn save(&mut self, path: &std::path::Path) -> rustyline::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        for line in self.mem.into_iter() {
            writeln!(file, "{line}")?;
        }
        self.new_entries = 0;
        Ok(())
    }

    fn append(&mut self, path: &std::path::Path) -> rustyline::Result<()> {
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        let first_new_entry = self.mem.len().saturating_sub(self.new_entries);
        for line in self.mem.into_iter().skip(first_new_entry) {
            writeln!(file, "{line}")?;
        }
        self.new_entries = 0;
        Ok(())
    }

    fn load(&mut self, path: &std::path::Path) -> rustyline::Result<()> {
        let content = read_to_string(path)?;
        for line in content.lines() {
            if !line.is_empty() {
                self.mem.add(line)?;
            }
        }
        Ok(())
    }

    fn clear(&mut self) -> rustyline::Result<()> {
        self.mem.clear()?;
        self.new_entries = 0;
        Ok(())
    }

    fn search(
        &self,
        term: &str,
        start: usize,
        dir: rustyline::history::SearchDirection,
    ) -> rustyline::Result<Option<rustyline::history::SearchResult<'_>>> {
        self.mem.search(term, start, dir)
    }

    fn starts_with(
        &self,
        term: &str,
        start: usize,
        dir: rustyline::history::SearchDirection,
    ) -> rustyline::Result<Option<rustyline::history::SearchResult<'_>>> {
        self.mem.starts_with(term, start, dir)
    }
}

impl Index<usize> for MyHistory {
    type Output = String;

    fn index(&self, index: usize) -> &String {
        self.mem.index(index)
    }
}

impl<'a> IntoIterator for &'a MyHistory {
    type IntoIter = vec_deque::Iter<'a, String>;
    type Item = &'a String;

    fn into_iter(self) -> Self::IntoIter {
        self.mem.into_iter()
    }
}
