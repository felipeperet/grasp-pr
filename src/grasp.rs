use crate::instance::Instance;
use crate::local_search::local_search_2opt;
use crate::solution::Solution;

use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub fn grasp(instance: &Instance, time_limit: Duration) -> Solution {
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
                local_search_2opt(&mut solution, instance);
                solution.eval(instance);

                let current_best_score = best_score.load(Ordering::Relaxed);
                if solution.total_distance < current_best_score {
                    best_score.store(solution.total_distance, Ordering::Relaxed);
                    println!("Improved distance = {}", solution.total_distance);
                }
            }
            best_score.load(Ordering::Relaxed)
        })
        .reduce_with(|best, next| if next < best { next } else { best });

    let mut final_solution = constructive_phase(instance);
    final_solution.total_distance =
        best_solution.expect("GRASP should return at least one solution");
    final_solution
}

pub fn symmetric_difference(sol1: &Solution, sol2: &Solution) -> usize {
    sol1.path
        .iter()
        .zip(sol2.path.iter())
        .filter(|&(a, b)| a != b)
        .count()
}

pub fn update_elite_set(
    elite_set: &mut Vec<Solution>,
    solution: Solution,
    max_elite_size: usize,
    min_difference: usize,
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

pub fn grasp_static_pr(instance: &Instance, time_limit: Duration, elite_size: usize) -> Solution {
    let min_difference = (instance.num_cities as f64 * 0.1).round() as usize;

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
                local_search_2opt(&mut solution, instance);
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

                local_search_2opt(&mut s, instance);

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

/// Constructive phase of GRASP
pub fn constructive_phase(instance: &Instance) -> Solution {
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

        let k = (candidates.len() as f32 / 3.0).ceil() as usize;
        let next_city = candidates[rand::random::<usize>() % k].0;

        remaining.retain(|&x| x != next_city);
        solution.path.push(next_city);
    }

    solution
}
