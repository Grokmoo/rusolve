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
fn simplex_neg_b() -> Result<()> {
    setup()?;

    let mut problem = Problem::continuous(2);
    problem.add_constraints(create_constraints!(
            [1.0, -1.0 ;le; -5.0],
            [1.0,  0.0 ;le; 10.0])
    )?;
    problem.set_objective(create_expr!(2.0, 3.0), ObjectiveKind::Maximize);
    solution_eq(problem, vec![10.0, 15.0], Some(65.0))
}

#[test]
fn simplex_eq() -> Result<()> {
    setup()?;

    let mut problem = Problem::continuous(3);
    problem.add_constraints(create_constraints!(
            [3.0, 2.0, 1.0 ;eq; 10.0 ],
            [2.0, 5.0, 3.0 ;eq; 15.0 ])
    )?;
    problem.set_objective(create_expr!(-2.0, -3.0, -4.0), ObjectiveKind::Minimize);
    solution_eq(problem, vec![2.142857, 0.0, 3.571429], Some(-18.571429))
}

#[test]
fn simplex_ge() -> Result<()> {
    setup()?;

    let mut problem = Problem::continuous(2);
    problem.add_constraints(create_constraints!(
            [1.0, 1.0 ;le; 10.0],
            [1.0, 2.0 ;le; 15.0])
    )?;
    problem.set_objective(create_expr!(2.0, 3.0), ObjectiveKind::Maximize);
    solution_eq(problem, vec![5.0, 5.0], Some(25.0))
}

#[test]
fn simplex_minimize() -> Result<()> {
    setup()?;

    let mut problem = Problem::continuous(3);
    problem.add_constraints(create_constraints!(
            [ 3.0, 2.0, 1.0 ;le; 10.0],
            [ 2.0, 5.0, 3.0 ;le; 15.0])
    )?;
    problem.set_objective(create_expr!(-2.0, -3.0, -4.0), ObjectiveKind::Minimize);
    solution_eq(problem, vec![0.0, 0.0, 5.0], Some(-20.0))
}

#[test]
fn simplex_unbounded() -> Result<()> {
    setup()?;

    let mut problem = Problem::continuous(3);
    problem.add_constraints(create_constraints!(
            [1.0, 1.0, 1.0 ;ge; 0.0 ])
    )?;
    problem.set_objective(create_expr!(1.0, 1.0, 1.0), ObjectiveKind::Maximize);
    solution_err(problem, ErrorKind::Infeasible)
}
