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

use log::{debug, info};

use crate::{Matrix, Row, Col, Problem, Solution, Result, SolverError};

pub fn setup_matrix(problem: &Problem) -> Result<Matrix> {
    if problem.objective_kind().is_some() {
        return SolverError::invalid_objective("Gaussian elimination does not accept\
                an objective function.");
    }

    if problem.num_constraints() != problem.num_variables() {
        return SolverError::invalid_constraint("Number of constraints must equal number of\
                variables for gaussian elimination.");
    }

    info!("Set up gaussian elimination with {} constraints and {} variables",
          problem.num_constraints(), problem.num_variables());

    let height = problem.num_constraints();
    let width = problem.num_variables() + 1;
    let mut coeffs = vec![0.0;width*height];

    for (row, constraint) in problem.constraints().iter().enumerate() {
        use crate::ConstraintKind::*;
        match constraint.kind() {
            GreaterThanOrEqualTo | LessThanOrEqualTo => {
                return SolverError::invalid_constraint("Gaussian elimination only accepts\
                        equality constraints.");
            }, EqualTo => (),
        }

        for (col, value) in constraint.expr().iter() {
            coeffs[*col as usize + row * width] = *value;
        }
        coeffs[width - 1 + row * width] = constraint.constant();
    }

    Ok(Matrix::new_gaussian_elimination(width, height, problem.num_variables(), coeffs))
}

pub fn gaussian_elimination(matrix: &mut Matrix) -> Result<Solution> {
    let mut pivot_row = matrix.first_row();
    let mut pivot_col = matrix.first_col();

    info!("Performing gaussian elimination");
    debug!("{:?}", matrix);

    info!("Reducing matrix to triangular form");
    while pivot_row.is_valid(matrix) && pivot_col.is_valid(matrix) {
        let pivot_max = find_pivot_max(matrix, pivot_row, pivot_col);

        if matrix.value(pivot_max, pivot_col) == 0.0 {
            pivot_col += 1;
        } else {
            matrix.swap_rows(pivot_row, pivot_max);

            for row in matrix.rows_from(pivot_row + 1) {
                let coeff = matrix.value(row, pivot_col) / matrix.value(pivot_row, pivot_col);
                matrix.set_value(row, pivot_col, 0.0);

                for col in matrix.cols_from(pivot_col + 1) {
                    let value = matrix.value(row, col) - matrix.value(pivot_row, col) * coeff;
                    matrix.set_value(row, col, value);
                }
            }

            pivot_row += 1;
            pivot_col += 1;
        }

        debug!("Step");
        debug!("{:?}", matrix);

        if matrix.has_zero_row() {
            return SolverError::underspecified("Two or more rows are linearly dependent.");
        }
    }

    info!("Back substituting");
    let mut coeffs = vec![0.0; matrix.height()];
    for row in matrix.rows().rev() {
        let mut coeff = matrix.value(row, matrix.last_col());
        for col in matrix.cols_range(Col::from(row + 1), matrix.last_col()) {
            coeff -= matrix.value(row, col) * coeffs[col.index()];
        }

        coeffs[row.index()] = coeff / matrix.value(row, Col::from(row));

        debug!("Current solution:");
        debug!("{:?}", coeffs);
    }

    let solution = Solution::new(coeffs, None);
    info!("Solution found: {:?}", solution);
    Ok(solution)
}

fn find_pivot_max(matrix: &Matrix, cur_pivot_row: Row, pivot_col: Col) -> Row {
    let mut max_value = 0.0;
    let mut pivot_max = cur_pivot_row;
    for row in matrix.rows_from(cur_pivot_row) {
        let cur_value = matrix.value(row, pivot_col).abs();
        if cur_value > max_value {
            max_value = cur_value;
            pivot_max = row;
        }
    }
    pivot_max
}
