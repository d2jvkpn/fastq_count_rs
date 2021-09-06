use std::process;

use chrono::prelude::*;
use clap::{App, Arg, Values};

use fastq_count_rs::read_input;
mod fq_count;
use fq_count::{base, count2};

#[macro_use]
extern crate serde_derive;

// https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

#[derive(Debug)]
struct Config<'a> {
    inputs: Vec<&'a str>,
    phred: u8,
    output: &'a str,
    json_fmt: bool,
    debug: bool,
}

fn main() {
    //##
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
                .default_value("33")
                .help("phred value"),
        )
        .arg(
            Arg::with_name("output")
                .long("output")
                .takes_value(true)
                .required(false)
                .default_value("")
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
        inputs: matches.values_of("inputs").map(Values::collect).unwrap_or_else(|| vec![]),
        phred: matches.value_of("phred").unwrap().parse::<u8>().unwrap(),
        output: matches.value_of("output").unwrap(),
        json_fmt: matches.is_present("json"),
        debug: matches.is_present("debug"),
    };

    if config.debug {
        dbg!(&config);
    }

    //##
    let mut fqc = base::FQCount::new(config.phred);
    let start: DateTime<Local> = Local::now();

    for input in config.inputs {
        let local: DateTime<Local> = Local::now();

        eprintln!(
            "{} fastq count read input: \"{}\"",
            local.to_rfc3339_opts(SecondsFormat::Millis, true),
            input
        );

        match read_input(&input) {
            Ok(buf_read) => match count2::read(buf_read, config.phred) {
                Ok(y) => fqc.add(y),
                Err(err) => panic!("fq_count::count2::read {}: {:?}", input, err),
            },
            Err(err) => {
                eprintln!("read_input {}: {:?}", input, err);
                process::exit(1);
            }
        };

        let log_elapsed = || {
            let end: DateTime<Local> = Local::now();
            eprintln!(
                "{} fastq count elapsed: {:?}",
                end.to_rfc3339_opts(SecondsFormat::Millis, true),
                end.signed_duration_since(start).to_std().unwrap(),
            );
        };

        log_elapsed();
        fqc.output(config.output, config.json_fmt).unwrap();
        // println!("{}", fqc);
    }
}
