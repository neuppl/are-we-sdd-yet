extern crate cudd_sys;

use clap::Parser;
use cudd_sys::{DdManager, cudd::{Cudd_Init, CUDD_UNIQUE_SLOTS, CUDD_CACHE_SLOTS}};

/// Test driver for one-shot benchmark
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Print debug messages to console
    #[clap(short, long, value_parser, default_value_t = false)]
    debug: bool,

    /// File to benchmark
    #[clap(short, long, value_parser)]
    file: String,

    #[clap(short, long, value_parser)]
    mode: String,
}

fn main() {
  let args = Args::parse();

  unsafe {
    let man = Cudd_Init(0,0,CUDD_UNIQUE_SLOTS,CUDD_CACHE_SLOTS,0);
  }
}
