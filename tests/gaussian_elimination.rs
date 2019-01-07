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

use rusolve::{Constraint, ConstraintKind, Problem, Expression, Result,
    create_problem, create_constraint, ErrorKind};

mod common;
use crate::common::{setup, solution_eq, solution_err};

#[test]
fn gaussian_lin_dep() -> Result<()> {
    setup()?;
    let problem = create_problem!( [2.0, 1.0, 1.0 ;eq; 3.0],
                                   [4.0, 2.0, 2.0 ;eq; 6.0],
                                   [1.0, 0.0, 1.0 ;eq; 1.5]);
    solution_err(problem, ErrorKind::Underspecified)
}

#[test]
fn gaussian_overspecified() -> Result<()> {
    setup()?;
    let problem = create_problem!( [2.0, 1.0, 1.0 ;eq; 3.0],
                                   [1.0, 0.0, 1.0 ;eq; 1.5],
                                   [2.0, 1.0, 0.0 ;eq; 2.0],
                                   [4.0, 2.0, 2.0 ;eq; 6.0]);
    solution_err(problem, ErrorKind::InvalidConstraint)
}

#[test]
fn gaussian_underspecified() -> Result<()> {
    setup()?;
    let problem = create_problem!( [2.0, 1.0, 1.0 ;eq; 3.0],
                                   [1.0, 0.0, 1.0 ;eq; 1.5]);
    solution_err(problem, ErrorKind::InvalidConstraint)
}

#[test]
fn gaussian_elim_simple() -> Result<()> {
    setup()?;
    let problem = create_problem!( [2.0, 1.0, 1.0 ;eq; 3.0],
                                   [1.0, 0.0, 1.0 ;eq; 1.5],
                                   [2.0, 1.0, 0.0 ;eq; 2.0]);
    solution_eq(problem, vec![0.5, 1.0, 1.0], None)}

#[test]
fn gaussian_elim_reduced() -> Result<()> {
    setup()?;
    let problem = create_problem!( [2.0, 1.0, 1.0 ;eq; 3.0],
                                   [0.0, 1.0, 1.0 ;eq; 2.0],
                                   [0.0, 0.0, 1.0 ;eq; 1.0]);
    solution_eq(problem, vec![0.5, 1.0, 1.0], None)
}
