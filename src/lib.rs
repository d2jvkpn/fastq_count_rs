#[macro_use]
extern crate serde_derive;

use std::io::prelude::*;
use std::{error, fs, io, time};

mod count;
use count::{base, count1, count2, count3};

use chrono::prelude::*;
use clap::{App, Arg}; // Values
use flate2::bufread::GzDecoder;

#[derive(Debug)]
pub struct Config {
    pub inputs: Vec<String>,
    pub phred: u8,
    pub output: String,
    pub json_fmt: bool,
    pub debug: bool,
    pub run: String,
}

pub fn read_input(input: &str) -> Result<Box<dyn BufRead>, io::Error> {
    if input == "-" {
        return Ok(Box::new(io::BufReader::new(io::stdin())));
    }

    let file = fs::File::open(input)?;
    let reader = io::BufReader::new(file);

    if input.ends_with(".gz") {
        Ok(Box::new(io::BufReader::new(GzDecoder::new(reader))))
    } else {
        Ok(Box::new(reader))
    }
}

pub fn get_args() -> Result<Config, Box<dyn error::Error>> {
    let matches = App::new(env!("CARGO_PKG_HOMEPAGE"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
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
                .help("in debug mode"),
        )
        .arg(
            Arg::with_name("run")
                .long("run")
                .takes_value(true)
                .possible_values(&["v1", "v2"])
                .help("run version"),
        )
        .get_matches();

    let phred = matches
        .value_of("phred")
        .unwrap_or("33")
        .parse::<u8>()
        .map_err(|e| format!("parse arg --phred error: {:?}", e))?;

    let config = Config {
        // <&str>
        // inputs: matches.values_of("inputs").map(Values::collect).unwrap_or_else(|| vec![]),
        // inputs: matches.values_of_lossy("inputs").into_iter().flat_map(|x| x).collect(),
        inputs: matches.values_of_lossy("inputs").unwrap_or(vec![]),
        phred,
        output: matches.value_of("output").unwrap_or("").to_string(),
        json_fmt: matches.is_present("json"),
        debug: matches.is_present("debug"),
        run: matches.value_of("output").unwrap_or("v2").to_string(),
    };

    if config.debug {
        dbg!(&config);
    }

    Ok(config)
}

pub fn run_v1(config: Config) -> Result<(), Box<dyn error::Error>> {
    let mut fqc = base::FQCount::new(config.phred);
    let start: DateTime<Local> = Local::now();

    for input in config.inputs {
        let local: DateTime<Local> = Local::now();

        eprintln!(
            "{} fastq count read input: \"{}\"",
            local.to_rfc3339_opts(SecondsFormat::Millis, true),
            input
        );

        let mut result = base::FQCount::new(config.phred);
        if let Some(e) = count1::read(&input, &mut result) {
            return Err(From::from(format!("read_input {}: {:?}", input, e)));
        }

        let log_elapsed = || {
            let end: DateTime<Local> = Local::now();
            eprintln!(
                "{} ~~~ elapsed: {:?}",
                end.to_rfc3339_opts(SecondsFormat::Millis, true),
                end.signed_duration_since(start).to_std().unwrap_or(time::Duration::new(0, 0)),
            );
        };

        log_elapsed();
    }

    fqc.output(&config.output, config.json_fmt)?;
    return Ok(());
}

pub fn run_v2(config: Config) -> Result<(), Box<dyn error::Error>> {
    let mut fqc = base::FQCount::new(config.phred);
    let start: DateTime<Local> = Local::now();

    for input in config.inputs {
        let local: DateTime<Local> = Local::now();

        eprintln!(
            "{} fastq count read input: \"{}\"",
            local.to_rfc3339_opts(SecondsFormat::Millis, true),
            input
        );

        let reader = read_input(&input).map_err(|e| format!("read_input {}: {:?}", input, e))?;
        let v = count2::read(reader, config.phred)
            .map_err(|e| format!("count2::read {}: {:?}", input, e))?;

        fqc.add(v);

        let log_elapsed = || {
            let end: DateTime<Local> = Local::now();
            eprintln!(
                "{} ~~~ elapsed: {:?}",
                end.to_rfc3339_opts(SecondsFormat::Millis, true),
                end.signed_duration_since(start).to_std().unwrap_or(time::Duration::new(0, 0)),
            );
        };

        log_elapsed();
    }

    fqc.output(&config.output, config.json_fmt)?;
    Ok(())
}

// bad performance with improper async
pub fn run_v3(config: Config) -> Result<(), Box<dyn error::Error>> {
    let mut fqc = base::FQCount::new(config.phred);
    let start: DateTime<Local> = Local::now();

    for input in config.inputs {
        let local: DateTime<Local> = Local::now();

        eprintln!(
            "{} fastq count read input: \"{}\"",
            local.to_rfc3339_opts(SecondsFormat::Millis, true),
            input
        );

        let v = count3::process(&input, config.phred)
            .map_err(|e| format!("count3::process {}: {:?}", input, e))?;

        fqc.add(v);

        let log_elapsed = || {
            let end: DateTime<Local> = Local::now();
            eprintln!(
                "{} ~~~ elapsed: {:?}",
                end.to_rfc3339_opts(SecondsFormat::Millis, true),
                end.signed_duration_since(start).to_std().unwrap_or(time::Duration::new(0, 0)),
            );
        };

        log_elapsed();
    }

    fqc.output(&config.output, config.json_fmt)?;
    Ok(())
}

pub fn run(config: Config) -> Result<(), Box<dyn error::Error>> {
    match &config.run[..] {
        "v1" => run_v1(config),
        "v2" => run_v2(config),
        _ => Err("unkonwm run")?,
    }
}
