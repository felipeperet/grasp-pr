use crate::{instance::Instance, local_search::local_search_2opt};

#[derive(Clone)]
pub struct Solution {
    pub path: Vec<usize>,
    pub total_distance: i32,
}

impl Solution {
    pub fn new(num_cities: usize) -> Self {
        Solution {
            path: Vec::with_capacity(num_cities),
            total_distance: 0,
        }
    }

    pub fn eval(&mut self, instance: &Instance) {
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

    pub fn copy(&self) -> Self {
        Solution {
            path: self.path.clone(),
            total_distance: self.total_distance,
        }
    }

    pub fn path_relinking(&mut self, target: &Solution, instance: &Instance) {
        let mut best_distance = self.total_distance;
        let mut best_path = self.path.clone();

        for i in 0..self.path.len() {
            if self.path[i] != target.path[i] {
                let target_index = self.path.iter().position(|&x| x == target.path[i]).unwrap();
                self.path.swap(i, target_index);
                self.eval(instance);

                local_search_2opt(self, instance);

                if self.total_distance < best_distance {
                    best_distance = self.total_distance;
                    best_path = self.path.clone();
                }

                if best_distance == self.total_distance {
                    break;
                }
            }
        }

        self.path = best_path;
        self.total_distance = best_distance;
    }
}
