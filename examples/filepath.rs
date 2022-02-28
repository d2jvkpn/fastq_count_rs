use std::io::Write;
use std::{fs, path};

fn main() {
    let target = "./wk_foo/bar.txt";

    if let Some(d) = path::Path::new(target).parent() {
        fs::create_dir_all(d).unwrap();
    }

    let mut file = fs::File::create(target).unwrap();
    writeln!(file, "{}", "Hello, world!").unwrap();
}
