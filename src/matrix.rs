use std::f64;
use std::fmt;

use crate::{Problem, Solution};

pub struct Matrix {
    width: usize,
    height: usize,
    variables: usize,
    coeffs: Vec<f64>,
}

impl fmt::Debug for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in 0..self.height {
            writeln!(f)?;
            for col in 0..self.width {
                write!(f, "{:.6}, ", self.value(row, col))?;
            }
        }
        writeln!(f)
    }
}

impl Matrix {
    pub fn new(problem: &Problem) -> Matrix {
        let height = problem.num_constraints() + 1;
        let width = problem.num_variables() + problem.num_constraints() + 2;
        let mut coeffs = vec![0.0;width*height];

        let mut row = 1;
        for constraint in problem.constraints().iter() {
            for (index, value) in constraint.iter() {
                coeffs[1 + *index as usize + row * width] = *value;
            }
            coeffs[width - 1 + row * width] = constraint.constant();

            row += 1;
        }

        let slack_start_row = problem.num_variables() + 1;
        for i in 0..problem.num_constraints() {
            coeffs[i + slack_start_row + (i + 1) * width] = 1.0;
        }

        if let Some(objective) = problem.objective() {
            let row = 0;
            for (index, value) in objective.iter(){
                coeffs[1 + *index as usize + row * width] = *value * -1.0;
            }
            coeffs[width - 1 + row * width] = objective.constant();
            coeffs[0 + row * width] = 1.0;
        }

        Matrix {
            width,
            height,
            variables: problem.num_variables(),
            coeffs,
        }
    }

    fn find_positive_objective_columns(&self) -> Vec<usize> {
        let mut result = Vec::new();
        for col in 1..self.width {
            let value = self.value(0, col);
            if value > 0.0 {
                result.push(col);
            }
        }

        result
    }

    fn select_pivot_column(&self, positive_cols: &[usize]) -> usize {
        if positive_cols.is_empty() { panic!(); }

        positive_cols[0]
    }

    fn select_pivot_row(&self, pivot_col: usize) -> Option<usize> {
        let mut candidates = Vec::new();

        for row in 1..self.height {
            if self.value(row, pivot_col) > 0.0 {
                candidates.push(row);
            }
        }

        if candidates.is_empty() { return None; }

        let mut min = f64::MAX;
        let mut min_row = candidates[0];
        for row in candidates {
            let min_ratio_test = self.value(row, self.width - 1) /
                self.value(row, pivot_col);

            if min_ratio_test < min {
                min_row = row;
                min = min_ratio_test;
            }
        }

        return Some(min_row);
    }

    fn simplex_pivot(&mut self, pivot_row: usize, pivot_col: usize) {
        let pivot_recip = 1.0 / self.value(pivot_row, pivot_col);
        for col in 0..self.width {
            let cur_val = self.value(pivot_row, col);
            self.set_value(pivot_row, col, cur_val * pivot_recip);
        }

        // println!("Pivot value {},{} set to 1", pivot_row, pivot_col);
        // println!("{:?}", self);

        for row in 0..self.height {
            if row == pivot_row { continue; }

            let delta = -1.0 * self.value(row, pivot_col);

            for col in 0..self.width {
                let cur_val = self.value(row, col);
                self.set_value(row, col, cur_val + delta * self.value(pivot_row, col));
            }
        }
    }

    pub fn simplex(&mut self) -> Solution {
        let mut iteration = 0;
        loop {
            println!("\nIteration {}", iteration);
            println!("{:?}", self);
            if iteration >= self.width * self.height {
                println!("Warning.  Failed to find solution after {} iterations", iteration);
                break;
            }

            let objective_columns = self.find_positive_objective_columns();
            if objective_columns.is_empty() { break; }

            let pivot_col = self.select_pivot_column(&objective_columns);
            println!("Selected pivot column {}", pivot_col);

            let pivot_row = match self.select_pivot_row(pivot_col) {
                None => {
                    println!("Unable to find a pivot row.  function is unbounded below.");
                    break;
                }, Some(row) => row,
            };
            println!("Selected pivot row {}", pivot_row);

            self.simplex_pivot(pivot_row, pivot_col);
            println!("Completed Pivot.");

            iteration += 1;
        }

        println!("Simplex complete.");
        let objective = Some(self.value(0, self.width - 1));
        let mut coeffs = vec![0.0; self.variables];

        for col in 1..(self.width - 1) {
            let mut ident_row = None;
            let mut skip = false;

            for row in 0..self.height {
                if self.value(row, col) != 0.0 {
                    if ident_row.is_some() {
                        skip = true;
                        break;
                    }

                    ident_row = Some(row);
                }
            }

            if skip { continue; }

            if let Some(row) = ident_row {
                let index = col - 1;
                if index < coeffs.len() {
                    coeffs[index] = self.value(row, self.width - 1);
                }
            }
        }

        Solution::new(coeffs, objective)
    }

    fn value(&self, row: usize, col: usize) -> f64 {
        self.coeffs[col + row * self.width]
    }

    fn set_value(&mut self, row: usize, col: usize, value: f64) {
        self.coeffs[col + row * self.width] = value;
    }

    fn swap_values(&mut self, row1: usize, col1: usize, row2: usize, col2: usize) {
        let temp = self.value(row1, col1);
        self.set_value(row1, col1, self.value(row2, col2));
        self.set_value(row2, col2, temp);
    }

    fn find_pivot_max(&self, cur_pivot_row: usize, pivot_col: usize) -> usize {
        let mut max_value = 0.0;
        let mut pivot_max = cur_pivot_row;
        for row in cur_pivot_row .. self.height {
            let cur_value = self.value(row, pivot_col).abs();
            if cur_value > max_value {
                max_value = cur_value;
                pivot_max = row;
            }
        }
        pivot_max
    }

    fn swap_rows(&mut self, row1: usize, row2: usize) {
        for col in 0..self.width {
            self.swap_values(row1, col, row2, col);
        }
    }

    pub fn gaussian_elimination(&mut self) -> Solution {
        let mut pivot_row = 0;
        let mut pivot_col = 0;

        println!("Reducing matrix to triangular form");
        println!("{:?}", self);
        while pivot_row < self.height && pivot_col < self.width {
            let pivot_max = self.find_pivot_max(pivot_row, pivot_col);

            if self.value(pivot_max, pivot_col) == 0.0 {
                pivot_col += 1;
            } else {
                self.swap_rows(pivot_row, pivot_max);

                for row in (pivot_row + 1) .. self.height {
                    let coeff = self.value(row, pivot_col) / self.value(pivot_row, pivot_col);
                    self.set_value(row, pivot_col, 0.0);

                    for col in (pivot_col + 1) .. self.width {
                        let value = self.value(row, col) - self.value(pivot_row, col) * coeff;
                        self.set_value(row, col, value);
                    }
                }

                pivot_row += 1;
                pivot_col += 1;
            }

            println!("Step");
            println!("{:?}", self);
        }

        println!("Back substituting");
        let mut coeffs = vec![0.0; self.height];
        for row in (0..self.height).rev() {
            let mut coeff = self.value(row, self.width - 1);
            for col in (row + 1) .. self.height {
                coeff -= self.value(row, col) * coeffs[col as usize];
            }

            coeffs[row as usize] = coeff / self.value(row, row);
        }

        Solution::new(coeffs, None)
    }
}
