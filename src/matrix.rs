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

use std::{f64, fmt, ops};

use log::{warn};

use crate::{Solution, ObjectiveKind, Result};
use crate::{simplex::simplex, gaussian_elimination::gaussian_elimination};

enum Mode {
    GaussianElimination,
    Simplex,
}

pub struct Matrix {
    width: usize,
    height: usize,
    variables: usize,
    objective_kind: ObjectiveKind,
    coeffs: Vec<f64>,
    mode: Mode,
}

impl fmt::Debug for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in self.rows() {
            writeln!(f)?;
            for col in self.cols() {
                write!(f, "{:.6}, ", self.value(row, col))?;
            }
        }
        writeln!(f)
    }
}

impl Matrix {
    pub(crate) fn new_gaussian_elimination(width: usize, height: usize, variables: usize,
                                           coeffs: Vec<f64>) -> Matrix {
        Matrix {
            width,
            height,
            variables,
            coeffs,
            mode: Mode::GaussianElimination,
            objective_kind: ObjectiveKind::Minimize, // unused for this problem
        }
    }

    pub(crate) fn new_simplex(width: usize, height: usize, variables: usize,
                              objective_kind: ObjectiveKind, coeffs: Vec<f64>) -> Matrix {
        Matrix {
            width,
            height,
            variables,
            objective_kind,
            coeffs,
            mode: Mode::Simplex,
        }
    }

    pub fn last_row(&self) -> Row {
        Row::new(self.height - 1)
    }

    pub fn last_col(&self) -> Col {
        Col::new(self.width - 1)
    }

    pub fn first_col(&self) -> Col {
        Col::new(0)
    }

    pub fn first_row(&self) -> Row {
        Row::new(0)
    }

    pub fn cols_range(&self, start: Col, end: Col) -> impl DoubleEndedIterator<Item=Col> {
        ops::Range { start: start.value, end: end.value }.map(|val| Col::new(val))
    }

    pub fn cols_from(&self, start: Col) -> impl DoubleEndedIterator<Item=Col> {
        ops::Range { start: start.value, end: self.width }.map(|val| Col::new(val))
    }

    pub fn rows_range(&self, start: Col, end: Col) -> impl DoubleEndedIterator<Item=Row> {
        ops::Range { start: start.value, end: end.value }.map(|val| Row::new(val))
    }

    pub fn rows_from(&self, start: Row) -> impl DoubleEndedIterator<Item=Row> {
        ops::Range { start: start.value, end: self.height }.map(|val| Row::new(val))
    }

    pub fn cols(&self) -> impl DoubleEndedIterator<Item=Col> {
        ops::Range { start: 0, end: self.width }.map(|val| Col::new(val))
    }

    pub fn rows(&self) -> impl DoubleEndedIterator<Item=Row> {
         ops::Range { start: 0, end: self.height }.map(|val| Row::new(val))
    }

    pub fn width(&self) -> usize { self.width }

    pub fn height(&self) -> usize { self.height }

    pub fn num_variables(&self) -> usize { self.variables }

    pub fn objective_kind(&self) -> ObjectiveKind { self.objective_kind }

    pub fn value(&self, row: Row, col: Col) -> f64 {
        self.coeffs[col.value + row.value * self.width]
    }

    /// Only for use in matrix initialization
    pub(crate) fn set_value_raw(&mut self, row: usize, col: usize, value: f64) {
        self.coeffs[col + row * self.width] = value;
    }

    /// Sets the specified row, col entry of the matrix to the value.  By careful using
    /// this to solve systems, it is better to use one of the basic row matrix operations
    /// below
    pub fn set_value(&mut self, row: Row, col: Col, value: f64) {
        self.coeffs[col.value + row.value * self.width] = value;
    }

    fn swap_values(&mut self, row1: Row, col1: Col, row2: Row, col2: Col) {
        let temp = self.value(row1, col1);
        self.set_value(row1, col1, self.value(row2, col2));
        self.set_value(row2, col2, temp);
    }

    /// One of the 3 basic matrix row operations - swaps row1 and row2
    pub fn swap_rows(&mut self, row1: Row, row2: Row) {
        for col in self.cols() {
            self.swap_values(row1, col, row2, col);
        }
    }

