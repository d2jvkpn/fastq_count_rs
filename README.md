### fastq_count_rs

Implemented fastq count (https://github.com/d2jvkpn/fastq_count) in Rust.

#### 1 build and run
```bash
cargo build --release

gunzip -c examples/Sample.fastq.gz |
  ./target/release/fastq_count_rs  \
  --output output.txt --json       \
  examples/Sample.fastq examples/Sample.fastq.gz - 
```
