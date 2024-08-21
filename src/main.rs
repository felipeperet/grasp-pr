use clap::{ArgAction, CommandFactory, Parser, ValueEnum};
use rayon::prelude::*;
use std::fmt;
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

struct Instance {
    num_cities: usize,
    distances: Vec<Vec<i32>>,
}

impl Instance {
    fn load(filename: &str) -> Self {
        let content = std::fs::read_to_string(filename).expect("Failed to read instance file");
        let lines: Vec<&str> = content.lines().collect();

        let mut num_cities = 0;
        let mut distances = Vec::new();
        let mut coords = Vec::new();

        let mut line_iter = lines.iter();

        while let Some(&line) = line_iter.next() {
            if line.starts_with("DIMENSION") {
                num_cities = line.split_whitespace().last().unwrap().parse().unwrap();
                distances = vec![vec![0; num_cities]; num_cities];
            } else if line.starts_with("EDGE_WEIGHT_TYPE: EXPLICIT") {
                while let Some(&line) = line_iter.next() {
                    if line.starts_with("EDGE_WEIGHT_FORMAT") {
                        let format = line.split_whitespace().last().unwrap();
                        match format {
                            "FULL_MATRIX" => {
                                while let Some(&line) = line_iter.next() {
                                    if line.starts_with("EDGE_WEIGHT_SECTION") {
                                        break;
                                    }
                                }

                                for (row_index, line) in
                                    line_iter.clone().enumerate().take(num_cities)
                                {
                                    let row: Vec<i32> = line
                                        .split_whitespace()
                                        .map(|x| x.parse().expect("Failed to parse distance"))
                                        .collect();
                                    distances[row_index][..row.len()].copy_from_slice(&row);
                                }
                            }
                            "UPPER_ROW" => {
                                while let Some(&line) = line_iter.next() {
                                    if line.starts_with("EDGE_WEIGHT_SECTION") {
                                        break;
                                    }
                                }

                                let mut row_index = 0;
                                let mut col_index = 1;
                                for line in line_iter.clone() {
                                    if line.starts_with("EOF") {
                                        break;
                                    }

                                    for value in line.split_whitespace() {
                                        distances[row_index][col_index] =
                                            value.parse().expect("Failed to parse distance");
                                        distances[col_index][row_index] =
                                            distances[row_index][col_index];
                                        col_index += 1;
                                        if col_index >= num_cities {
                                            row_index += 1;
                                            col_index = row_index + 1;
                                        }
                                    }
                                }
                            }
                            _ => panic!("Unsupported EDGE_WEIGHT_FORMAT: {}", format),
                        }
                        break;
                    }
                }
            } else if line.starts_with("EDGE_WEIGHT_TYPE: EUC_2D") {
                while let Some(&line) = line_iter.next() {
                    if line.starts_with("NODE_COORD_SECTION") {
                        break;
                    }
                }

                for _ in 0..num_cities {
                    if let Some(&line) = line_iter.next() {
                        let coords_data: Vec<f64> = line
                            .split_whitespace()
                            .skip(1)
                            .map(|x| x.parse().expect("Failed to parse coordinate"))
                            .collect();
                        coords.push((coords_data[0], coords_data[1]));
                    }
                }

                for i in 0..num_cities {
                    for j in i + 1..num_cities {
                        let dx = coords[i].0 - coords[j].0;
                        let dy = coords[i].1 - coords[j].1;
                        let dist = ((dx * dx + dy * dy).sqrt() + 0.5).floor() as i32;
                        distances[i][j] = dist;
                        distances[j][i] = dist;
                    }
                }
            }
        }

        if distances.is_empty() {
            panic!("Failed to parse the instance file");
        }

        Instance {
            num_cities,
            distances,
        }
    }
}

#[derive(Clone)]
struct Solution {
    path: Vec<usize>,
    total_distance: i32,
}

impl Solution {
    fn new(num_cities: usize) -> Self {
        Solution {
            path: Vec::with_capacity(num_cities),
            total_distance: 0,
        }
    }

    fn eval(&mut self, instance: &Instance) {
        if self.path.len() != instance.num_cities {
            panic!("Path length does not match the number of cities in the instance");
        }
        self.total_distance = 0;

        for i in 0..self.path.len() - 1 {
            let from = self.path[i];
            let to = self.path[i + 1];
            self.total_distance += instance.distances[from][to]
        }

        let last = *self.path.last().unwrap();
        let first = self.path[0];
        self.total_distance += instance.distances[last][first]
    }

    fn copy(&self) -> Self {
        Solution {
            path: self.path.clone(),
            total_distance: self.total_distance,
        }
    }

