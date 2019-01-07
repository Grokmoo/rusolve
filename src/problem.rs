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
use std::collections::HashMap;

use crate::{Result, simplex, gaussian_elimination};

#[derive(Copy, Clone, Debug)]
pub struct Variable {
    index: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum ObjectiveKind {
    Minimize,
    Maximize,
}

#[derive(Debug, Clone, Copy)]
pub enum ConstraintKind {
    GreaterThanOrEqualTo,
    EqualTo,
    LessThanOrEqualTo,
}

#[derive(Debug)]
pub struct Expression {
    coeffs: HashMap<u32, f64>,
}

impl Default for Expression {
    fn default() -> Self { Expression { coeffs: HashMap::new() } }
}

impl Expression {
    pub fn new(coeffs: &[f64]) -> Expression {
        let mut expr = Expression::default();

        for (index, coeff) in coeffs.iter().enumerate() {
            expr.coeffs.insert(index as u32, *coeff);
        }

        expr
    }

    pub fn add_coeffs(&mut self, coeffs: &[f64]) {
        for (index, coeff) in coeffs.iter().enumerate() {
            self.coeffs.insert(index as u32, *coeff);
        }
    }

    pub fn add_term(&mut self, coeff: f64, variable: Variable) {
        self.coeffs.insert(variable.index, coeff);
    }

    pub fn iter(&self) -> impl Iterator<Item=(&u32, &f64)> {
        self.coeffs.iter()
    }
}

#[derive(Debug)]
pub struct Constraint {
    expr: Expression,
    kind: ConstraintKind,
    constant: f64,
}

impl Constraint {
    pub fn new(expr: Expression, kind: ConstraintKind, constant: f64) -> Constraint {
        Constraint {
            expr,
            kind,
            constant,
        }
    }

    pub fn kind(&self) -> ConstraintKind { self.kind }

    pub fn expr(&self) -> &Expression { &self.expr }

    pub fn constant(&self) -> f64 {
        self.constant
    }
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

    pub fn value(&self, index: usize) -> f64 {
        self.coeffs[index]
    }

    pub fn iter(&self) -> impl Iterator<Item=&f64> {
        self.coeffs.iter()
    }

    pub fn objective(&self) -> Option<f64> {
        self.objective
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
    ( $( [ $($val:expr),* ; $kind:expr ; $constant:expr ] ),* ) => {
        {
            let mut problem = Problem::new();
            $(
                let constraint = create_constraint!($($val),*; $kind ; $constant);
                problem.add_constraint(constraint);
            )*
            problem
        }
    }
}

#[macro_export]
macro_rules! create_expr {
    ( $($val:expr),* ) => {
        {
            let mut coeffs = Vec::new();
            $(
                coeffs.push($val);
            )*
            Expression::new(&coeffs)
        }
    }
}

#[macro_export]
macro_rules! create_constraint {
    ( $($val:expr),* ; $kind:expr ; $constant:expr ) => {
        {
            let mut expr = Expression::default();
            let mut coeffs = Vec::new();
            $(
                coeffs.push($val);
            )*
            expr.add_coeffs(&coeffs);

            match stringify!($kind) {
                "le" => Constraint::new(expr, ConstraintKind::LessThanOrEqualTo, $constant),
                "ge" => Constraint::new(expr, ConstraintKind::GreaterThanOrEqualTo, $constant),
                "eq" => Constraint::new(expr, ConstraintKind::EqualTo, $constant),
                _ => panic!("Invalid constraint type"),
            }
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

#[derive(Debug)]
struct Objective {
    expr: Expression,
    kind: ObjectiveKind,
}

#[derive(Debug)]
pub struct Problem {
    max_variable: u32,
    constraints: Vec<Constraint>,
    objective: Option<Objective>,

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

    pub fn constraint(&self, index: usize) -> &Constraint {
        &self.constraints[index]
    }

    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }

    pub fn objective_kind(&self) -> Option<ObjectiveKind> {
        match &self.objective {
            None => None,
            Some(obj) => Some(obj.kind),
        }
    }

    pub fn objective(&self) -> Option<&Expression> {
        match &self.objective {
            None => None,
            Some(obj) => Some(&obj.expr)
        }
    }

    pub fn num_variables(&self) -> usize {
        self.max_variable as usize
    }

    pub fn set_objective(&mut self, expr: Expression, kind: ObjectiveKind) {
        self.objective = Some(Objective { expr, kind });
    }

    pub fn add_row(&mut self, values: &[f64], kind: ConstraintKind, constant: f64) {
        if self.max_variable < values.len() as u32 {
            self.max_variable = values.len() as u32;
        }

        let expr = Expression::new(values);
        let constraint = Constraint::new(expr, kind, constant);
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
        if self.max_variable < constraint.expr.coeffs.len() as u32 {
            self.max_variable = constraint.expr.coeffs.len() as u32;
        }
        self.constraints.push(constraint);
    }

    pub fn solve(&self) -> Result<Solution> {
        match self.objective {
            None => gaussian_elimination::solve(self),
            Some(_) => simplex::solve(self),
        }
    }
}
