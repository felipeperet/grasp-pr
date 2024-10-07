use clap::{ArgAction, Parser, ValueEnum};
use std::fmt;
use std::fs;

pub fn list_available_instances() -> String {
    let mut instances = String::new();
    if let Ok(entries) = fs::read_dir("instances") {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(file_name) = entry.path().file_name() {
                    if let Some(file_name_str) = file_name.to_str() {
                        instances.push_str(&format!("  instances/{}\n", file_name_str));
                    }
                }
            }
        }
    }
    instances
}

#[derive(Debug, Clone, ValueEnum)]
pub enum GraspVariant {
    Basic,
    StaticPR,
    Benchmark,
}

impl fmt::Display for GraspVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraspVariant::Basic => write!(f, "Basic"),
            GraspVariant::StaticPR => write!(f, "StaticPR"),
            GraspVariant::Benchmark => write!(f, "Benchmark"),
        }
    }
}

/// Command-line interface (CLI) options.
#[derive(Parser)]
#[command(name = "GRASP TSP Solver")]
#[command(about = "A simple CLI to solve TSP using GRASP algorithm", long_about = None)]
#[command(long_about = None, help_template =
    "{before-help}{name} {version}\n\n{about}\n\n{usage-heading} {usage}\n\n{all-args}{after-help}")]
pub struct Cli {
    /// Path to the instance file.
    #[arg(short = 'f', long, default_value_t = String::from("instances/bays29.txt"))]
    pub instance_file: String,

    /// Time limit for the GRASP algorithm in seconds.
    #[arg(short = 't', long, default_value_t = 120)]
    pub time_limit: u64,

    /// Variant of the GRASP to be used.
    #[arg(short = 'v', long, default_value = "basic")]
    pub variant: GraspVariant,

    /// Size of the elite set for StaticPR (ignored for Basic).
    #[arg(short = 'e', long, default_value_t = 10)]
    pub elite_size: usize,

    /// Execute with default settings.
    #[arg(short = 'd', long, default_value_t = false, action = ArgAction::SetTrue)]
    pub default: bool,
}
