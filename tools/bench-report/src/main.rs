use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Deserialize)]
struct Estimates {
    mean: Estimate,
    median: Estimate,
}

#[derive(Deserialize)]
struct Estimate {
    point_estimate: f64,
}

#[derive(Deserialize)]
struct Benchmark {
    full_id: String,
    #[serde(default)]
    throughput: Option<Throughput>,
}

#[derive(Deserialize)]
struct Throughput {
    #[serde(rename = "Bytes")]
    bytes: Option<u64>,
}

struct Row {
    id: String,
    median_ms: f64,
    mean_ms: f64,
    mb_per_s: Option<f64>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/criterion"));
    let mut rows = Vec::new();
    collect(&root, &mut rows)?;
    rows.sort_by(|a, b| a.id.cmp(&b.id));
    println!("| bench | median ms | mean ms | MB/s |");
    println!("|---|---:|---:|---:|");
    for row in rows {
        let mbs = row.mb_per_s.map_or(String::new(), |v| format!("{v:.2}"));
        println!(
            "| {} | {:.3} | {:.3} | {} |",
            row.id, row.median_ms, row.mean_ms, mbs
        );
    }
    Ok(())
}

fn collect(dir: &Path, rows: &mut Vec<Row>) -> Result<(), Box<dyn Error>> {
    let new = dir.join("new");
    if new.join("estimates.json").is_file() && new.join("benchmark.json").is_file() {
        let estimates: Estimates = serde_json::from_slice(&fs::read(new.join("estimates.json"))?)?;
        let benchmark: Benchmark = serde_json::from_slice(&fs::read(new.join("benchmark.json"))?)?;
        let median_ns = estimates.median.point_estimate;
        let bytes = benchmark.throughput.and_then(|t| t.bytes);
        rows.push(Row {
            id: benchmark.full_id,
            median_ms: median_ns / 1e6,
            mean_ms: estimates.mean.point_estimate / 1e6,
            mb_per_s: bytes.map(|b| b as f64 / (1024.0 * 1024.0) / (median_ns / 1e9)),
        });
        return Ok(());
    }
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(&path, rows)?;
        }
    }
    Ok(())
}
