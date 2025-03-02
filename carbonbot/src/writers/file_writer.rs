use super::Writer;

use std::{fs, io::Write, sync::Mutex};

use log::*;

pub struct FileWriter {
    file: Mutex<fs::File>,
    path: String,
}

impl FileWriter {
    pub fn new(path: &str) -> Self {
        let file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .unwrap_or_else(|_| panic!("Failed to open {}", path));

        FileWriter {
            file: Mutex::new(file),
            path: path.to_string(),
        }
    }
}

impl Writer for FileWriter {
    fn write(&self, s: &str) {
        if let Err(e) = writeln!(self.file.lock().unwrap(), "{}", s) {
            error!("{}, {}", self.path, e);
        }
    }

    fn close(&self) {
        let mut file = self.file.lock().unwrap();
        if let Err(e) = file.flush() {
            error!("{}, {}", self.path, e);
        }

        if let Err(e) = file.sync_all() {
            error!("{}, {}", self.path, e);
        }
    }
}
