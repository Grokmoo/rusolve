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

use std::f64;

use log::{debug, info, warn};

use crate::{ConstraintKind, Row, Col, Matrix, Problem, Solution, ObjectiveKind,
    Result, SolverError};

pub fn setup_matrix(problem: &Problem) -> Result<Matrix> {
    if problem.objective_kind().is_none() {
        return SolverError::invalid_objective("Must set an objective for simplex.");
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

    let mut matrix = Matrix::new_simplex(width, height, problem.num_variables(),
        problem.objective_kind().unwrap(), coeffs);
    info!("Set up initial problem.");
    debug!("{:?}", matrix);

    let slack_start_row = problem.num_variables() + 1;
    for (i, constraint) in problem.constraints().iter().enumerate() {
        let value = match constraint.kind() {
            ConstraintKind::LessThanOrEqualTo => 1.0,
            ConstraintKind::GreaterThanOrEqualTo => -1.0,
            ConstraintKind::EqualTo => 0.0,
        };
        matrix.set_value_raw(i + 1, i + slack_start_row, value);
    }
    info!("Set up slack variables");
    debug!("{:?}", matrix);

    Ok(matrix)
}

pub fn simplex(matrix: &mut Matrix) -> Result<Solution> {
    info!("Initializing simplex solver");
    debug!("Problem:");
    debug!("{:?}", matrix);

    let mut iteration = 0;
    loop {
        info!("\nIteration {}", iteration);
        debug!("{:?}", matrix);
        if iteration >= matrix.width() * matrix.height() {
            warn!("Warning.  Failed to find solution after {} iterations", iteration);
            return SolverError::unable_to_solve("Failed to find solution");
        }

        let objective_columns = find_valid_objective_columns(matrix);
        if objective_columns.is_empty() { break; }

        let pivot_col = select_pivot_column(matrix, &objective_columns);
        info!("Selected pivot column {:?}", pivot_col);

        let pivot_row = match select_pivot_row(matrix, pivot_col) {
            None => {
                warn!("Unable to find a pivot row.  function is unbounded below.");
                return SolverError::infeasible("Function is unbounded below.");
            }, Some(row) => row,
        };
        info!("Selected pivot row {:?}", pivot_row);

        simplex_pivot(matrix, pivot_row, pivot_col);
        info!("Completed pivot and moving to next iteration.");

        iteration += 1;
    }

    info!("Simplex solve complete.");

    produce_solution(matrix)
}

fn produce_solution(matrix: &Matrix) -> Result<Solution> {
    let objective = Some(matrix.value(matrix.first_row(), matrix.last_col()));
    let mut coeffs = vec![0.0; matrix.num_variables()];

    for col in matrix.cols_range(matrix.first_col() + 1, matrix.last_col()) {
        let mut ident_row = None;
        let mut skip = false;

        for row in matrix.rows() {
            if matrix.value(row, col) != 0.0 {
                if ident_row.is_some() {
                    skip = true;
                    break;
                }

                ident_row = Some(row);
            }
        }

        if skip { continue; }

        if let Some(row) = ident_row {
            let index = col.index() - 1;
            if index < coeffs.len() {
                coeffs[index] = matrix.value(row, matrix.last_col());
            }
        }
    }

    let solution = Solution::new(coeffs, objective);
    info!("Solution found {:?}", solution);
    Ok(solution)
}

fn find_valid_objective_columns(matrix: &Matrix) -> Vec<Col> {
    let mut result = Vec::new();
    for col in matrix.cols_from(matrix.first_col() + 1) {
        let value = matrix.value(matrix.first_row(), col);
        match matrix.objective_kind() {
            ObjectiveKind::Minimize => if value > 0.0 { result.push(col); }
            ObjectiveKind::Maximize => if value < 0.0 { result.push(col); }
        }
    }

    result
}

fn select_pivot_column(_matrix: &Matrix, valid_cols: &[Col]) -> Col {
    if valid_cols.is_empty() { panic!(); }

    // TODO select the best column through an algorithm such as devex
    valid_cols[0]
}

fn select_pivot_row(matrix: &Matrix, pivot_col: Col) -> Option<Row> {
    let mut candidates = Vec::new();

    for row in matrix.rows_from(matrix.first_row() + 1) {
        if matrix.value(row, pivot_col) > 0.0 {
            candidates.push(row);
        }
    }

    if candidates.is_empty() { return None; }

    let mut min = f64::MAX;
    let mut min_row = candidates[0];
    for row in candidates {
        let min_ratio_test = matrix.value(row, matrix.last_col()) /
            matrix.value(row, pivot_col);

        if min_ratio_test < min {
            min_row = row;
            min = min_ratio_test;
        }
    }

    return Some(min_row);
}

fn simplex_pivot(matrix: &mut Matrix, pivot_row: Row, pivot_col: Col) {
    let pivot_recip = 1.0 / matrix.value(pivot_row, pivot_col);
    matrix.multiply_row(pivot_row, pivot_recip);

    debug!("Pivot value {:?},{:?} set to 1", pivot_row, pivot_col);

    for row in matrix.rows() {
        if row == pivot_row { continue; }

        let mult = -1.0 * matrix.value(row, pivot_col);
        matrix.add_row(pivot_row, row, mult);
    }
}
