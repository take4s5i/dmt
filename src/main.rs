#![allow(dead_code, unused_macros)]

mod cli;
mod selector;
mod value;

mod prelude {
    pub use crate::cli::*;
    pub use crate::selector::*;
    pub use crate::value::*;
}

use crate::prelude::*;
use structopt::*;
use std::{fs, io, path::PathBuf};

fn get_input(input: Option<&PathBuf>) -> io::Result<Box<dyn io::BufRead>> {
    if let Some(path) = input {
        let file = fs::File::open(path)?;
        Ok(Box::new(io::BufReader::new(file)))
    } else {
        Ok(Box::new(io::BufReader::new(io::stdin())))
    }
}

fn get_output(output: Option<&PathBuf>) -> io::Result<Box<dyn io::Write>> {
    if let Some(path) = output {
        let file = fs::File::open(path)?;
        Ok(Box::new(io::BufWriter::new(file)))
    } else {
        Ok(Box::new(io::BufWriter::new(io::stdout())))
    }
}

fn main() {
    let cmd = &Cmd::from_args();

    let mut input = get_input(cmd.input.as_ref()).unwrap();
    let mut output = get_output(cmd.output.as_ref()).unwrap();

    let content: Value = match cmd.from {
        Format::Auto | Format::Json => serde_json::from_reader(input).unwrap(),
        Format::Yaml => serde_yaml::from_reader(input).unwrap(),
        Format::Toml => {
            let mut s = String::new();
            input.read_to_string(&mut s).unwrap();
            toml::from_str(&s).unwrap()
            },
    };

    let content = if let Some(expr) = &cmd.expr {
        let (s, m) = MatcherChain::parse(&expr).unwrap();
        if !s.is_empty() {
            panic!("malformed expr");
        }
        m.try_match(&content).unwrap()
    } else {
        content
    };

    dbg!(&content);

    match cmd.to {
        Format::Auto | Format::Json => serde_json::to_writer(output, &content).unwrap(),
        Format::Yaml => serde_yaml::to_writer(output, &content).unwrap(),
        Format::Toml => {
            let s = toml::to_string(&content).unwrap();
            output.write_all(s.as_ref()).unwrap();
            },
    }
}
