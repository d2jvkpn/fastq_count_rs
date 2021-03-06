#![allow(dead_code)]

use std::io::{self, prelude::*};
use std::{error, sync, thread};

use super::base;

impl base::FQCount {
    pub fn countb2(&mut self, line: String) {
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

    pub fn countq2(&mut self, line: String) {
        for v in line.as_bytes() {
            let q = match *v as u8 {
                q if q >= self.phred => q - self.phred,
                _ => 0, // ignore unexpected value
            };

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
    pub fn count(&mut self, bl: String, ql: String) {
        self.countb2(bl);
        self.countq2(ql);
    }
}

pub fn read(reader: Box<dyn BufRead>, phred: u8) -> Result<base::FQCount, Box<dyn error::Error>> {
    let (tx1, rx1) = sync::mpsc::channel();
    let (tx2, rx2) = sync::mpsc::channel();

    let th1 = thread::spawn(move || -> Result<base::FQCount, io::Error> {
        let mut fqc = base::FQCount::new(phred);
        for line in rx1 {
            fqc.countb2(line);
        }
        return Ok(fqc);
    });

    let th2 = thread::spawn(move || -> Result<base::FQCount, io::Error> {
        let mut fqc = base::FQCount::new(phred);
        for line in rx2 {
            fqc.countq2(line);
        }
        return Ok(fqc);
    });

    for (num, result) in reader.lines().enumerate() {
        let line = result?;

        match num % 4 {
            1 => tx1.send(line)?,
            3 => tx2.send(line)?,
            _ => continue,
        }
    }
    drop(tx1);
    drop(tx2);

    // https://stackoverflow.com/questions/56535634/propagating-errors-from-within-a-closure-in-a-thread-in-rust
    let mut fqc = th1.join().unwrap()?;
    let fqc2 = th2.join().unwrap()?;
    fqc.add(fqc2);

    return Ok(fqc);
}
