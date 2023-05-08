extern crate serde_json;

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};

/// Test driver for one-shot benchmark
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// File to benchmark
    #[clap(short, long, value_parser)]
    file: String,

    // /// Mode to compile in
    // /// Options:
    // ///    bdd_topological
    // ///    sdd_right_linear
    // ///    sdd_topological_elim: compile in a topological elimination order
    // ///    sdd_with_vtree: compile with a supplied vtree file
    // #[clap(short, long, value_parser)]
    // mode: String,
    #[clap(long, value_parser, default_value = "./sdd")]
    path_to_sdd: String,

    #[clap(long, value_parser, default_value = "./rsdd")]
    path_to_rsdd: String,
}

/// copied over from rsdd.rs
#[derive(Serialize, Deserialize)]
struct SddBenchmarkLog {
    // name: String,
    compilation_time: f64,
    // sdd_size: usize,
    // sdd_node_count: usize,
}

/// copied over from rsdd.rs
#[derive(Serialize, Deserialize)]
struct RsddBenchmarkLog {
    name: String,
    num_recursive: usize,
    time_in_sec: f64,
    circuit_size: usize,
    mode: String,
}

fn sdd(path_to_sdd: &str, file: &str, mode: &str) -> SddBenchmarkLog {
    let command = Command::new(path_to_sdd)
        .arg("-c")
        .arg(file)
        .arg("-t")
        .arg(mode)
        .arg("-r")
        .arg("0")
        .stdout(Stdio::piped())
        .spawn()
        .expect("rsdd failure");

    let stdout = command.wait_with_output().unwrap().stdout;

    serde_json::from_slice::<SddBenchmarkLog>(&stdout).unwrap()
}

fn rsdd(path_to_rsdd: &str, file: &str, mode: &str) -> RsddBenchmarkLog {
    let command = Command::new(path_to_rsdd)
        .arg("-f")
        .arg(file)
        .arg("-m")
        .arg(mode)
        .stdout(Stdio::piped())
        .spawn()
        .expect("rsdd failure");

    let stdout = command.wait_with_output().unwrap().stdout;

    serde_json::from_slice::<RsddBenchmarkLog>(&stdout).unwrap()
}

fn main() {
    let args = Args::parse();

    let sdd_out = sdd(&args.path_to_sdd, &args.file, "right");
    let rsdd_out = rsdd(&args.path_to_rsdd, &args.file, "sdd_right_linear");

    println!("SDD time: {}", sdd_out.compilation_time);
    println!("RSDD time: {}", rsdd_out.time_in_sec);
}
