extern crate serde_json;

use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    fmt::{self, Display},
    fs,
    process::{Command, Stdio},
};

/// Test driver for one-shot benchmark
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value_t = false)]
    debug: bool,

    /// CNF files to benchmark
    #[clap(short, long, value_parser)]
    files: Vec<String>,

    /// Mode to compile in
    #[clap(short, long, value_parser, default_value = "right")]
    mode: String,

    /// File to output JSON to, if any
    #[clap(short, long, value_parser, default_value = "")]
    output: String,

    /// Path to compiled sdd binary (i.e. UCLA CS ARG's sdd)
    #[clap(long, value_parser, default_value = "./sdd")]
    path_to_sdd: String,

    /// Path to compiled rsdd binary
    #[clap(long, value_parser, default_value = "./rsdd")]
    path_to_rsdd: String,
}

// TODO: add BDD, etc.
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, PartialEq, Serialize)]
enum CompilationMode {
    SDDLeftLinear,
    SDDRightLinear,
    // SDDBalanced,
    SDDBestFit, // EvenSplit(usize),
                // FromDTreeLinear,
                // FromDTreeMinFill,
}

impl CompilationMode {
    fn as_libsdd(&self) -> &'static str {
        match &self {
            CompilationMode::SDDLeftLinear => "left",
            CompilationMode::SDDRightLinear => "right",
            // CompilationMode::SDDBalanced => "balanced",
            CompilationMode::SDDBestFit => "right", // TODO: this seems wrong?
        }
    }

    fn as_rsdd(&self) -> &'static str {
        match &self {
            CompilationMode::SDDLeftLinear => "sdd_left_linear",
            CompilationMode::SDDRightLinear => "sdd_right_linear",
            // CompilationMode::SDDBalanced => "linear",
            CompilationMode::SDDBestFit => "sdd_dtree_minfill",
        }
    }
}

impl Display for CompilationMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            CompilationMode::SDDLeftLinear => f.write_str("left linear"),
            CompilationMode::SDDRightLinear => f.write_str("right linear"),
            // CompilationMode::SDDBalanced => f.write_str("balanced"),
            CompilationMode::SDDBestFit => f.write_str("best fit"),
        }
    }
}

/// copied over from rsdd.rs
#[derive(Serialize, Deserialize)]
struct SddBenchmarkLog {
    // name: String,
    compilation_time: f64,
    sdd_size: usize,
    sdd_count: usize,
}

/// copied over from rsdd.rs
#[derive(Serialize, Deserialize)]
struct RsddBenchmarkLog {
    name: String,
    num_recursive: usize,
    time_in_sec: f64,
    circuit_size: usize,
    num_nodes: usize,
    mode: String,
}

#[derive(Serialize)]
struct BenchmarkLog {
    file: String,
    mode: CompilationMode,
    rsdd: RsddBenchmarkLog,
    sdd: SddBenchmarkLog,
}

fn sdd(path_to_sdd: &str, file: &str, mode: &CompilationMode, debug: bool) -> SddBenchmarkLog {
    let mut command = Command::new(path_to_sdd);

    command
        .arg("-c")
        .arg(file)
        .arg("-t")
        .arg(mode.as_libsdd())
        .stdout(Stdio::piped());

    if (*mode) != CompilationMode::SDDBestFit {
        command.arg("-r").arg("0");
    }

    if !debug {
        command.stderr(Stdio::null());
    }

    let command = command.spawn().expect("libsdd failure");

    let stdout = command.wait_with_output().unwrap().stdout;

    serde_json::from_slice::<SddBenchmarkLog>(&stdout).unwrap()
}

fn rsdd(path_to_rsdd: &str, file: &str, mode: &CompilationMode, debug: bool) -> RsddBenchmarkLog {
    let mut command = Command::new(path_to_rsdd);

    command
        .arg("-f")
        .arg(file)
        .arg("-m")
        .arg(mode.as_rsdd())
        .stdout(Stdio::piped());

    if !debug {
        command.stderr(Stdio::null());
    }

    let command = command.spawn().expect("rsdd failure");

    let stdout = command.wait_with_output().unwrap().stdout;

    serde_json::from_slice::<RsddBenchmarkLog>(&stdout).unwrap()
}

fn benchmark(args: &Args, mode: &CompilationMode) -> Vec<BenchmarkLog> {
    args.files
        .iter()
        .map(|file| BenchmarkLog {
            file: file.to_string(),
            mode: *mode,
            rsdd: rsdd(&args.path_to_rsdd, file, mode, args.debug),
            sdd: sdd(&args.path_to_sdd, file, mode, args.debug),
        })
        .collect()
}

fn str_to_mode(str: &str) -> CompilationMode {
    match str {
        "left" => CompilationMode::SDDLeftLinear,
        // "balanced" => CompilationMode::SDDBalanced,
        "best" => CompilationMode::SDDBestFit,
        _ => CompilationMode::SDDRightLinear,
    }
}

fn main() {
    let args = Args::parse();

    let mode = str_to_mode(&args.mode);

    let benches = benchmark(&args, &mode);

    for bench in benches.iter() {
        println!("Compiling {} with vtree strategy {}", bench.file, mode);
        println!(
            "{:.2}x speedup (rsdd: {:.6}s, sdd: {:.6}s)",
            bench.sdd.compilation_time / bench.rsdd.time_in_sec,
            bench.rsdd.time_in_sec,
            bench.sdd.compilation_time
        );
        println!(
            "{:.2}x circuit size (rsdd: {}, sdd: {})",
            bench.rsdd.circuit_size as f64 / bench.sdd.sdd_size as f64,
            bench.rsdd.circuit_size,
            bench.sdd.sdd_size
        );
        println!(
            "{:.2}x alloc nodes (rsdd: {}, sdd: {})",
            bench.rsdd.num_nodes as f64 / bench.sdd.sdd_count as f64,
            bench.rsdd.num_nodes,
            bench.sdd.sdd_count
        );
    }

    if !args.output.is_empty() {
        let out = json!(benches);
        let pretty_str = serde_json::to_string_pretty(&out).unwrap();
        println!("Writing to {}...", &args.output);
        fs::write(&args.output, pretty_str).expect("Error writing to file");
    }
}