    fn path_relinking(&mut self, target: &Solution, instance: &Instance) {
        let mut best_distance = self.total_distance;
        let mut best_path = self.path.clone();

        for i in 0..self.path.len() {
            if self.path[i] != target.path[i] {
                let target_index = self.path.iter().position(|&x| x == target.path[i]).unwrap();
                self.path.swap(i, target_index);
                self.eval(instance);

                local_search(self, instance);

                if self.total_distance < best_distance {
                    best_distance = self.total_distance;
                    best_path = self.path.clone();
                }
            }
        }

        self.path = best_path;
        self.total_distance = best_distance;
    }
}

fn grasp(instance: &Instance, time_limit: Duration) -> Solution {
    let best_score = Arc::new(AtomicI32::new(i32::MAX));
    let stop_flag = Arc::new(AtomicBool::new(false));
    let start_time = Instant::now();

    let best_solution = (0..num_cpus::get())
        .into_par_iter()
        .map(|_| {
            while !stop_flag.load(Ordering::Relaxed) {
                if start_time.elapsed() >= time_limit {
                    stop_flag.store(true, Ordering::Relaxed);
                    break;
                }

                let mut solution = constructive_phase(instance);
                local_search(&mut solution, instance);
                solution.eval(instance);

                let current_best_score = best_score.load(Ordering::Relaxed);
                if solution.total_distance < current_best_score {
                    best_score.store(solution.total_distance, Ordering::Relaxed);
                    println!("Improved distance = {}", solution.total_distance);
                }
            }
            best_score.load(Ordering::Relaxed)
        })
        .reduce_with(|best, _| best);

    let mut final_solution = constructive_phase(instance);
    final_solution.total_distance =
        best_solution.expect("GRASP should return at least one solution");
    final_solution
}

fn symmetric_difference(sol1: &Solution, sol2: &Solution) -> usize {
    sol1.path
        .iter()
        .zip(sol2.path.iter())
        .filter(|&(a, b)| a != b)
        .count()
}

fn update_elite_set(
    elite_set: &mut Vec<Solution>,
    solution: Solution,
    max_elite_size: usize,
    min_difference: usize, // This parameter is still needed within this function
) {
    if elite_set.is_empty() {
        elite_set.push(solution);
        return;
    }

    let is_different = elite_set
        .iter()
        .all(|s| symmetric_difference(&s, &solution) >= min_difference);

    if is_different {
        if elite_set.len() < max_elite_size {
            elite_set.push(solution);
        } else {
            let worst_index = elite_set
                .iter()
                .enumerate()
                .max_by_key(|&(_, sol)| sol.total_distance)
                .map(|(i, _)| i)
                .unwrap();
            if solution.total_distance < elite_set[worst_index].total_distance {
                elite_set[worst_index] = solution;
            }
        }
    }
}

fn grasp_static_pr(instance: &Instance, time_limit: Duration, elite_size: usize) -> Solution {
    let min_difference = (instance.num_cities as f64 * 0.1).round() as usize; // 10% of the number of cities

    let elite_set = Arc::new(Mutex::new(Vec::with_capacity(elite_size)));
    let best_score = Arc::new(AtomicI32::new(i32::MAX));
    let best_solution = Arc::new(Mutex::new(None));
    let stop_flag = Arc::new(AtomicBool::new(false));
    let start_time = Instant::now();

    let _ = (0..num_cpus::get())
        .into_par_iter()
        .map(|_| {
            while !stop_flag.load(Ordering::Relaxed) {
                if start_time.elapsed() >= time_limit {
                    stop_flag.store(true, Ordering::Relaxed);
                    break;
                }

                let mut solution = constructive_phase(instance);
                local_search(&mut solution, instance);
                solution.eval(instance);

                let current_best_score = best_score.load(Ordering::Relaxed);
                if solution.total_distance < current_best_score {
                    best_score.store(solution.total_distance, Ordering::Relaxed);
                    *best_solution.lock().unwrap() = Some(solution.copy());
                    println!("Improved distance = {}", solution.total_distance);
                }

                let mut elite_set = elite_set.lock().unwrap();
                update_elite_set(&mut elite_set, solution.copy(), elite_size, min_difference);
            }
            best_score.load(Ordering::Relaxed)
        })
        .reduce_with(|best, _| best);

    {
        let elite_set = elite_set.lock().unwrap();
        elite_set.par_iter().enumerate().for_each(|(i, _)| {
            if stop_flag.load(Ordering::Relaxed) {
                return;
            }

            for j in i + 1..elite_set.len() {
                let mut s = elite_set[i].copy();
                s.path_relinking(&elite_set[j], instance);

                local_search(&mut s, instance);

                let current_best_score = best_score.load(Ordering::Relaxed);
                if s.total_distance < current_best_score {
                    best_score.store(s.total_distance, Ordering::Relaxed);
                    *best_solution.lock().unwrap() = Some(s.copy());
                    println!("Improved distance = {}", s.total_distance);
                }

                if start_time.elapsed() >= time_limit {
                    stop_flag.store(true, Ordering::Relaxed);
                    return;
                }
            }
        });
    }

    let final_solution = best_solution
        .lock()
        .unwrap()
        .take()
        .expect("There should be at least one solution");
    final_solution
}

