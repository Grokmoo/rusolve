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

use std::fmt;
use std::io::{Error};
use std::collections::HashMap;

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
}

impl fmt::Debug for Solution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (index, value) in self.coeffs.iter().enumerate() {
            write!(f, "x[{}] = {:.6},", index, value)?;
        }
        writeln!(f)
    }
}

#[derive(Debug)]
pub struct Problem {
    max_variable: u32,
    constraints: Vec<Constraint>,
}

impl Problem {
    pub fn new() -> Problem {
        Problem {
            max_variable: 0,
            constraints: Vec::new(),
        }
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

    pub fn solve(&mut self) -> Result<Solution, Error> {
        let mut matrix = Matrix::new(self);
        Ok(matrix.solve())
    }
}

impl Constraint {
    pub fn new() -> Constraint {
        Constraint {
            coeffs: HashMap::new(),
            constant: 0.0,
        }
    }

    pub fn set_value(&mut self, constant: f64) {
        self.constant = constant;
    }

    pub fn add_term(&mut self, coeff: f64, variable: Variable) {
        self.coeffs.insert(variable.index, coeff);
    }
}

struct Matrix {
    width: usize,
    height: usize,
    coeffs: Vec<f64>,
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
    fn new(problem: &Problem) -> Matrix {
        let width = problem.max_variable as usize + 1;
        let height = width - 1;
        let mut coeffs = vec![0.0;width*height];

        let mut row = 0;
        for constraint in problem.constraints.iter() {
            for (index, value) in constraint.coeffs.iter() {
                coeffs[*index as usize + row * width] = *value;
            }
            coeffs[width - 1 + row * width] = constraint.constant;
            row += 1;
        }

        Matrix {
            width,
            height,
            coeffs,
        }
    }

    fn value(&self, row: usize, col: usize) -> f64 {
        self.coeffs[col + row * self.width]
    }

    fn set_value(&mut self, row: usize, col: usize, value: f64) {
        self.coeffs[col + row * self.width] = value;
    }

    fn swap_values(&mut self, row1: usize, col1: usize, row2: usize, col2: usize) {
        let temp = self.value(row1, col1);
        self.set_value(row1, col1, self.value(row2, col2));
        self.set_value(row2, col2, temp);
    }

    fn find_pivot_max(&self, cur_pivot_row: usize, pivot_col: usize) -> usize {
        let mut max_value = 0.0;
        let mut pivot_max = cur_pivot_row;
        for row in cur_pivot_row .. self.height {
            let cur_value = self.value(row, pivot_col).abs();
            if cur_value > max_value {
                max_value = cur_value;
                pivot_max = row;
            }
        }
        pivot_max
    }

    fn swap_rows(&mut self, row1: usize, row2: usize) {
        for col in 0..self.width {
            self.swap_values(row1, col, row2, col);
        }
    }

    fn solve(&mut self) -> Solution {
        let mut pivot_row = 0;
        let mut pivot_col = 0;

        println!("Reducing matrix to triangular form");
        println!("{:?}", self);
        while pivot_row < self.height && pivot_col < self.width {
            let pivot_max = self.find_pivot_max(pivot_row, pivot_col);

            if self.value(pivot_max, pivot_col) == 0.0 {
                pivot_col += 1;
            } else {
                self.swap_rows(pivot_row, pivot_max);

                for row in (pivot_row + 1) .. self.height {
                    let coeff = self.value(row, pivot_col) / self.value(pivot_row, pivot_col);
                    self.set_value(row, pivot_col, 0.0);

                    for col in (pivot_col + 1) .. self.width {
                        let value = self.value(row, col) - self.value(pivot_row, col) * coeff;
                        self.set_value(row, col, value);
                    }
                }

                pivot_row += 1;
                pivot_col += 1;
            }

            println!("Step");
            println!("{:?}", self);
        }

        println!("Back substituting");
        let mut coeffs = vec![0.0; self.height];
        for row in (0..self.height).rev() {
            let mut coeff = self.value(row, self.width - 1);
            for col in (row + 1) .. self.height {
                coeff -= self.value(row, col) * coeffs[col as usize];
            }

            coeffs[row as usize] = coeff / self.value(row, row);
        }

        Solution {
            coeffs
        }
    }
}
