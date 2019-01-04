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
use std::fmt;
use std::io::{Error};
use std::collections::HashMap;

use crate::Matrix;

#[derive(Copy, Clone, Debug)]

pub struct Variable {
    index: u32,
}

#[derive(Debug)]
pub struct Constraint {
    coeffs: HashMap<u32, f64>,
    constant: f64,
}

pub struct Solution {
    coeffs: Vec<f64>,
    objective: Option<f64>,
}

impl PartialEq for Solution {
    fn eq(&self, other: &Solution) -> bool {
        if self.objective != other.objective { return false; }

        if self.coeffs.len() != other.coeffs.len() { return false; }

        for i in 0..self.coeffs.len() {
            if self.coeffs[i] != other.coeffs[i] { return false; }
        }

        return true;
    }
}

impl Solution {
    pub fn new(coeffs: Vec<f64>, objective: Option<f64>) -> Solution {
        Solution {
            coeffs,
            objective,
        }
    }
}

impl fmt::Debug for Solution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(val) = self.objective {
            writeln!(f, "Objective = {:.6}", val)?;
        }

        for (index, value) in self.coeffs.iter().enumerate() {
            writeln!(f, "x[{}] = {:.6}", index, value)?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! create_problem {
    ( $( [ $($val:expr),*; $constant:expr ] ),* ) => {
        {
            let mut problem = Problem::new();
            $(
                let mut values = Vec::new();
                $(
                    values.push($val);
                )*
                problem.add_row(&values, $constant);
            )*
            problem
        }
    }
}

#[macro_export]
macro_rules! create_constraint {
    ( $($val:expr),*; $constant:expr ) => {
        {
            let mut constraint = Constraint::new();
            constraint.set_value($constant);
            let mut coeffs = Vec::new();
            $(
                coeffs.push($val);
            )*
                constraint.add_coeffs(&coeffs);
            constraint
        }
    }
}

#[macro_export]
macro_rules! add_variables {
    ( $problem:expr, $( $var:ident ),* ) => {
        $(
            let $var = $problem.add_variable();
        )*
    };
}

#[macro_export]
macro_rules! add_constraint {
    ( $problem:expr, $( $var:ident => $val:expr ),*; $constant_term:expr) => {
        {
            let mut constraint = Constraint::new();
            constraint.set_value($constant_term);
            $(
                constraint.add_term($val, $var);
            )*
            $problem.add_constraint(constraint);
        }
    }
}

#[derive(Debug)]
pub struct Problem {
    max_variable: u32,
    constraints: Vec<Constraint>,
    objective: Option<Constraint>,
}

impl Problem {
    pub fn new() -> Problem {
        Problem {
            max_variable: 0,
            constraints: Vec::new(),
            objective: None,
        }
    }

    pub fn num_constraints(&self) -> usize {
        self.constraints.len()
    }

    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }

    pub fn objective(&self) -> Option<&Constraint> {
        self.objective.as_ref()
    }

    pub fn num_variables(&self) -> usize {
        self.max_variable as usize
    }

    pub fn set_objective(&mut self, objective: Constraint) {
        self.objective = Some(objective);
    }

    pub fn add_row(&mut self, values: &[f64], constant: f64) {
        if self.max_variable < values.len() as u32 {
            self.max_variable = values.len() as u32;
        }

        let mut coeffs = HashMap::new();
        for i in 0..values.len() {
            coeffs.insert(i as u32, values[i]);
        }

        let constraint = Constraint {
            coeffs,
            constant
        };
        self.add_constraint(constraint);
    }

    pub fn add_variable(&mut self) -> Variable {
        let variable = Variable {
            index: self.max_variable,
        };

        self.max_variable += 1;

        variable
    }

    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    pub fn solve(&self) -> Result<Solution, Error> {
        let mut matrix = Matrix::new(self);
        Ok(matrix.simplex())
    }
}

impl Constraint {
    pub fn new() -> Constraint {
        Constraint {
            coeffs: HashMap::new(),
            constant: 0.0,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=(&u32, &f64)> {
        self.coeffs.iter()
    }

    pub fn constant(&self) -> f64 {
        self.constant
    }

    pub fn set_value(&mut self, constant: f64) {
        self.constant = constant;
    }

    pub fn add_coeffs(&mut self, coeffs: &[f64]) {
        for (index, coeff) in coeffs.iter().enumerate() {
            self.coeffs.insert(index as u32, *coeff);
        }
    }

    pub fn add_term(&mut self, coeff: f64, variable: Variable) {
        self.coeffs.insert(variable.index, coeff);
    }
}
