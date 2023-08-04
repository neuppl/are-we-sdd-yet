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

    /// Path to compiled cnf2obdd binary
    #[clap(long, value_parser, default_value = "./cnf2obdd")]
    path_to_cnf2obdd: String,
}

#[derive(Clone, Copy, PartialEq, Serialize)]
enum CompilationMode {
    SDDLeftLinear,
    SDDRightLinear,
    // SDDBalanced,
    BestFit,
    BDDBestFit,
}

impl CompilationMode {
    fn as_libsdd(&self) -> &'static str {
        match &self {
            CompilationMode::SDDLeftLinear => "left",
            CompilationMode::SDDRightLinear => "right",
            // CompilationMode::SDDBalanced => "balanced",
            CompilationMode::BestFit => "right", // TODO: this seems wrong?
            _ => panic!("invalid compilation mode for libsdd"),
        }
    }

    fn as_rsdd(&self) -> &'static str {
        match &self {
            CompilationMode::SDDLeftLinear => "sdd_left_linear",
            CompilationMode::SDDRightLinear => "sdd_right_linear",
            // CompilationMode::SDDBalanced => "linear",
            CompilationMode::BestFit => "sdd_dtree_minfill",
            CompilationMode::BDDBestFit => "bdd_dtree_minfill",
        }
    }
}

impl Display for CompilationMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            CompilationMode::SDDLeftLinear => f.write_str("left linear"),
            CompilationMode::SDDRightLinear => f.write_str("right linear"),
            // CompilationMode::SDDBalanced => f.write_str("balanced"),
            CompilationMode::BestFit => f.write_str("best fit"),
            CompilationMode::BDDBestFit => f.write_str("best fit (bdd)"),
        }
    }
}

/// copied over from rsdd.rs
#[derive(Serialize, Deserialize)]
struct SddBenchmarkLog {
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
    mode: String,
}

#[derive(Serialize, Deserialize)]
struct Cnf2ObddBenchmarkLog {
    time: f64,
}

#[derive(Serialize)]
struct BenchmarkLog {
    file: String,
    mode: CompilationMode,
    rsdd: RsddBenchmarkLog,
    cnf2obdd: Option<Cnf2ObddBenchmarkLog>,
    sdd: Option<SddBenchmarkLog>,
}

impl Display for BenchmarkLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let header = format!("Benchmark: {} (strategy: {})...", self.file, self.mode);

        let rsdd_v_sdd = match (&self.rsdd, &self.sdd) {
            (rsdd, Some(sdd)) => {
                let speedup = format!(
                    "{:.2}x speedup (rsdd: {:.6}s, sdd: {:.6}s)",
                    sdd.compilation_time / rsdd.time_in_sec,
                    rsdd.time_in_sec,
                    sdd.compilation_time
                );
                let size = format!(
                    "{:.2}x circuit size (rsdd: {}, sdd: {})",
                    rsdd.circuit_size as f64 / sdd.sdd_size as f64,
                    rsdd.circuit_size,
                    sdd.sdd_size
                );
                format!("{}\n{}\n", speedup, size)
            }
            (rsdd, None) => {
                format!(
                    "no sdd run reported\nrsdd: {:.6}s, {} size",
                    rsdd.time_in_sec, rsdd.circuit_size
                )
            }
        };

        let rsdd_v_cnf2obdd = match (&self.rsdd, &self.cnf2obdd) {
            (rsdd, Some(cnf2obdd)) => {
                let speedup = format!(
                    "{:.2}x speedup (rsdd: {:.6}s, cnf2obdd: {:.6}s)",
                    cnf2obdd.time / rsdd.time_in_sec,
                    rsdd.time_in_sec,
                    cnf2obdd.time
                );
                speedup
            }
            (rsdd, None) => {
                format!(
                    "no cnf2obdd run reported\nrsdd: {:.6}s, {} size",
                    rsdd.time_in_sec, rsdd.circuit_size
                )
            }
        };

        f.write_fmt(format_args!(
            "===\n{}\n---\nrsdd v sdd\n{}\n---\nrsdd v cnf2obdd\n{}",
            header, rsdd_v_sdd, rsdd_v_cnf2obdd
        ))
    }
}

fn sdd(
    path_to_sdd: &str,
    file: &str,
    mode: &CompilationMode,
    debug: bool,
) -> Option<SddBenchmarkLog> {
    if !matches!(
        mode,
        CompilationMode::SDDLeftLinear | CompilationMode::SDDRightLinear | CompilationMode::BestFit
    ) {
        return None;
    }

    let mut command = Command::new(path_to_sdd);

    command
        .arg("-c")
        .arg(file)
        .arg("-t")
        .arg(mode.as_libsdd())
        .stdout(Stdio::piped());

    if !matches!(mode, CompilationMode::BestFit) {
        command.arg("-r").arg("0");
    }

    if !debug {
        command.stderr(Stdio::null());
    }

    let command = command.spawn().expect("libsdd failure");

    let stdout = command.wait_with_output().unwrap().stdout;

    Some(serde_json::from_slice::<SddBenchmarkLog>(&stdout).unwrap())
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

fn cnf2obdd(
    path_to_cnf2obdd: &str,
    file: &str,
    mode: &CompilationMode,
    debug: bool,
) -> Option<Cnf2ObddBenchmarkLog> {
    if !matches!(mode, CompilationMode::BestFit | CompilationMode::BDDBestFit) {
        return None;
    }

    let mut command = Command::new(path_to_cnf2obdd);

    command.arg(file).stdout(Stdio::piped());

    if !debug {
        command.stderr(Stdio::null());
    }

    let command = command.spawn().expect("cnf2obdd failure");

    let output = command.wait_with_output();

    match output {
        Ok(output) => match serde_json::from_slice::<Cnf2ObddBenchmarkLog>(&output.stdout) {
            Ok(s) => Some(s),
            Err(e) => {
                if debug {
                    eprintln!("{}", e)
                };
                None
            }
        },
        Err(e) => {
            if debug {
                eprintln!("{}", e)
            };
            None
        }
    }
}

fn benchmark(args: &Args, mode: &CompilationMode) -> Vec<BenchmarkLog> {
    args.files
        .iter()
        .map(|file| BenchmarkLog {
            file: file.to_string(),
            mode: *mode,
            rsdd: rsdd(&args.path_to_rsdd, file, mode, args.debug),
            cnf2obdd: cnf2obdd(&args.path_to_cnf2obdd, file, mode, args.debug),
            sdd: sdd(&args.path_to_sdd, file, mode, args.debug),
        })
        .collect()
}

fn str_to_mode(str: &str) -> CompilationMode {
    match str {
        "left" => CompilationMode::SDDLeftLinear,
        // "balanced" => CompilationMode::SDDBalanced,
        "best" => CompilationMode::BestFit,
        "best-bdd" | "bdd-best" => CompilationMode::BDDBestFit,
        _ => CompilationMode::SDDRightLinear,
    }
}

fn main() {
    let args = Args::parse();

    let mode = str_to_mode(&args.mode);

    let benches = benchmark(&args, &mode);

    for bench in benches.iter() {
        println!("{}", bench);
    }

    if !args.output.is_empty() {
        let out = json!(benches);
        let pretty_str = serde_json::to_string_pretty(&out).unwrap();
        println!("Writing to {}...", &args.output);
        fs::write(&args.output, pretty_str).expect("Error writing to file");
    }
}
