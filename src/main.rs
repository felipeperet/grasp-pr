mod cli;
mod grasp;
mod instance;
mod local_search;
mod solution;

use clap::{CommandFactory, Parser};
use cli::{list_available_instances, Cli, GraspVariant};
use grasp::{grasp, grasp_static_pr};
use local_search::benchmark_local_search;
use std::time::Duration;

use crate::instance::Instance;

fn main() {
    let mut cli = Cli::parse();

    if std::env::args().len() == 1 {
        let mut cmd = Cli::command();
        cmd.print_help().unwrap();

        let bold = "\x1b[1m";
        let reset = "\x1b[0m";
        let underline = "\x1b[4m";

        println!(
            "\n{}{}Available instances:{}\n{}",
            bold,
            underline,
            reset,
            list_available_instances()
        );
        std::process::exit(0);
    }

    if cli.default {
        cli.instance_file = "instances/bier127.tsp".to_string();
        cli.time_limit = 120;
        cli.variant = GraspVariant::Basic;
    }

    match cli.variant {
        GraspVariant::Benchmark => {
            let instances = vec!["instances/bays29.tsp", "instances/brg180.tsp"];

            for instance_file in &instances {
                println!("\nRunning benchmark for instance: {}\n", instance_file);

                let instance = Instance::load(instance_file);
                let instance_name = if instance_file.contains("bays29") {
                    "bays29"
                } else if instance_file.contains("brg180") {
                    "brg180"
                } else {
                    "unknown_instance"
                };

                benchmark_local_search(&instance, instance_name, 100);
            }
        }
        GraspVariant::Basic => {
            let instance = Instance::load(&cli.instance_file);
            let best_solution = grasp(&instance, Duration::from_secs(cli.time_limit));
            println!("\nBest solution found: {:?}", best_solution.path);
            println!("Total distance: {}", best_solution.total_distance);
        }
        GraspVariant::StaticPR => {
            let instance = Instance::load(&cli.instance_file);
            let best_solution = grasp_static_pr(
                &instance,
                Duration::from_secs(cli.time_limit),
                cli.elite_size,
            );
            println!("\nBest solution found: {:?}", best_solution.path);
            println!("Total distance: {}", best_solution.total_distance);
        }
    }
}
