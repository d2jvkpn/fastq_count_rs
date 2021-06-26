use std::io::prelude::*;
use std::{error, fs, io, process, sync, thread};

#[macro_use]
extern crate serde_derive;

use clap::{App, Arg};
use flate2::bufread::GzDecoder;

// https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

fn main() {
    //##
    let input_arg = Arg::with_name("input")
        .long("input")
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

    let json_format_arg = Arg::with_name("json_format")
        .long("json_format")
        .takes_value(false)
        .required(false)
        .help("output json format");

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
        .arg(input_arg)
        .arg(phred_arg)
        .arg(json_format_arg)
        .arg(output_arg)
        .get_matches();

    let phred = args.value_of("phred").unwrap().parse::<u8>().unwrap();
    let inputs = args.values_of("input").unwrap();
    let json_format = args.is_present("json_format");
    let output = args.value_of("output").unwrap();

    //##
    let mut fqc = FQCount::new(phred);

    for input in inputs {
        match calculate2(input, phred) {
            Ok(out) => {
                fqc.add(out);
            }
            Err(err) => {
                eprintln!("{:?}", err);
                process::exit(1);
            }
        };
    }

    //##
    let result = if json_format { fqc.json() } else { fqc.text() };
    if output == "" {
        println!("{}", result);
        return;
    }

    let mut file = fs::File::create(output).unwrap();
    writeln!(file, "{}", result).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FQCount {
    phred: u8, // phred value

    reads: u64, // reads number
    bases: u64, // bases number
    n: u64,     // base N number
    gc: u64,    // base GC number
    q20: u64,   // Q20 number
    q30: u64,   // Q30 number

    reads_mb: f64,
    bases_gb: f64,
    n_perc: f64,
    gc_perc: f64,  // GC percentage
    q20_perc: f64, // Q20 percentage
    q30_perc: f64, // Q30 percentage
}

// basic
impl FQCount {
    fn new(phred: u8) -> FQCount {
        FQCount {
            phred: phred,

            reads: 0,
            bases: 0,
            n: 0,
            gc: 0,
            q20: 0,
            q30: 0,

            reads_mb: 0.0,
            bases_gb: 0.0,
            n_perc: 0.0,
            gc_perc: 0.0,
            q20_perc: 0.0,
            q30_perc: 0.0,
        }
    }

    fn percs(&mut self) {
        if self.bases == 0 {
            return;
        }

        self.reads_mb = self.reads as f64 / 1e6;
        self.bases_gb = self.bases as f64 / 1e9;
        self.n_perc = (self.n * 100_000 / self.bases) as f64 / 1e3;
        self.gc_perc = (self.gc * 100_000 / self.bases) as f64 / 1e3;
        self.q20_perc = (self.q20 * 100_000 / self.bases) as f64 / 1e3;
        self.q30_perc = (self.q30 * 100_000 / self.bases) as f64 / 1e3;
    }

    fn add(&mut self, inst: FQCount) {
        self.reads += inst.reads;
        self.bases += inst.bases;
        self.n += inst.n;
        self.gc += inst.gc;
        self.q20 += inst.q20;
        self.q30 += inst.q30;
    }

    fn text(&mut self) -> String {
        self.percs();
        format!(
            "Reads\tBases\tN-bases\tGC\tQ20\tQ30
{:.2}MB\t{:.2}GB\t{:.2}%\t{:.2}%\t{:.2}%\t{:.2}%
{}\t{}\t{}\t{}\t{}\t{}",
            self.reads_mb,
            self.bases_gb,
            self.n_perc,
            self.gc_perc,
            self.q20_perc,
            self.q30_perc,
            self.reads,
            self.bases,
            self.n,
            self.gc,
            self.q20,
            self.q30,
        )
    }

    fn json(&mut self) -> String {
        self.percs();
        serde_json::to_string(&self).unwrap_or(String::from(""))
    }
}

// classic way
impl FQCount {
    #[allow(dead_code)]
    fn countb(&mut self, line: &str) {
        self.reads += 1;
        self.bases += line.len() as u64;

        for v in line.to_ascii_uppercase().chars() {
            if v == 'G' || v == 'C' {
                self.gc += 1;
            } else if v == 'N' {
                self.n += 1;
            }
        }
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    fn read<R: BufRead>(&mut self, reader: R) -> Option<io::Error> {
        for (num, line) in reader.lines().enumerate() {
            let line = match line {
                Ok(line) => line,
                Err(err) => return Some(err),
            };

            match num % 4 {
                1 => self.countb(&line),
                3 => self.countq(&line),
                _ => {}
            }
        }

        return None;
    }
}

#[allow(dead_code)]
fn calculate(input: &str, fqc: &mut FQCount) -> Option<io::Error> {
    // Option<Box<dyn std::error::Error>>
    eprintln!(">>> fastq count reading \"{}\"", input);

    if input == "-" {
        let stdin = io::stdin();
        let handle = stdin.lock();
        fqc.read(handle)?;
        return None;
    }

    let file = match fs::File::open(input) {
        Ok(f) => f,
        Err(e) => return Some(e), // Some(Box::new(e))
    };

    if input.ends_with(".gz") {
        let reader = io::BufReader::new(GzDecoder::new(io::BufReader::new(file)));
        fqc.read(reader)?;
    } else {
        let reader = io::BufReader::new(file);
        fqc.read(reader)?;
    }

    return None;
}

// multithreading with channels
impl FQCount {
    fn countb2(&mut self, line: String) {
        self.reads += 1;
        self.bases += line.len() as u64;

        for v in line.to_ascii_uppercase().chars() {
            match v {
                'G' | 'C' => self.gc += 1,
                'N' => self.n += 1,
                _ => {}
            }
        }
    }

    fn countq2(&mut self, line: String) {
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

    fn from_reader<R: BufRead>(reader: R, phred: u8) -> Result<FQCount, Box<dyn error::Error>> {
        let (tx1, rx1) = sync::mpsc::channel();
        let (tx2, rx2) = sync::mpsc::channel();

        let th1 = thread::spawn(move || {
            let mut fqc = FQCount::new(phred);
            for line in rx1 {
                fqc.countb2(line);
            }
            fqc
        });

        let th2 = thread::spawn(move || {
            let mut fqc = FQCount::new(phred);
            for line in rx2 {
                fqc.countq2(line);
            }
            fqc
        });

        for (num, line) in reader.lines().enumerate() {
            let line = match line {
                Ok(line) => line,
                Err(err) => return Err(Box::new(err)),
            };

            match num % 4 {
                1 => tx1.send(line)?,
                3 => tx2.send(line)?,
                _ => continue,
            }
        }
        drop(tx1);
        drop(tx2);

        let mut fqc = th1.join().unwrap(); //?? handle error
        let fqc2 = th2.join().unwrap(); //?? handle error
        fqc.add(fqc2);

        return Ok(fqc);
    }
}

fn calculate2(input: &str, phred: u8) -> Result<FQCount, Box<dyn error::Error>> {
    eprintln!(">>> fastq count reading \"{}\"", input);

    if input == "-" {
        let stdin = io::stdin();
        let handle = stdin.lock();
        FQCount::from_reader(handle, phred)?;
    }

    let file = match fs::File::open(input) {
        Ok(file) => file,
        Err(e) => return Err(Box::new(e)),
    };

    if input.ends_with(".gz") {
        let reader = io::BufReader::new(GzDecoder::new(io::BufReader::new(file)));
        let fqc = FQCount::from_reader(reader, phred)?;
        return Ok(fqc);
    }

    let reader = io::BufReader::new(file);
    let fqc = FQCount::from_reader(reader, phred)?;
    return Ok(fqc);
}
