mod body;
mod dynamics;
mod writer;

use body::Body;
use dynamics::simulate;

use clap::Parser;
use serde_json;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// JSON file with initial conditions
    input: PathBuf,

    /// File to store results of the simulation
    #[arg(short, long, default_value = "newtonian.parquet")]
    output: Option<PathBuf>,

    /// Number of seconds to simulate (e.g., "60*60*24*365")
    #[arg(short, long, default_value = "60*60*24*365", value_parser = parse_expression)]
    total_time: f64,

    /// Time step in seconds for finite difference method (e.g., "1.0 / 1000.0")
    #[arg(short, long, default_value = "0.001", value_parser = parse_expression)]
    delta_t: f64,

    /// Record every N seconds (e.g., "60*10")
    #[arg(short, long, default_value = "1", value_parser = parse_expression_to_u32)]
    record_interval: u64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let bodies = load_initial_conditions(&args.input)?;
    let output_file = args
        .output
        .unwrap_or_else(|| PathBuf::from("newtonian.parquet"));
    let mut writer = writer::Writer::new(output_file)?;
    simulate(
        &mut bodies.clone(),
        args.total_time,
        args.delta_t,
        args.record_interval,
        &mut writer,
    )?;

    writer.close()?;
    Ok(())
}

fn load_initial_conditions(file_path: &PathBuf) -> Result<Vec<Body>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let bodies: Vec<Body> = serde_json::from_reader(reader)?;
    Ok(bodies)
}

/// Parses a string expression (e.g., "60*60*24") into an f64 value.
fn parse_expression(expr_str: &str) -> Result<f64, String> {
    meval::eval_str(expr_str).map_err(|e| e.to_string())
}

fn parse_expression_to_u32(expr_str: &str) -> Result<u64, String> {
    meval::eval_str(expr_str)
        .map(|val: f64| val.round() as u64)
        .map_err(|e| e.to_string())
}
