use std::io::{self, prelude::*};
use std::{fmt, fs};

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FQCount {
    pub phred: u8, // phred value

    pub reads: u64, // reads number
    pub bases: u64, // bases number
    pub n: u64,     // base N number
    pub gc: u64,    // base GC number
    pub q20: u64,   // Q20 number
    pub q30: u64,   // Q30 number

    pub reads_mb: f64,
    pub bases_gb: f64,
    pub n_perc: f64,
    pub gc_perc: f64,  // GC percentage
    pub q20_perc: f64, // Q20 percentage
    pub q30_perc: f64, // Q30 percentage
}

// basic
impl FQCount {
    pub fn new(phred: u8) -> FQCount {
        FQCount { phred: phred, ..Default::default() }
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
