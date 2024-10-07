pub struct Instance {
    pub num_cities: usize,
    pub distances: Vec<Vec<i32>>,
}

impl Instance {
    pub fn load(filename: &str) -> Self {
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