/// Local search implementation using 2-opt
fn local_search(solution: &mut Solution, instance: &Instance) {
    let mut improvement = true;

    while improvement {
        improvement = false;

        for i in 1..solution.path.len() - 1 {
            for j in i + 1..solution.path.len() {
                if j - i == 1 {
                    continue;
                }

                let old_distance = instance.distances[solution.path[i - 1]][solution.path[i]]
                    + instance.distances[solution.path[j - 1]][solution.path[j]];
                let new_distance = instance.distances[solution.path[i - 1]][solution.path[j - 1]]
                    + instance.distances[solution.path[i]][solution.path[j]];

                if new_distance < old_distance {
                    solution.path[i..j].reverse();
                    solution.eval(instance);

                    improvement = true;
                }
            }
        }
    }
}

/// Constructive phase of GRASP
fn constructive_phase(instance: &Instance) -> Solution {
    let mut solution = Solution::new(instance.num_cities);
    let mut remaining: Vec<usize> = (0..instance.num_cities).collect();

    let start_city = remaining.remove(rand::random::<usize>() % remaining.len());
    solution.path.push(start_city);

    while !remaining.is_empty() {
        let last_city = *solution.path.last().unwrap();
        let mut candidates: Vec<(usize, i32)> = remaining
            .iter()
            .map(|&city| (city, instance.distances[last_city][city]))
            .collect();
        candidates.sort_by_key(|&(_, dist)| dist);

        let next_city = candidates[rand::random::<usize>() % candidates.len()].0;
        remaining.retain(|&x| x != next_city);
        solution.path.push(next_city);
    }

    solution
}

fn list_available_instances() -> String {
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
enum GraspVariant {
    Basic,
    StaticPR,
}

impl fmt::Display for GraspVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraspVariant::Basic => write!(f, "Basic"),
            GraspVariant::StaticPR => write!(f, "StaticPR"),
        }
    }
}

/// Command-line interface (CLI) options.
#[derive(Parser)]
#[command(name = "GRASP TSP Solver")]
#[command(about = "A simple CLI to solve TSP using GRASP algorithm", long_about = None)]
#[command(long_about = None, help_template =
    "{before-help}{name} {version}\n\n{about}\n\n{usage-heading} {usage}\n\n{all-args}{after-help}")]
struct Cli {
    /// Path to the instance file.
    #[arg(short = 'f', long, default_value_t = String::from("instances/bays29.txt"))]
    instance_file: String,

    /// Number of iterations for the GRASP algorithm.
    #[arg(short = 'i', long, default_value_t = 1000)]
    iterations: u32,

    /// Variant of the GRASP to be used.
    #[arg(short = 'v', long, default_value = "basic")]
    variant: GraspVariant,

    /// Size of the elite set for StaticPR (ignored for Basic).
    #[arg(short = 'e', long, default_value_t = 10)]
    elite_size: usize,

    /// Execute with default settings.
    #[arg(short = 'd', long, default_value_t = false, action = ArgAction::SetTrue)]
    default: bool,
}

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
        cli.instance_file = "instances/bays29.tsp".to_string();
        cli.iterations = 1000;
        cli.variant = GraspVariant::Basic
    }

    let instance = Instance::load(&cli.instance_file);

    println!("Instance file: {}", cli.instance_file);
    println!("Number of iterations: {}\n", cli.iterations);
    println!("Variant: {}\n", cli.variant);

    match cli.variant {
        GraspVariant::StaticPR => {
            println!("Elite Size: {}\n", cli.elite_size);
        }
        _ => {}
    }

    let best_solution = match cli.variant {
        GraspVariant::Basic => grasp(&instance, Duration::from_secs(90)),
        GraspVariant::StaticPR => {
            grasp_static_pr(&instance, Duration::from_secs(90), cli.elite_size)
        }
    };

    println!("\nBest solution found: {:?}", best_solution.path);
    println!("Total distance: {}", best_solution.total_distance);
}
