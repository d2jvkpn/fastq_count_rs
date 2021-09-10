use std::io::prelude::*;
use std::{error, fs, io};

use clap::{App, Arg}; // Values
use flate2::bufread::GzDecoder;

// https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

#[derive(Debug)]
pub struct Config {
    // inputs: Vec<&'a str>,
    pub inputs: Vec<String>,
    pub phred: u8,
    pub output: String,
    pub json_fmt: bool,
    pub debug: bool,
}

pub fn read_input(input: &str) -> Result<Box<dyn BufRead>, Box<dyn error::Error>> {
    if input == "-" {
        return Ok(Box::new(io::BufReader::new(io::stdin())));
    }

    let file = fs::File::open(input)?;
    let reader = io::BufReader::new(file);

    /*
    if input.ends_with(".gz") {
        return Ok(Box::new(io::BufReader::new(GzDecoder::new(reader))));
    }

    return Ok(Box::new(reader));
    */

    match input {
        input if input.ends_with(".gz") => Ok(Box::new(io::BufReader::new(GzDecoder::new(reader)))),
        _ => Ok(Box::new(reader)),
    }
}

pub fn get_args() -> Result<Config, Box<dyn error::Error>> {
    let matches = App::new("fastq(https://en.wikipedia.org/wiki/FASTQ_format) count in rust")
        .about("count fastq reads, bases, N Bases, Q20, Q30, GC")
        .author(AUTHORS)
        .version(VERSION)
        .set_term_width(100)
        .arg(
            Arg::with_name("inputs")
                .takes_value(true)
                .required(true)
                .multiple(true)
                .help("input fastq, gzipped fastq or stdin(-)"),
        )
        .arg(
            Arg::with_name("phred")
                .long("phred")
                .takes_value(true)
                .required(false)
                // .default_value("33")
                .help("phred value"),
        )
        .arg(
            Arg::with_name("output")
                .long("output")
                .takes_value(true)
                .required(false)
                // .default_value("")
                .help("output to file"),
        )
        .arg(
            Arg::with_name("json")
                .long("json")
                .takes_value(false)
                .required(false)
                .help("output in json format"),
        )
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .takes_value(false)
                .required(false)
                .help("run in debug mode"),
        )
        .get_matches();

    // let inputs = args.values_of("inputs");
    let config = Config {
        // <&str>
        // inputs: matches.values_of("inputs").map(Values::collect).unwrap_or_else(|| vec![]),
        // inputs: matches.values_of_lossy("inputs").into_iter().flat_map(|x| x).collect(),
        inputs: matches.values_of_lossy("inputs").unwrap_or(vec![]),
        phred: matches.value_of("phred").unwrap_or("33").parse::<u8>()?,
        output: matches.value_of("output").unwrap_or("").to_string(),
        json_fmt: matches.is_present("json"),
        debug: matches.is_present("debug"),
    };

    Ok(config)
}
