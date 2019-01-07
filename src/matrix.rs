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

pub struct Matrix {
    start_x: usize,
    start_y: usize,
    end_x: usize,
    end_y: usize,
    total_width: usize,
    _total_height: usize,
    coeffs: Vec<f64>,
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
    pub(crate) fn new(width: usize, height: usize, coeffs: Vec<f64>) -> Matrix {
        if width * height != coeffs.len() {
            panic!();
        }

        if width <= 2 || height <= 2 {
            panic!();
        }

        Matrix {
            start_x: 0,
            start_y: 0,
            end_x: width,
            end_y: height,
            total_width: width,
            _total_height: height,
            coeffs,
        }
    }

    /// Sets this matrix to be a sub-view into the overall matrix with the specified
    /// `start_x` and `start_y` coords
    pub fn sub(&mut self, start_x: usize, start_y: usize) {
        if self.start_x >= self.end_x || self.start_y >= self.end_y {
            panic!();
        }
        self.start_x = start_x;
        self.start_y = start_y;
    }

    pub fn last_row(&self) -> Row {
        Row::new(self.end_y - 1)
    }

    pub fn last_col(&self) -> Col {
        Col::new(self.end_x - 1)
    }

    pub fn first_col(&self) -> Col {
        Col::new(self.start_x)
    }

    pub fn first_row(&self) -> Row {
        Row::new(self.start_y)
    }

    pub fn cols_range(&self, start: Col, end: Col) -> impl DoubleEndedIterator<Item=Col> {
        ops::Range { start: start.value, end: end.value }.map(|val| Col::new(val))
    }

    pub fn cols_from(&self, start: Col) -> impl DoubleEndedIterator<Item=Col> {
        ops::Range { start: start.value, end: self.end_x }.map(|val| Col::new(val))
    }

    pub fn rows_range(&self, start: Col, end: Col) -> impl DoubleEndedIterator<Item=Row> {
        ops::Range { start: start.value, end: end.value }.map(|val| Row::new(val))
    }

    pub fn rows_from(&self, start: Row) -> impl DoubleEndedIterator<Item=Row> {
        ops::Range { start: start.value, end: self.end_y }.map(|val| Row::new(val))
    }

    pub fn cols(&self) -> impl DoubleEndedIterator<Item=Col> {
        ops::Range { start: self.start_x, end: self.end_x }.map(|val| Col::new(val))
    }

    pub fn rows(&self) -> impl DoubleEndedIterator<Item=Row> {
         ops::Range { start: self.start_y, end: self.end_y }.map(|val| Row::new(val))
    }

    pub fn width(&self) -> usize { self.end_x - self.start_x }

    pub fn height(&self) -> usize { self.end_y - self.start_y }

    pub fn value(&self, row: Row, col: Col) -> f64 {
        self.coeffs[col.value + row.value * self.total_width]
    }

    /// Only for use in matrix initialization
    pub(crate) fn set_value_raw(&mut self, row: usize, col: usize, value: f64) {
        self.coeffs[col + row * self.total_width] = value;
    }

    /// Sets the specified row, col entry of the matrix to the value.  By careful using
    /// this to solve systems, it is better to use one of the basic row matrix operations
    /// below
    pub fn set_value(&mut self, row: Row, col: Col, value: f64) {
        self.coeffs[col.value + row.value * self.total_width] = value;
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
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Row {
    value: usize,
}

impl Row {
    pub(crate) fn new(value: usize) -> Row {
        Row { value }
    }

    pub fn is_valid(&self, matrix: &Matrix) -> bool {
        self.value < matrix.end_y
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
    pub(crate) fn new(value: usize) -> Col {
        Col { value }
    }

    pub fn is_valid(&self, matrix: &Matrix) -> bool {
        self.value < matrix.end_x
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
