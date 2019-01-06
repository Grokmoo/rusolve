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

use std::sync::{Once, ONCE_INIT};

use env_logger;

use rusolve::{SolverError, ErrorKind, Result, Problem};

static INIT: Once = ONCE_INIT;

pub fn setup() -> Result<()> {
    INIT.call_once(|| {
        env_logger::init()
    });

    Ok(())
}

const TOLERANCE: f64 = 1e-6;

pub fn solution_err(problem: Problem, kind: ErrorKind) -> Result<()> {
    match problem.solve() {
        Err(error) => {
            if error.kind() == kind { Ok(()) }
            else {
                SolverError::invalid_solution(format!("Expected error {}, got {}", kind, error.kind()))
            }
        },
        Ok(_) => SolverError::invalid_solution(format!("Expected solver error {}", kind)),
    }
}

pub fn solution_eq(problem: Problem, vars: Vec<f64>, objective: Option<f64>) -> Result<()> {
    let solution = problem.solve()?;

    for index in 0..vars.len() {
        if (solution.value(index) - vars[index]).abs() > TOLERANCE {
            let got: Vec<_> = solution.iter().collect();
            return SolverError::invalid_solution(format!("Expected solution {:?}, got {:?}",
                                                         vars, got));
        }
    }

    if let Some(found_obj) = solution.objective() {
        if let Some(obj) = objective {
            if (obj - found_obj).abs() > TOLERANCE {
                SolverError::invalid_solution(format!("Objective {} != {}", obj, found_obj))
            } else {
                Ok(())
            }
        } else {
            SolverError::invalid_solution(format!("Objective {} but None specified", found_obj))
        }
    } else {
        if let Some(obj) = objective {
            SolverError::invalid_solution(format!("Objective None but {} specified", obj))
        } else {
            Ok(())
        }
    }
}
