mod parser;
mod string_composer;
mod buf_read_streamer;

use parser::parse;

use buf_read_streamer::BufReadStreamer;

use std::io::{BufReader, stdin};

fn main() {
    let mut bf = BufReader::new(stdin());
    let mut composer = String::new();
    let mut buf_streamer = BufReadStreamer::new(&mut bf);
    match parse(&mut buf_streamer, &mut composer) {
        Ok(_) => println!("{}", composer),
        Err(error) => println!("Error: {}", error),
    }
}