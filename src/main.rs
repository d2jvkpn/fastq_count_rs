use clap::{App, Arg};
use flate2::bufread::GzDecoder;
use std::fs::File;
use std::io;
use std::io::prelude::*;

fn main() {
    let args = App::new("fastq count in rust")
        .version("0.1")
        .about("fastq count  reads, bases, N Bases, Q20, Q30, GC")
        .arg(
            Arg::with_name("input")
                .long("input")
                .help("input fastq file")
                .takes_value(true)
                // .multiple(true)
                .required(true),
        )
        .arg(
            Arg::with_name("phred")
                .long("phred")
                .help("input fastq file")
                .takes_value(true)
                .default_value("33")
                .required(false),
        )
        .get_matches();

    let phred = args.value_of("phred").unwrap();
    let phred = phred.parse::<u8>().unwrap();
    let input = args.value_of("input").unwrap();

    let result = calculate(input, phred);
    println!("{:?}", result);
}

#[derive(Debug)]
struct FqResult {
    reads: u64, // reads number
    bases: u64, // bases number
    n: u64,     // base N number
    q20: u64,   // Q20 number
    q30: u64,   // Q30 number
    gc: u64,    // base GC number
}

impl FqResult {
    fn new() -> FqResult {
        FqResult {
            reads: 0,
            bases: 0,
            n: 0,
            q20: 0,
            q30: 0,
            gc: 0,
        }
    }
}

fn calculate(input: &str, phred: u8) -> FqResult {
    let f = File::open(input).unwrap();

    // let reader = io::BufReader::new(f);
    let reader = io::BufReader::new(GzDecoder::new(io::BufReader::new(f)));
    let mut result = FqResult::new();

    for (num, line_) in reader.lines().enumerate() {
        let line = line_.unwrap();
        // println!("{}", num);

        match num % 4 {
            0 => result.reads += 1,
            1 => {
                result.bases += line.len() as u64;
                let upper_line = line.to_ascii_uppercase();

                for v in upper_line.chars() {
                    if v == 'N' {
                        result.n += 1;
                    }
                    if v == 'G' || v == 'C' {
                        result.gc += 1;
                    }
                }
            }
            3 => {
                for v in line.as_bytes() {
                    let q = *v as u8 - phred;

                    if q < 20 {
                        continue;
                    }
                    result.q20 += 1;

                    if q >= 30 {
                        result.q30 += 1;
                    }
                }
            }
            _ => {}
        }
    }

    result
}
