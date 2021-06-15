use std::fs::File;
use std::io;
use std::io::prelude::*;

use clap::{App, Arg};
use flate2::bufread::GzDecoder;

// https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

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
        .help("phred value")
        .takes_value(true)
        .default_value("33")
        .required(false);

    let args = App::new("fastq count in rust")
        .author(AUTHORS)
        .version(VERSION)
        .about("fastq count reads, bases, N Bases, Q20, Q30, GC")
        .set_term_width(100)
        .arg(input_arg)
        .arg(phred_arg)
        .get_matches();

    let phred = args.value_of("phred").unwrap().parse::<u8>().unwrap();
    let inputs = args.values_of("input").unwrap();

    //##
    let mut fqc = FQCount::new(phred);

    for input in inputs {
        println!(">>> fastq_count_rs reading \"{}\"", input);
        // let out = calculate(input, phred).unwrap_or_else(|err| { panic!("{:?}", err) });
        let out = match calculate(input, phred) {
            Ok(out) => out,
            Err(err) => {
                fqc.print();
                println!("!!! reading file {}: {:?}", input, err);
                std::process::exit(1);
            }
        };

        fqc.add(out);
    }

    //##
    fqc.print();
}

#[derive(Debug)]
struct FQCount {
    reads: u64, // reads number
    bases: u64, // bases number
    n: u64,     // base N number
    gc: u64,    // base GC number
    q20: u64,   // Q20 number
    q30: u64,   // Q30 number

    phred: u8,    // phred value
    q20perc: f64, // Q20 number percentage
    q30perc: f64, // Q30 number percentage
}

impl FQCount {
    fn new(phred: u8) -> FQCount {
        FQCount {
            reads: 0,
            bases: 0,
            n: 0,
            q20: 0,
            q30: 0,
            gc: 0,
            phred: phred,
            q20perc: 0.0,
            q30perc: 0.0,
        }
    }

    fn countb(&mut self, line: &str) {
        self.bases += line.len() as u64;

        for v in line.to_ascii_uppercase().chars() {
            if v == 'G' || v == 'C' {
                self.gc += 1;
            } else if v == 'N' {
                self.n += 1;
            }
        }
    }

    fn countq(&mut self, line: &str) {
        for v in line.as_bytes() {
            let q = *v as u8 - self.phred;

            if q < 20 {
                continue;
            }
            self.q20 += 1;

            if q >= 30 {
                self.q30 += 1;
            }
        }
    }

    fn percs(&mut self) {
        if self.bases == 0 {
            return;
        }

        self.q20perc = (self.q20 * 1000 / self.bases) as f64 / 1000.0;
        self.q30perc = (self.q30 * 1000 / self.bases) as f64 / 1000.0;
    }

    fn print(&mut self) {
        self.percs();
        println!("{:?}", self);
    }

    fn add(&mut self, inst: FQCount) {
        self.reads += inst.reads;
        self.bases += inst.bases;
        self.n += inst.n;
        self.gc += inst.gc;
        self.q20 += inst.q20;
        self.q30 += inst.q30;
    }
}

fn calculate(input: &str, phred: u8) -> Result<FQCount, std::io::Error> {
    let f = match File::open(input) {
        Ok(file) => file,
        Err(e) => return Err(e),
    };

    // let reader = io::BufReader::new(f);
    let reader = io::BufReader::new(GzDecoder::new(io::BufReader::new(f)));
    let mut fqc = FQCount::new(phred);

    for (num, line_) in reader.lines().enumerate() {
        // let line = line_.unwrap();
        let line = match line_ {
            Ok(line) => line,
            Err(e) => return Err(e),
        };

        match num % 4 {
            0 => fqc.reads += 1,
            1 => fqc.countb(&line),
            3 => fqc.countq(&line),
            _ => {}
        }
    }

    Ok(fqc)
}
