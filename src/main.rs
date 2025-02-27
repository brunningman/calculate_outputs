use std::collections::{BinaryHeap, HashMap};
use std::time::Instant;
use std::cmp::Ordering;
use std::env;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Node {
    values: Vec<i64>, // Scaled values as integers
    path: Vec<String>, // Track operations
    depth: usize,      // Track depth
    estimated_cost: i64, // Used for priority in A* (difference from target)
}

// Custom Ord for BinaryHeap priority based on estimated cost (A* heuristic)
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.estimated_cost.cmp(&self.estimated_cost) // Min-heap behavior
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

const SCALE: f64 = 1000.0;
const MAX_DEPTH: usize = 6; // Reduced max depth to limit path expansion

fn main() {
    // let inputs = vec![60.0];
    // let target = 12.0;
    // let can_be_off_by = 1.0;
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <inputs> <target> <canBeOffBy>", args[0]);
        eprintln!("Example: {} \"10.0,10.0,10.0\" 12.0 1.0", args[0]);
        std::process::exit(1);
    }

    // Convert inputs to a vector of floats
    let inputs: Vec<f64> = args[1]
        .split(',')
        .map(|s| s.parse().expect("Failed to parse input values"))
        .collect();

    // Parse target and canBeOffBy as floats
    let target: f64 = args[2].parse().expect("Failed to parse target");
    let can_be_off_by: f64 = args[3].parse().expect("Failed to parse canBeOffBy");

    let start_time = Instant::now(); // Start the timer

    if let Some((final_output, remainder, path)) = shortest_path_to_target(inputs, target, can_be_off_by) {
        println!("Final Outputs: {:?}", final_output);
        println!("Remainder: {:?}", remainder);
        for step in path {
            println!("{}", step);
        }
    } else {
        println!("No solution found.");
    }

    let duration = start_time.elapsed(); // Calculate time taken
    println!("Total time taken: {:?}", duration);
}

// Helper function to scale values for integer representation
fn scale_value(value: f64) -> i64 {
    (value * SCALE) as i64
}

// New function for calculating heuristic based on proximity to target
fn calculate_heuristic(values: &[i64], target: i64) -> i64 {
    values.iter().map(|&v| (v - target).abs()).sum()
}

// Operation to split a value into two equal parts
fn split_into_two(input: f64) -> (f64, f64) {
    (input / 2.0, input / 2.0)
}

// Operation to split a value into three equal parts
fn split_into_three(input: f64) -> (f64, f64, f64) {
    let part = input / 3.0;
    (part, part, part)
}

// Operation to combine two values by summing them
fn combine_two(a: f64, b: f64) -> f64 {
    a + b
}

// Operation to combine three values by summing them
fn combine_three(a: f64, b: f64, c: f64) -> f64 {
    a + b + c
}

