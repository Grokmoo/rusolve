//  This file is part of rusolve, an optimizer / solver written in Rust.
//  Copyright 2019 Jared Stephen
//
//  rusolve is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  rusolve is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with rusolve.  If not, see <http://www.gnu.org/licenses/>

use std::io::{Error, ErrorKind};
use std::f64;
use std::fmt;

use log::{debug, info, warn};

use crate::{Problem, Solution, ObjectiveKind, ConstraintKind};

pub struct Matrix {
    width: usize,
    height: usize,
    variables: usize,
    objective_kind: ObjectiveKind,
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
    pub fn new_simplex(problem: &Problem) -> Result<Matrix, Error> {
        if problem.objective_kind().is_none() {
            return Err(Error::new(ErrorKind::InvalidInput,
                "Must set an objective function for simplex solver."));
        }

        info!("Set up simplex problem with {} constraints and {} variables",
              problem.num_constraints(), problem.num_variables());

        let height = problem.num_constraints() + 1;
        let width = problem.num_variables() + problem.num_constraints() + 2;
        let mut coeffs = vec![0.0;width*height];

        let mut row = 1;
        for constraint in problem.constraints().iter() {
            for (index, value) in constraint.expr().iter() {
                coeffs[1 + *index as usize + row * width] = *value;
            }
            coeffs[width - 1 + row * width] = constraint.constant();

            row += 1;
        }

        if let Some(objective) = problem.objective() {
            let row = 0;
            for (index, value) in objective.iter(){
                coeffs[1 + *index as usize + row * width] = *value * -1.0;
            }
            coeffs[0 + row * width] = 1.0;
        }

        let mut matrix = Matrix {
            width,
            height, objective_kind: problem.objective_kind().unwrap(),
            variables: problem.num_variables(),
            coeffs,
        };
        info!("Set up initial problem.");
        debug!("{:?}", matrix);

        let slack_start_row = problem.num_variables() + 1;
        for (i, constraint) in problem.constraints().iter().enumerate() {
            let value = match constraint.kind() {
                ConstraintKind::LessThanOrEqualTo => 1.0,
                ConstraintKind::GreaterThanOrEqualTo => -1.0,
                ConstraintKind::EqualTo => -1.0,
            };
            matrix.coeffs[i + slack_start_row + (i + 1) * width] = value;
        }
        info!("Set up slack variables");
        debug!("{:?}", matrix);

        Ok(matrix)
    }

    fn find_valid_objective_columns(&self) -> Vec<usize> {
        let mut result = Vec::new();
        for col in 1..self.width {
            let value = self.value(0, col);
            match self.objective_kind {
                ObjectiveKind::Minimize => if value > 0.0 { result.push(col); }
                ObjectiveKind::Maximize => if value < 0.0 { result.push(col); }
            }
        }

        result
    }

    fn select_pivot_column(&self, valid_cols: &[usize]) -> usize {
        if valid_cols.is_empty() { panic!(); }

        // TODO select the best column through an algorithm such as devex
        valid_cols[0]
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

        debug!("Pivot value {},{} set to 1", pivot_row, pivot_col);

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
        info!("Initializing simplex solver");
        debug!("Problem:");
        debug!("{:?}", self);

        let mut iteration = 0;
        loop {
            info!("\nIteration {}", iteration);
            debug!("{:?}", self);
            if iteration >= self.width * self.height {
                warn!("Warning.  Failed to find solution after {} iterations", iteration);
                break;
            }

            let objective_columns = self.find_valid_objective_columns();
            if objective_columns.is_empty() { break; }

            let pivot_col = self.select_pivot_column(&objective_columns);
            info!("Selected pivot column {}", pivot_col);

            let pivot_row = match self.select_pivot_row(pivot_col) {
                None => {
                    warn!("Unable to find a pivot row.  function is unbounded below.");
                    break;
                }, Some(row) => row,
            };
            info!("Selected pivot row {}", pivot_row);

            self.simplex_pivot(pivot_row, pivot_col);
            info!("Completed pivot and moving to next iteration.");

            iteration += 1;
        }

        info!("Simplex solve complete.");

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

        let solution = Solution::new(coeffs, objective);
        info!("Solution found {:?}", solution);
        solution
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

        info!("Performing gaussian elimination");
        debug!("{:?}", self);

        info!("Reducing matrix to triangular form");
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

            debug!("Step");
            debug!("{:?}", self);
        }

        info!("Back substituting");
        let mut coeffs = vec![0.0; self.height];
        for row in (0..self.height).rev() {
            let mut coeff = self.value(row, self.width - 1);
            for col in (row + 1) .. self.height {
                coeff -= self.value(row, col) * coeffs[col as usize];
            }

            coeffs[row as usize] = coeff / self.value(row, row);
        }

        let solution = Solution::new(coeffs, None);
        info!("Solution found: {:?}", solution);
        solution
    }
}
