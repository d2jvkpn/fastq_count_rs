use std::process;

use fastq_count::{get_args, run};

fn main() {
    // if let Err(err) = get_args().and_then(run) { // "an_then" is just like "cmd1 && cmd2" in shell
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

    if let Err(err) = run(config) {
        eprintln!("run: {}", err);
        process::exit(1);
    }
}
