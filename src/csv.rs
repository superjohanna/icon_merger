use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CSV {
    buf_reader: BufReader<std::fs::File>,
}

impl CSV {
    pub fn new(path: &str) -> Self {
        Self {
            buf_reader: BufReader::new(File::open(path).expect("Couldn't open file")),
        }
    }
}

impl std::iter::Iterator for CSV {
    type Item = Batch;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = String::new();
        if self.buf_reader.read_line(&mut buf).is_err() {
            return None;
        }

        // Loop variables
        let mut key = String::new();
        let mut value = Vec::<String>::new();
        let mut string_buf = String::new();
        for c in buf.chars() {
            match c {
                ',' => {
                    if key.is_empty() {
                        key = string_buf;
                        string_buf = String::new();
                        continue;
                    }
                    value.push(string_buf);
                    string_buf = String::new();
                }
                c if c.is_whitespace() => continue,
                _ => string_buf += &c.to_string(),
            }
        }

        // Check if buffer is full
        if !string_buf.is_empty() {
            if key.is_empty() {
                key = string_buf;
            } else {
                value.push(string_buf);
            }
        }

        if !key.is_empty() {
            Some(Batch { key, values: value })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Batch {
    pub key: String,
    pub values: Vec<String>,
}
