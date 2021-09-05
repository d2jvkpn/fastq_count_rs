use std::io::prelude::*;
use std::{error, fmt, fs, io, sync, thread};

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FQCount {
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
    pub fn new(phred: u8) -> FQCount {
        FQCount {
            phred: phred,
            ..Default::default()
        }
    }

    fn calc_reads(&self) -> f64 {
        self.reads as f64 / 1e6
    }

    fn calc_bases(&self) -> f64 {
        self.bases as f64 / 1e9
    }

    fn calc_n(&self) -> f64 {
        if self.bases == 0 {
            return 0.0;
        }
        (self.n * 100_000 / self.bases) as f64 / 1e3
    }

    fn calc_gc(&self) -> f64 {
        if self.bases == 0 {
            return 0.0;
        }
        (self.gc * 100_000 / self.bases) as f64 / 1e3
    }

    fn calc_q20(&self) -> f64 {
        if self.bases == 0 {
            return 0.0;
        }
        (self.q20 * 100_000 / self.bases) as f64 / 1e3
    }

    fn calc_q30(&self) -> f64 {
        if self.bases == 0 {
            return 0.0;
        }
        (self.q30 * 100_000 / self.bases) as f64 / 1e3
    }

    pub fn percs(&mut self) {
        if self.bases == 0 {
            return;
        }

        self.reads_mb = self.calc_reads();
        self.bases_gb = self.calc_bases();
        self.n_perc = self.calc_n();
        self.gc_perc = self.calc_gc();
        self.q20_perc = self.calc_q20();
        self.q30_perc = self.calc_q30();
    }

    pub fn add(&mut self, inst: FQCount) {
        self.reads += inst.reads;
        self.bases += inst.bases;
        self.n += inst.n;
        self.gc += inst.gc;
        self.q20 += inst.q20;
        self.q30 += inst.q30;
    }

    fn json(&mut self) -> String {
        self.percs();
        serde_json::to_string(&self).unwrap_or(String::from(""))
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

    pub fn output(&mut self, output: &str, json_fmt: bool) -> Result<(), io::Error> {
        let result = if json_fmt { self.json() } else { self.text() };

        if output == "" {
            println!("{}", result);
            return Ok(());
        }

        let mut file = fs::File::create(output)?;
        writeln!(file, "{}", result)?;
        return Ok(());
    }
}

impl FQCount {
    fn countb(&mut self, line: String) {
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

    fn countq(&mut self, line: String) {
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
}

impl fmt::Display for FQCount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Reads: {:.2}MB, Bases: {:.2}GB, N-bases: {:.2}%, GC: {:.2}%, Q20: {:.2}%, Q30: {:.2}%",
            self.calc_reads(),
            self.calc_bases(),
            self.calc_n(),
            self.calc_gc(),
            self.calc_q20(),
            self.calc_q30(),
        )
    }
}

pub fn read(reader: Box<dyn BufRead>, phred: u8) -> Result<FQCount, Box<dyn error::Error>> {
    let (tx1, rx1) = sync::mpsc::channel();
    let (tx2, rx2) = sync::mpsc::channel();

    let th1 = thread::spawn(move || -> Result<FQCount, io::Error> {
        let mut fqc = FQCount::new(phred);
        for line in rx1 {
            fqc.countb(line);
        }
        return Ok(fqc);
    });

    let th2 = thread::spawn(move || -> Result<FQCount, io::Error> {
        let mut fqc = FQCount::new(phred);
        for line in rx2 {
            fqc.countq(line);
        }
        return Ok(fqc);
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

    // https://stackoverflow.com/questions/56535634/propagating-errors-from-within-a-closure-in-a-thread-in-rust
    let mut fqc = th1.join().unwrap()?;
    let fqc2 = th2.join().unwrap()?;
    fqc.add(fqc2);

    return Ok(fqc);
}
