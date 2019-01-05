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

use rusolve::{Constraint, ConstraintKind, Problem, Expression, ObjectiveKind, Result,
    Solution, create_problem, create_expr, create_constraint};

mod common;
use crate::common::setup;

#[test]
fn simplex_minimize() -> Result<()> {
    setup()?;
    let mut problem = create_problem!( [ 3.0, 2.0, 1.0; <= 10.0],
                                       [ 2.0, 5.0, 3.0; <= 15.0]);
    problem.set_objective(create_expr!(-2.0, -3.0, -4.0), ObjectiveKind::Minimize);

    assert_eq!(problem.solve()?, Solution::new(vec![0.0, 0.0, 5.0], Some(-20.0)));
    Ok(())
}

#[test]
fn simplex_maximize() -> Result<()> {
    setup()?;
    let mut problem = create_problem!( [ 3.0, 2.0, 1.0; <= 10.0 ],
                                       [ 2.0, 5.0, 3.0; <= 15.0 ]);
    problem.set_objective(create_expr!(2.0, 3.0, 4.0), ObjectiveKind::Maximize);

    assert_eq!(problem.solve()?, Solution::new(vec![0.0, 0.0, 5.0], Some(20.0)));
    Ok(())
}
