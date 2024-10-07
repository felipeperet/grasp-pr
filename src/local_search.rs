use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Instant;

use crate::grasp::constructive_phase;
use crate::instance::Instance;
use crate::solution::Solution;

/// Local search implementation using Swap (1-opt)
pub fn local_search_swap(solution: &mut Solution, instance: &Instance) {
    let mut improvement = true;

    while improvement {
        improvement = false;

        for i in 1..solution.path.len() - 1 {
            for j in i + 1..solution.path.len() - 1 {
                let current_cost = instance.distances[solution.path[i - 1]][solution.path[i]]
                    + instance.distances[solution.path[i]][solution.path[i + 1]]
                    + instance.distances[solution.path[j - 1]][solution.path[j]]
                    + instance.distances[solution.path[j]][solution.path[j + 1]];

                solution.path.swap(i, j);

                let new_cost = instance.distances[solution.path[i - 1]][solution.path[i]]
                    + instance.distances[solution.path[i]][solution.path[i + 1]]
                    + instance.distances[solution.path[j - 1]][solution.path[j]]
                    + instance.distances[solution.path[j]][solution.path[j + 1]];

                if new_cost < current_cost {
                    improvement = true;
                    solution.eval(instance);
                    break;
                } else {
                    solution.path.swap(i, j);
                }
            }

            if improvement {
                break;
            }
        }
    }
}

/// Local search implementation using 2-opt
pub fn local_search_2opt(solution: &mut Solution, instance: &Instance) {
    let mut improvement = true;

    while improvement {
        improvement = false;

        for i in 1..solution.path.len() - 1 {
            for j in i + 1..solution.path.len() {
                if j - i == 1 {
                    continue;
                }

                let current_cost = instance.distances[solution.path[i - 1]][solution.path[i]]
                    + instance.distances[solution.path[j - 1]][solution.path[j]];
                let new_cost = instance.distances[solution.path[i - 1]][solution.path[j - 1]]
                    + instance.distances[solution.path[i]][solution.path[j]];

                if new_cost < current_cost {
                    solution.path[i..j].reverse();
                    solution.eval(instance);
                    improvement = true;
                    break;
                }
            }

            if improvement {
                break;
            }
        }
    }
}

pub fn benchmark_local_search(instance: &Instance, instance_name: &str, num_runs: usize) {
    let mut results = vec![];

    for run in 1..=num_runs {
        println!("\n=== Run {} for 2-opt ===", run);
        let mut solution_2opt = constructive_phase(instance);
        let start_2opt = Instant::now();
        local_search_2opt(&mut solution_2opt, instance);
        let duration_2opt = start_2opt.elapsed();

        println!(
            "2-opt: Distance = {}, Time = {:.2?}",
            solution_2opt.total_distance, duration_2opt
        );

        println!("\n=== Run {} for Swap ===", run);
        let mut solution_swap = constructive_phase(instance);
        let start_swap = Instant::now();
        local_search_swap(&mut solution_swap, instance);
        let duration_swap = start_swap.elapsed();

        println!(
            "Swap: Distance = {}, Time = {:.2?}",
            solution_swap.total_distance, duration_swap
        );

        results.push((
            run,
            solution_2opt.total_distance,
            duration_2opt.as_micros(),
            solution_swap.total_distance,
            duration_swap.as_micros(),
        ));
    }

    let file_path = format!("{}_benchmark_results.csv", instance_name);
    let file = File::create(&file_path).expect("Unable to create file");
    let mut writer = BufWriter::new(file);

    writeln!(
        writer,
        "Run,2-opt Distance,2-opt Time (µs),Swap Distance,Swap Time (µs)"
    )
    .expect("Failed to write header to CSV");

    for (run, d2opt, t2opt, dswap, tswap) in results {
        writeln!(writer, "{},{},{},{},{}", run, d2opt, t2opt, dswap, tswap)
            .expect("Failed to write results to CSV");
    }

    println!("Benchmark results saved to {}", file_path);
}
