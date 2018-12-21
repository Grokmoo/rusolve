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

use rusolve::{Problem, Constraint};

fn main() -> Result<(), Error> {
    let mut problem = Problem::new();

    let x = problem.add_variable();
    let y = problem.add_variable();
    let z = problem.add_variable();

    let mut l1 = Constraint::new();
    l1.set_value(8.0);
    l1.add_term(2.0, x);
    l1.add_term(1.0, y);
    l1.add_term(-1.0, z);
    problem.add_constraint(l1);

    let mut l2 = Constraint::new();
    l2.set_value(-11.0);
    l2.add_term(-3.0, x);
    l2.add_term(-1.0, y);
    l2.add_term(2.0, z);
    problem.add_constraint(l2);

    let mut l3 = Constraint::new();
    l3.set_value(-3.0);
    l3.add_term(-2.0, x);
    l3.add_term(1.0, y);
    l3.add_term(2.0, z);
    problem.add_constraint(l3);

    let solution = problem.solve()?;

    println!("Solution: {:#?}", solution);
    Ok(())
}