// Main A* function to find the shortest path to target
fn shortest_path_to_target(inputs: Vec<f64>, target: f64, can_be_off_by: f64) -> Option<(Vec<f64>, Vec<f64>, Vec<String>)> {
    let mut priority_queue = BinaryHeap::new();
    let mut visited = HashMap::new();

    // Scaling parameters
    let scaled_target = scale_value(target);
    let scaled_margin = scale_value(can_be_off_by);

    // Initialize with the starting node
    let start = Node {
        values: inputs.iter().map(|&v| scale_value(v)).collect(),
        path: vec![],
        depth: 0,
        estimated_cost: calculate_heuristic(&inputs.iter().map(|&v| scale_value(v)).collect::<Vec<i64>>(), scaled_target),
    };
    priority_queue.push(start.clone());
    visited.insert(start.values.clone(), start.depth);

    while let Some(current) = priority_queue.pop() {
        // Evaluate for target proximity
        if let Some(final_values) = find_final_and_remainder(&current.values, scaled_target, scaled_margin) {
            let (final_output_scaled, remainder_scaled) = final_values;
            let final_output = final_output_scaled.iter().map(|&v| v as f64 / SCALE).collect();
            let remainder = remainder_scaled.iter().map(|&v| v as f64 / SCALE).collect();
            println!("Solution found at level {}", current.depth);
            return Some((final_output, remainder, current.path.clone()));
        }

        // Avoid unnecessary depth
        if current.depth >= MAX_DEPTH {
            continue;
        }

        // Explore operations: split/combine
        for i in 0..current.values.len() {
            let value = current.values[i];
            let mut new_values;
            let mut new_path;

            // Apply each operation (split/combine) and check result immediately

            // Split into two
            let (part1, part2) = split_into_two(value as f64 / SCALE);
            new_values = current.values.clone();
            new_values.remove(i);
            new_values.push(scale_value(part1));
            new_values.push(scale_value(part2));
            new_path = current.path.clone();
            new_path.push(format!("{} -> [{}, {}]", value as f64 / SCALE, part1, part2));

            let estimated_cost = calculate_heuristic(&new_values, scaled_target);
            let new_node = Node {
                values: new_values.clone(),
                path: new_path.clone(),
                depth: current.depth + 1,
                estimated_cost,
            };

            // Prune based on heuristic and if state has been reached at lower cost
            if !visited.contains_key(&new_node.values) || visited[&new_node.values] > new_node.depth {
                priority_queue.push(new_node.clone());
                visited.insert(new_node.values.clone(), new_node.depth);
            }

            // Split into three
            let (part1, part2, part3) = split_into_three(value as f64 / SCALE);
            new_values = current.values.clone();
            new_values.remove(i);
            new_values.push(scale_value(part1));
            new_values.push(scale_value(part2));
            new_values.push(scale_value(part3));
            new_path = current.path.clone();
            new_path.push(format!("{} -> [{}, {}, {}]", value as f64 / SCALE, part1, part2, part3));

            let estimated_cost = calculate_heuristic(&new_values, scaled_target);
            let new_node = Node {
                values: new_values.clone(),
                path: new_path.clone(),
                depth: current.depth + 1,
                estimated_cost,
            };

            if !visited.contains_key(&new_node.values) || visited[&new_node.values] > new_node.depth {
                priority_queue.push(new_node.clone());
                visited.insert(new_node.values.clone(), new_node.depth);
            }

            // Combine two values
            for j in (i+1)..current.values.len() {
                let other_value = current.values[j];
                let combined = combine_two(value as f64 / SCALE, other_value as f64 / SCALE);
                new_values = current.values.clone();
                new_values.remove(i);
                new_values.remove(j - 1); // Adjust index after removal
                new_values.push(scale_value(combined));
                new_path = current.path.clone();
                new_path.push(format!("{} + {} -> {}", value as f64 / SCALE, other_value as f64 / SCALE, combined));

                let estimated_cost = calculate_heuristic(&new_values, scaled_target);
                let new_node = Node {
                    values: new_values.clone(),
                    path: new_path.clone(),
                    depth: current.depth + 1,
                    estimated_cost,
                };

                if !visited.contains_key(&new_node.values) || visited[&new_node.values] > new_node.depth {
                    priority_queue.push(new_node.clone());
                    visited.insert(new_node.values.clone(), new_node.depth);
                }
            }

            // Combine three values
            for j in (i+1)..current.values.len() {
                for k in (j+1)..current.values.len() {
                    let value_b = current.values[j];
                    let value_c = current.values[k];
                    let combined = combine_three(value as f64 / SCALE, value_b as f64 / SCALE, value_c as f64 / SCALE);
                    new_values = current.values.clone();
                    new_values.remove(i);
                    new_values.remove(j - 1);
                    new_values.remove(k - 2); // Adjust indices after each removal
                    new_values.push(scale_value(combined));
                    new_path = current.path.clone();
                    new_path.push(format!("{} + {} + {} -> {}", value as f64 / SCALE, value_b as f64 / SCALE, value_c as f64 / SCALE, combined));

                    let estimated_cost = calculate_heuristic(&new_values, scaled_target);
                    let new_node = Node {
                        values: new_values.clone(),
                        path: new_path.clone(),
                        depth: current.depth + 1,
                        estimated_cost,
                    };

                    if !visited.contains_key(&new_node.values) || visited[&new_node.values] > new_node.depth {
                        priority_queue.push(new_node.clone());
                        visited.insert(new_node.values.clone(), new_node.depth);
                    }
                }
            }
        }
    }

    None // No solution found
}

// Helper function to separate final values close to target and remainder
fn find_final_and_remainder(values: &[i64], target: i64, margin: i64) -> Option<(Vec<i64>, Vec<i64>)> {
    let mut final_values = Vec::new();
    let mut remainder = Vec::new();

    for &value in values {
        if (value - target).abs() <= margin {
            final_values.push(value);
        } else {
            remainder.push(value);
        }
    }

    if !final_values.is_empty() {
        Some((final_values, remainder))
    } else {
        None
    }
}
