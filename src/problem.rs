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

use crate::{Result, SolverError, simplex, gaussian_elimination, brute};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VariableKind {
    Continuous,
    Integer(i32, i32),
}

#[derive(Copy, Clone, Debug)]
pub struct Variable {
    index: u32,
    kind: VariableKind,
}

impl Variable {
    pub fn kind(&self) -> VariableKind { self.kind }
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

    pub fn get(&self, index: u32) -> f64 {
        *self.coeffs.get(&index).unwrap_or(&0.0)
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
macro_rules! create_constraints {
    ( $( [ $($val:expr),* ; $kind:expr ; $constant:expr ] ),* ) => {
        {
            let mut constraints = Vec::new();
            $(
                let constraint = create_constraint!($($val),*; $kind ; $constant);
                constraints.push(constraint);
            )*
            constraints
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

#[derive(Debug)]
struct Objective {
    expr: Expression,
    kind: ObjectiveKind,
}

#[derive(Debug)]
pub struct Problem {
    variables: Vec<Variable>,
    constraints: Vec<Constraint>,
    objective: Option<Objective>,
}

impl Problem {
    pub fn new() -> Problem {
        Problem {
            variables: Vec::new(),
            constraints: Vec::new(),
            objective: None,
        }
    }

    pub fn boolean(vars: u32) -> Problem {
        let mut problem = Problem::new();
        problem.add_variables(VariableKind::Integer(0, 1), vars);
        problem
    }

    pub fn continuous(vars: u32) -> Problem {
        let mut problem = Problem::new();
        problem.add_variables(VariableKind::Continuous, vars);
        problem
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

    pub fn objective(&self) -> Option<(&Expression, ObjectiveKind)> {
        match &self.objective {
            None => None,
            Some(obj) => Some((&obj.expr, obj.kind)),
        }
    }

    pub fn objective_kind(&self) -> Option<ObjectiveKind> {
        match &self.objective {
            None => None,
            Some(obj) => Some(obj.kind),
        }
    }

    pub fn objective_expr(&self) -> Option<&Expression> {
        match &self.objective {
            None => None,
            Some(obj) => Some(&obj.expr)
        }
    }

    pub fn variables(&self) -> impl Iterator<Item=&Variable> {
        self.variables.iter()
    }

    pub fn num_variables(&self) -> usize {
        self.variables.len()
    }

    pub fn set_objective(&mut self, expr: Expression, kind: ObjectiveKind) {
        self.objective = Some(Objective { expr, kind });
    }

    pub fn add_row(
        &mut self,
        values: &[f64],
        kind: ConstraintKind,
        constant: f64
    ) -> Result<()> {
        let expr = Expression::new(values);
        let constraint = Constraint::new(expr, kind, constant);
        self.add_constraint(constraint)
    }

    pub fn add_variables(&mut self, kind: VariableKind, num: u32) {
        for i in 0..num {
            let variable = Variable {
                index: i,
                kind: kind,
            };
            self.variables.push(variable);
        }
    }

    pub fn add_variable(&mut self, kind: VariableKind) {
        let index = self.variables.len();
        let variable = Variable {
            index: index as u32,
            kind: kind,
        };
        self.variables.push(variable);
    }

    pub fn add_constraints(&mut self, constraints: Vec<Constraint>) -> Result<()> {
        for constraint in constraints {
            self.add_constraint(constraint)?;
        }

        Ok(())
    }

    pub fn add_constraint(&mut self, constraint: Constraint) -> Result<()> {
        if constraint.expr.coeffs.len() != self.variables.len() {
            return SolverError::invalid_constraint(
                format!("Expected {} vars in constraint, got {}.",
                    self.variables.len(), constraint.expr.coeffs.len()));
        }
        self.constraints.push(constraint);
        Ok(())
    }

    pub fn solve(&self) -> Result<Solution> {
        use VariableKind::*;
        let mut var_kind = Continuous;
        for var in &self.variables {
            if let Integer(..) = var.kind {
                var_kind = Integer(0, 0);
            }
        }


        match self.objective {
            None => {
                match var_kind {
                    Continuous => gaussian_elimination::solve(self),
                    Integer(..) => SolverError::unable_to_solve(
                        "A mixed integer or integer problem must specify an objective."),
                }
            },
            Some(_) => {
                match var_kind {
                    Continuous => simplex::solve(self),
                    Integer(..) => brute::solve(self),
                }
            },
        }
    }
}
