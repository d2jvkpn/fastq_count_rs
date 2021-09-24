use assert_cmd::Command;

#[test]
fn read_fastq() {
    let mut cmd = Command::cargo_bin("fastq_count").unwrap();
    cmd.arg("examples/Sample.fastq").arg("examples/Sample.fastq.gz").assert().success();
}