    /// One of the 3 basic matrix row operations - multiplies a row by a constant
    pub fn multiply_row(&mut self, row: Row, mult: f64) {
        if mult == 0.0 {
            warn!("Multiplying row by 0 which is an invalid operation");
        }

        for col in self.cols() {
            let value = self.value(row, col);
            self.set_value(row, col, value * mult);
        }
    }

    /// One of the 3 basic matrix row operations.  Adds `src` multiplied
    /// by `mult` to `dest`
    pub fn add_row(&mut self, src: Row, dest: Row, mult: f64) {
        if mult == 0.0 {
            warn!("Multiplying row by 0 which is an invalid operation");
        }

        for col in self.cols() {
            let value = self.value(src, col) * mult + self.value(dest, col);
            self.set_value(dest, col, value);
        }
    }

    pub fn has_zero_row(&self) -> bool {
        for row in self.rows() {
            let mut zero_row = true;
            for col in self.cols() {
                if self.value(row, col) != 0.0 {
                    zero_row = false;
                }
            }

            if zero_row {
                return true;
            }
        }

        false
    }

    pub fn solve(&mut self) -> Result<Solution> {
        match self.mode {
            Mode::Simplex => simplex(self),
            Mode::GaussianElimination => gaussian_elimination(self),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Row {
    value: usize,
}

impl Row {
    fn new(value: usize) -> Row {
        Row { value }
    }

    pub fn is_valid(&self, matrix: &Matrix) -> bool {
        self.value < matrix.height
    }

    pub fn index(&self) -> usize {
        self.value
    }
}

impl From<Col> for Row {
    fn from(col: Col) -> Row {
        Row { value: col.value }
    }
}

impl ops::Add<Row> for Row {
    type Output = Row;

    fn add(self, rhs: Row) -> Row {
        Row { value: self.value + rhs.value }
    }
}

impl ops::Sub<Row> for Row {
    type Output = Row;

    fn sub(self, rhs: Row) -> Row {
        Row { value: self.value - rhs.value }
    }
}

impl ops::Sub<usize> for Row {
    type Output = Row;

    fn sub(self, rhs: usize) -> Row {
        Row { value: self.value - rhs }
    }
}

impl ops::Add<usize> for Row {
    type Output = Row;

    fn add(self, rhs: usize) -> Row {
        Row { value: self.value + rhs }
    }
}

impl ops::AddAssign<usize> for Row {
    fn add_assign(&mut self, rhs: usize) {
        self.value += rhs;
    }
}

impl ops::SubAssign<usize> for Row {
    fn sub_assign(&mut self, rhs: usize) {
        self.value -= rhs;
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Col {
    value: usize,
}

impl Col {
    fn new(value: usize) -> Col {
        Col { value }
    }

    pub fn is_valid(&self, matrix: &Matrix) -> bool {
        self.value < matrix.width
    }

    pub fn index(&self) -> usize {
        self.value
    }
}

impl From<Row> for Col {
    fn from(row: Row) -> Col {
        Col { value: row.value }
    }
}

impl ops::Add<Col> for Col {
    type Output = Col;

    fn add(self, rhs: Col) -> Col {
        Col { value: self.value + rhs.value }
    }
}

impl ops::Add<usize> for Col {
    type Output = Col;

    fn add(self, rhs: usize) -> Col {
        Col { value: self.value + rhs }
    }
}

impl ops::Sub<Col> for Col {
    type Output = Col;

    fn sub(self, rhs: Col) -> Col {
        Col { value: self.value - rhs.value }
    }
}

impl ops::Sub<usize> for Col {
    type Output = Col;

    fn sub(self, rhs: usize) -> Col {
        Col { value: self.value - rhs }
    }
}

impl ops::AddAssign<usize> for Col {
    fn add_assign(&mut self, rhs: usize) {
        self.value += rhs;
    }
}

impl ops::SubAssign<usize> for Col {
    fn sub_assign(&mut self, rhs: usize) {
        self.value -= rhs;
    }
}
