use std::{error, sync::mpsc, thread};

use super::base;

use async_std::{
    fs::File,
    io::{self, BufReader},
    prelude::*,
    task,
};

pub fn process(input: &str, phred: u8) -> Result<base::FQCount, Box<dyn error::Error>> {
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();

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

    task::block_on(read_input(input, tx1, tx2))?;

    let mut fqc = th1.join().unwrap()?;
    let fqc2 = th2.join().unwrap()?;
    fqc.add(fqc2);

    Ok(fqc)
}

// read fastq input only
async fn read_input(
    input: &str,
    tx1: mpsc::Sender<String>,
    tx2: mpsc::Sender<String>,
) -> Result<(), Box<dyn error::Error>> {
    let file = File::open(input).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut num = 0;
    while let Some(line) = lines.next().await {
        num += 1;
        let line = line?;

        match num % 4 {
            1 => tx1.send(line)?,
            3 => tx2.send(line)?,
            _ => continue,
        }
    }
    drop(tx1);
    drop(tx2);

    return Ok(());
}
