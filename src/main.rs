use std::process;

use chrono::prelude::*;
use clap::{App, Arg};

mod fq_count;

#[macro_use]
extern crate serde_derive;

// https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

fn main() {
    //##
    let inputs_arg = Arg::with_name("inputs")
        .takes_value(true)
        .required(true)
        .multiple(true)
        .help("input fastq, gzipped fastq or stdin(-)");

    let phred_arg = Arg::with_name("phred")
        .long("phred")
        .takes_value(true)
        .required(false)
        .default_value("33")
        .help("phred value");

    let json_arg = Arg::with_name("json")
        .long("json")
        .takes_value(false)
        .required(false)
        .help("output in json format");

    let output_arg = Arg::with_name("output")
        .long("output")
        .takes_value(true)
        .required(false)
        .default_value("")
        .help("output to file");

    let args = App::new("fastq(https://en.wikipedia.org/wiki/FASTQ_format) count in rust")
        .about("count fastq reads, bases, N Bases, Q20, Q30, GC")
        .author(AUTHORS)
        .version(VERSION)
        .set_term_width(100)
        .arg(inputs_arg)
        .arg(phred_arg)
        .arg(json_arg)
        .arg(output_arg)
        .get_matches();

    let phred = args.value_of("phred").unwrap().parse::<u8>().unwrap();
    let inputs = args.values_of("inputs").unwrap();
    let json_fmt = args.is_present("json");
    let output = args.value_of("output").unwrap();

    //##
    let mut fqc = fq_count::FQCount::new(phred);
    let start: DateTime<Local> = Local::now();

    for input in inputs {
        let local: DateTime<Local> = Local::now();

        eprintln!(
            "{} fastq count input: \"{}\"",
            local.to_rfc3339_opts(SecondsFormat::Millis, true),
            input
        );

        match fastq_count_rs::read_input(input) {
            Ok(buf_read) => match fq_count::read(buf_read, phred) {
                Ok(y) => fqc.add(y),
                Err(err) => panic!("fq_count::read {}: {:?}", input, err),
            },
            Err(err) => {
                eprintln!("read_input {}: {:?}", input, err);
                process::exit(1);
            }
        };

        let log_elapsed = || {
            let end: DateTime<Local> = Local::now();
            // let dura = end.signed_duration_since(start);
            eprintln!(
                "{} fastq count elapsed: {:?}",
                end.to_rfc3339_opts(SecondsFormat::Millis, true),
                end.signed_duration_since(start).to_std().unwrap(),
            );
        };

        log_elapsed();
        fqc.output(output, json_fmt).unwrap();
    }
}
