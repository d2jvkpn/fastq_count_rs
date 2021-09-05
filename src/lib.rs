use std::io::prelude::*;
use std::{error, fs, io};

use flate2::bufread::GzDecoder;

pub fn read_input(input: &str) -> Result<Box<dyn BufRead>, Box<dyn error::Error>> {
    if input == "-" {
        return Ok(Box::new(io::BufReader::new(io::stdin())));
    }

    let file = fs::File::open(input)?;
    let reader = io::BufReader::new(file);

    /*
    if input.ends_with(".gz") {
        return Ok(Box::new(io::BufReader::new(GzDecoder::new(reader))));
    }

    return Ok(Box::new(reader));
    */

    match input {
        input if input.ends_with(".gz") => Ok(Box::new(io::BufReader::new(GzDecoder::new(reader)))),
        _ => Ok(Box::new(reader)),
    }
}
