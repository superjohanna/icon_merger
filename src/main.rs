use std::{collections::HashMap, io::Read};

use csv::CSV;
use svg::lexer::Tokenizer;

pub mod csv;
pub mod svg;

#[allow(dead_code)]
fn main() {
    let mut csv = CSV::new("./file.csv");
    let mut buf = String::new();

    let mut file = std::fs::File::open("./file.svg").unwrap();
    let mut tokenizer = Tokenizer::new(std::io::BufReader::new(file));
    let result = tokenizer.tokenize();
    if let Ok(t) = result {
        for token in t {
            println!("{}", token);
        }
    } else if let Err(e) = result {
        println!("{:?}", e);
    }
}
