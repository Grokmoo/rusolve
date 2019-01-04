//  This file is part of rusolve, an optimizer / solver written in Rust.
//  Copyright 2018 Jared Stephen
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

use std::io::Error;

use rusolve::{Constraint, Problem, create_problem, create_constraint};

fn main() -> Result<(), Error> {
    let mut problem = create_problem!( [ 3.0, 2.0, 1.0; 10.0],
                                       [ 2.0, 5.0, 3.0; 15.0]);

    let obj = create_constraint!(-2.0, -3.0, -4.0; 0.0);
    problem.set_objective(obj);

    let solution = problem.solve()?;

    println!("Solution:");
    println!("{:#?}", solution);
    Ok(())
}
