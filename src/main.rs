use std::process;

use chrono::prelude::*;

use fastq_count::{get_args, read_input};
mod count;
use count::{base, count2};

#[macro_use]
extern crate serde_derive;

fn main() {
    //##
    let config = get_args().unwrap();
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
                Err(err) => panic!("count::count2::read {}: {:?}", input, err),
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
        fqc.output(&config.output, config.json_fmt).unwrap();
        // println!("{}", fqc);
    }
}
