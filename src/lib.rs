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

use std::fmt;
use std::error;

pub mod matrix;
pub mod problem;

mod gaussian_elimination;
mod simplex;
mod brute;

pub use crate::problem::{Variable, Constraint, ConstraintKind,
    ObjectiveKind, Expression, Problem, Solution, VariableKind};
pub use crate::matrix::{Matrix, Row, Col};

pub type Result<T> = std::result::Result<T, SolverError>;

#[derive(Debug, Clone)]
pub struct SolverError {
    message: String,
    kind: ErrorKind,
}

impl SolverError {
    pub fn new<T, M: Into<String>>(kind: ErrorKind, message: M) -> Result<T> {
        Err(SolverError {
            message: message.into(),
            kind,
        })
    }

    pub fn invalid_constraint<T, M: Into<String>>(message: M) -> Result<T> {
        SolverError::new(ErrorKind::InvalidConstraint, message)
    }

    pub fn invalid_objective<T, M: Into<String>>(message: M) -> Result<T> {
        SolverError::new(ErrorKind::InvalidObjective, message)
    }

    pub fn infeasible<T, M: Into<String>>(message: M) -> Result<T> {
        SolverError::new(ErrorKind::Infeasible, message)
    }

    pub fn underspecified<T, M: Into<String>>(message: M) -> Result<T> {
        SolverError::new(ErrorKind::Underspecified, message)
    }

    pub fn unable_to_solve<T, M: Into<String>>(message: M) -> Result<T> {
        SolverError::new(ErrorKind::UnableToSolve, message)
    }

    pub fn invalid_solution<T, M: Into<String>>(message: M) -> Result<T> {
        SolverError::new(ErrorKind::InvalidSolution, message)
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ErrorKind {
    InvalidConstraint,
    InvalidObjective,
    Infeasible,
    Underspecified,
    UnableToSolve,
    InvalidSolution,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for SolverError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl error::Error for SolverError {
    fn description(&self) -> &str {
        &self.message
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}
