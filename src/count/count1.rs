#![allow(dead_code)]

use std::fs;
use std::io::{self, prelude::*};

use flate2::bufread::GzDecoder;

use super::base;

impl base::FQCount {
    fn countb1(&mut self, line: &str) {
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

    fn countq1(&mut self, line: &str) {
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

    fn read<R: BufRead>(&mut self, reader: R) -> Option<io::Error> {
        for (num, result) in reader.lines().enumerate() {
            let line = match result {
                Ok(v) => v,
                Err(e) => return Some(e),
            };

            match num % 4 {
                1 => self.countb1(&line),
                3 => self.countq1(&line),
                _ => {}
            }
        }

        return None;
    }
}

pub fn read(input: &str, fqc: &mut base::FQCount) -> Option<io::Error> {
    // Option<Box<dyn std::error::Error>>
    eprintln!(">>> fastq count reading \"{}\"", input);

    if input == "-" {
        let stdin = io::stdin();
        let handle = stdin.lock();
        fqc.read(handle)?;
        return None;
    }

    let file = match fs::File::open(input) {
        Ok(v) => v,
        Err(e) => return Some(e),
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
