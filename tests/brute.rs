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

use rusolve::{Constraint, ConstraintKind, Problem, Expression, ObjectiveKind, Result, ErrorKind,
    create_constraints, create_expr, create_constraint};

mod common;
use crate::common::{setup, solution_eq, solution_err};

#[test]
fn brute_no_obj() -> Result<()> {
    setup()?;

    let mut problem = Problem::boolean(1);
    problem.add_constraints(create_constraints!(
            [1.0 ;eq; 1.0])
    )?;

    solution_err(problem, ErrorKind::UnableToSolve)
}

#[test]
fn brute_infeasible() -> Result<()> {
    setup()?;

    let mut problem = Problem::boolean(2);
    problem.add_constraints(create_constraints!(
        [1.0, 1.0 ;eq; 1.0],
        [1.0, 1.0 ;eq; 2.0])

    )?;
    problem.set_objective(create_expr!(1.0, 1.0), ObjectiveKind::Maximize);
    solution_err(problem, ErrorKind::Infeasible)
}

#[test]
fn brute_one_var() -> Result<()> {
    setup()?;

    let mut problem = Problem::boolean(1);
    problem.add_constraints(create_constraints!(
        [1.0 ;eq; 1.0])
    )?;
    problem.set_objective(create_expr!(1.0), ObjectiveKind::Maximize);
    solution_eq(problem, vec![1.0], Some(1.0))
}

#[test]
fn brute_simple() -> Result<()> {
    setup()?;

    let mut problem = Problem::boolean(3);
    problem.add_constraints(create_constraints!(
            [1.0, 0.0, 1.0 ;eq; 2.0],
            [0.0, 1.0, 1.0 ;eq; 2.0])
    )?;
    problem.set_objective(create_expr!(1.0, 2.0, 3.0), ObjectiveKind::Maximize);
    solution_eq(problem, vec![1.0, 1.0, 1.0], Some(6.0))
}

#[test]
fn brute_bigger() -> Result<()> {
    setup()?;

    let mut problem = Problem::boolean(5);
    problem.add_constraints(create_constraints!(
        [1.0, 1.0, 1.0, 1.0, 1.0 ;eq; 2.0],
        [3.0, 2.0, 1.0, 1.0, 2.0 ;eq; 3.0],
        [-2.0, -1.0, -1.0, -3.0, -2.0 ;eq; -4.0])
    )?;
    problem.set_objective(
        create_expr!(4.0, 5.0, 3.0, 4.0, 5.0),
        ObjectiveKind::Maximize
    );
    solution_eq(problem, vec![0.0, 1.0, 0.0, 1.0, 0.0], Some(9.0))
}
