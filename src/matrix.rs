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
        for row in 0..self.height {
            writeln!(f)?;
            for col in 0..self.width {
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

    pub fn width(&self) -> usize { self.width }

    pub fn height(&self) -> usize { self.height }

    pub fn num_variables(&self) -> usize { self.variables }

    pub fn objective_kind(&self) -> ObjectiveKind { self.objective_kind }

    pub fn value(&self, row: usize, col: usize) -> f64 {
        self.coeffs[col + row * self.width]
    }

    /// Sets the specified row, col entry of the matrix to the value.  By careful using
    /// this to solve systems, it is better to use one of the basic row matrix operations
    /// below
    pub fn set_value(&mut self, row: usize, col: usize, value: f64) {
        self.coeffs[col + row * self.width] = value;
    }

    fn swap_values(&mut self, row1: usize, col1: usize, row2: usize, col2: usize) {
        let temp = self.value(row1, col1);
        self.set_value(row1, col1, self.value(row2, col2));
        self.set_value(row2, col2, temp);
    }

    /// One of the 3 basic matrix row operations - swaps row1 and row2
    pub fn swap_rows(&mut self, row1: usize, row2: usize) {
        for col in 0..self.width {
            self.swap_values(row1, col, row2, col);
        }
    }

    /// One of the 3 basic matrix row operations - multiplies a row by a constant
    pub fn multiply_row(&mut self, row: usize, mult: f64) {
        if mult == 0.0 {
            warn!("Multiplying row by 0 which is an invalid operation");
        }

        for col in 0..self.width {
            let value = self.value(row, col);
            self.set_value(row, col, value * mult);
        }
    }

    /// One of the 3 basic matrix row operations.  Adds `src` multiplied
    /// by `mult` to `dest`
    pub fn add_row(&mut self, src: usize, dest: usize, mult: f64) {
        if mult == 0.0 {
            warn!("Multiplying row by 0 which is an invalid operation");
        }

        for col in 0..self.width {
            let value = self.value(src, col) * mult + self.value(dest, col);
            self.set_value(dest, col, value);
        }
    }

    pub fn has_zero_row(&self) -> bool {
        for row in 0..self.height() {
            let mut zero_row = true;
            for col in 0..self.width() {
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
