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

const OPTIMALITY_TOL: f64 = 1e-6;

pub fn solve(problem: &Problem) -> Result<Solution> {
    let num_variables = problem.num_variables();
    let objective_kind = match problem.objective_kind() {
        None => {
            return SolverError::invalid_objective("Must set an objective for simplex.");
        }, Some(kind) => kind,
    };

    let (mut matrix, num_artificial) = setup_matrix(problem)?;
    info!("Set up {} artificial variables", num_artificial);

    info!("Performing Phase I simplex solve");
    let phase1 = simplex(&mut matrix, num_variables, 2, ObjectiveKind::Minimize)?;

    if let Some(solution) = phase1.objective() {
        if solution.abs() > OPTIMALITY_TOL {
            return SolverError::infeasible("No feasible solution exists.");
        }
    }

    info!("Performing Phase II simplex solve");
    matrix.sub(1, 1);
    let last_col = matrix.last_col();
    for row in matrix.rows() {
        for col in matrix.cols_range(last_col - num_artificial, last_col) {
            matrix.set_value(row, col, 0.0);
        }
    }

    info!("Phase 2 Matrix:");
    info!("{:?}", matrix);
    simplex(&mut matrix, num_variables, 1, objective_kind)
}

pub fn setup_matrix(problem: &Problem) -> Result<(Matrix, usize)> {
    info!("Set up simplex problem with {} constraints and {} variables",
          problem.num_constraints(), problem.num_variables());

    let height = problem.num_constraints() + 2;
    let width = problem.num_variables() + problem.num_constraints() + 3;
    let mut coeffs = vec![0.0;width*height];

    let mut row = 2;
    for constraint in problem.constraints().iter() {
        let neg = if constraint.constant() < 0.0 { -1.0 } else { 1.0 };

        for (index, value) in constraint.expr().iter() {
            coeffs[2 + *index as usize + row * width] = *value * neg;
        }
        coeffs[width - 1 + row * width] = constraint.constant() * neg;

        row += 1;
    }

    coeffs[0] = 1.0;
    if let Some(objective) = problem.objective_expr() {
        let row = 1;
        for (index, value) in objective.iter(){
            coeffs[2 + *index as usize + row * width] = *value * -1.0;
        }
        coeffs[1 + row * width] = 1.0;
    }

    let mut matrix = Matrix::new(width, height, coeffs);
    info!("Set up initial problem.");
    debug!("{:?}", matrix);

    let mut num_artificial = 0;
    let slack_start_row = problem.num_variables() + 2;
    for (i, constraint) in problem.constraints().iter().enumerate() {
        let (value, artificial) = match constraint.kind() {
            ConstraintKind::LessThanOrEqualTo => (1.0, false),
            ConstraintKind::GreaterThanOrEqualTo => (-1.0, false),
            ConstraintKind::EqualTo => (1.0, true),
        };
        matrix.set_value_raw(i + 2, i + slack_start_row, value);

        if artificial {
            matrix.set_value_raw(0, i + slack_start_row, -1.0);
            matrix.add_row(Row::new(i + 2), Row::new(0), 1.0);
            num_artificial += 1;
        }
    }
    info!("Set up slack and artifical variables");
    debug!("{:?}", matrix);

    Ok((matrix, num_artificial))
}

pub fn simplex(matrix: &mut Matrix,
               num_variables: usize,
               objective_rows: usize,
               objective_kind: ObjectiveKind) -> Result<Solution> {
    info!("Initializing simplex solver");

    let mut iteration = 0;
    loop {
        info!("Iteration {}", iteration);
        debug!("{:?}", matrix);
        if iteration >= matrix.width() * matrix.height() {
            warn!("Warning.  Failed to find solution after {} iterations", iteration);
            return SolverError::unable_to_solve("Failed to find solution");
        }

        let pivot_col = match select_pivot_column(matrix, objective_kind) {
            None => {
                info!("No available pivot columns - solution is optimal");
                break;
            }, Some(col) => col,
        };
        info!("Selected pivot column {:?}", pivot_col);

        let pivot_row = match select_pivot_row(matrix, pivot_col, objective_rows) {
            None => {
                warn!("Unable to find a pivot row.  function is unbounded.");
                return SolverError::infeasible("Function is unbounded.");
            }, Some(row) => row,
        };
        info!("Selected pivot row {:?}", pivot_row);

        simplex_pivot(matrix, pivot_row, pivot_col);
        info!("Completed pivot and moving to next iteration.");

        iteration += 1;
    }

    info!("Simplex solve complete.");

    produce_solution(matrix, num_variables)
}

fn produce_solution(matrix: &Matrix, num_variables: usize) -> Result<Solution> {
    let objective = Some(matrix.value(matrix.first_row(), matrix.last_col()));
    let mut coeffs = vec![0.0; num_variables];

    let mut out_index = 0;
    let first_col = matrix.first_col();
    for col in matrix.cols_range(first_col + 1, first_col + 1 + coeffs.len()) {
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

        if !skip {
            if let Some(row) = ident_row {
                coeffs[out_index] = matrix.value(row, matrix.last_col());
            }
        }

        out_index += 1;
    }

    let solution = Solution::new(coeffs, objective);
    info!("Solution found {:?}", solution);
    Ok(solution)
}

fn select_pivot_column(matrix: &Matrix, objective_kind: ObjectiveKind) -> Option<Col> {
    let mut best_val = 0.0;
    let mut best_col = None;

    for col in matrix.cols_range(matrix.first_col() + 1, matrix.last_col()) {
        let value = matrix.value(matrix.first_row(), col);
        match objective_kind {
            ObjectiveKind::Minimize => {
                if value > best_val {
                    best_col = Some(col);
                    best_val = value;
                }
            },
            ObjectiveKind::Maximize => {
                if value < best_val {
                    best_col = Some(col);
                    best_val = value;
                }
            }
        }
    }

    best_col
}

fn select_pivot_row(matrix: &Matrix, pivot_col: Col, num_rows_skip: usize) -> Option<Row> {
    let mut min = f64::MAX;
    let mut min_row = None;

    for row in matrix.rows_from(matrix.first_row() + num_rows_skip) {
        let value = matrix.value(row, pivot_col);
        if value <= 0.0 { continue; }

        let min_ratio_test = matrix.value(row, matrix.last_col()) / value;

        if min_ratio_test < 0.0 { continue; }

        if min_ratio_test < min {
            min_row = Some(row);
            min = min_ratio_test;
        }
    }

    return min_row
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
