use std::process;

use fastq_count::{get_args, run_v2}; // run_v1

fn main() {
    // if let Err(err) = get_args().and_then(run) {
    //     eprintln!("{}", err);
    //     process::exit(1);
    // }

    let config = match get_args() {
        Ok(data) => data,
        Err(err) => {
            eprintln!("get_args: {}", err);
            process::exit(1);
        }
    };

    if let Err(err) = run_v2(config) {
        eprintln!("run_v2: {}", err);
        process::exit(1);
    }
}
