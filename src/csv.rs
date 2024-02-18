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
        let mut start_x = String::new();
        let mut start_y = String::new();
        let mut end_x = String::new();
        let mut end_y = String::new();
        let mut value = Vec::<String>::new();
        let mut string_buf = String::new();
        for c in buf.chars() {
            match c {
                ',' => {
                    if key.is_empty() {
                        key = string_buf;
                        string_buf = String::new();
                        continue;
                    } else if start_x.is_empty() {
                        start_x = string_buf;
                        string_buf = String::new();
                        continue;
                    } else if start_y.is_empty() {
                        start_y = string_buf;
                        string_buf = String::new();
                        continue;
                    } else if end_x.is_empty() {
                        end_x = string_buf;
                        string_buf = String::new();
                        continue;
                    } else if end_y.is_empty() {
                        end_y = string_buf;
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
            } else if start_x.is_empty() {
                start_x = string_buf;
            } else if start_y.is_empty() {
                start_y = string_buf;
            } else if end_x.is_empty() {
                end_x = string_buf;
            } else if end_y.is_empty() {
                end_y = string_buf;
            } else {
                value.push(string_buf);
            }
        }

        // Parsing
        let start_x = start_x
            .parse()
            .unwrap_or_else(|_| panic!("Couldn't parse {0}", start_x));
        let start_y = start_y
            .parse()
            .unwrap_or_else(|_| panic!("Couldn't parse {0}", start_y));
        let end_x = end_x.parse().unwrap_or_else(|_| panic!("Couldn't parse {0}", end_x));
        let end_y = end_y.parse().unwrap_or_else(|_| panic!("Couldn't parse {0}", end_y));

        Some(Batch {
            key,
            start_x,
            start_y,
            end_x,
            end_y,
            values: value,
        })
    }
}

#[derive(Debug)]
pub struct Batch {
    key: String,
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
    values: Vec<String>,
}
