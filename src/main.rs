use clap::{App, Arg};
use flate2::bufread::GzDecoder;
use std::fs::File;
use std::io;
use std::io::prelude::*;

fn main() {
    //##
    let input_arg = Arg::with_name("input")
        .long("input")
        .help("input fastq file")
        .multiple(true)
        .takes_value(true)
        .required(true);

    let phred_arg = Arg::with_name("phred")
        .long("phred")
        .help("input fastq file")
        .takes_value(true)
        .default_value("33")
        .required(false);

    let args = App::new("fastq count in rust")
        .author("d2jvkpn")
        .version("0.1")
        .about("fastq count reads, bases, N Bases, Q20, Q30, GC")
        .set_term_width(100)
        .arg(input_arg)
        .arg(phred_arg)
        .get_matches();

    let phred = args.value_of("phred").unwrap().parse::<u8>().unwrap();
    let inputs = args.values_of("input").unwrap();

    //##
    let mut result = FQCResult::new();

    for input in inputs {
        let out = calculate(input, phred).unwrap_or_else(|error| {
            panic!("{:?}", error);
        });

        result.add(out);
    }

    //##
    println!("{:?}", result);
}

#[derive(Debug)]
struct FQCResult {
    reads: u64, // reads number
    bases: u64, // bases number
    n: u64,     // base N number
    gc: u64,    // base GC number
    q20: u64,   // Q20 number
    q30: u64,   // Q30 number
}

impl FQCResult {
    fn new() -> FQCResult {
        FQCResult {
            reads: 0,
            bases: 0,
            n: 0,
            q20: 0,
            q30: 0,
            gc: 0,
        }
    }

    fn add(&mut self, inst: FQCResult) {
        self.reads += inst.reads;
        self.bases += inst.bases;
        self.n += inst.n;
        self.gc += inst.gc;
        self.q20 += inst.q20;
        self.q30 += inst.q30;
    }
}

fn calculate(input: &str, phred: u8) -> Result<FQCResult, std::io::Error> {
    let f = match File::open(input) {
        Ok(file) => file,
        Err(e) => return Err(e),
    };

    // let reader = io::BufReader::new(f);
    let reader = io::BufReader::new(GzDecoder::new(io::BufReader::new(f)));
    let mut result = FQCResult::new();

    for (num, line_) in reader.lines().enumerate() {
        // let line = line_.unwrap();
        let line = match line_ {
            Ok(line) => line,
            Err(e) => return Err(e),
        };

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

    Ok(result)
}
