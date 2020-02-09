//  This file is part of rusolve, an optimizer / solver written in Rust.
//  Copyright 2020 Jared Stephen
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

use log::{debug, info, trace};

use crate::{Result, SolverError, Problem, Solution, ConstraintKind, ObjectiveKind,
    VariableKind};

struct Var {
    value: i32,
    bound: Bound,
}

impl std::fmt::Debug for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

struct Bound {
    min: i32,
    max: i32,
}

struct Constraint {
    coeffs: Vec<f64>,
    kind: ConstraintKind,
    constant: f64,
}

struct Objective {
    coeffs: Vec<f64>,
    kind: ObjectiveKind,
}

pub fn solve(problem: &Problem) -> Result<Solution> {
    let (obj_expr, obj_kind) = match problem.objective() {
        None => return SolverError::invalid_objective("Must set an objective for Integer problems."),
        Some((expr, kind)) => (expr, kind),
    };

    let bounds: Vec<Bound> = problem.variables().map(|var| {
        match var.kind() {
            VariableKind::Continuous => Bound { min: 0, max: 0 },
            VariableKind::Integer(min, max) => Bound { min, max },
        }
    }).collect();

    let num_vars = problem.num_variables() as u32;
    let constraints: Vec<Constraint> = problem.constraints().iter().map(|constraint| {
        let mut coeffs = Vec::new();
        for i in 0..num_vars {
            coeffs.push(constraint.expr().get(i));
        }
        Constraint {
            coeffs,
            kind: constraint.kind(),
            constant: constraint.constant(),
        }
    }).collect();


    let mut obj_coeffs = Vec::new();
    for i in 0..num_vars {
        obj_coeffs.push(obj_expr.get(i));
    }

    let objective = Objective {
        coeffs: obj_coeffs,
        kind: obj_kind
    };

    let mut solution = vec![0; num_vars as usize];
    let mut best_obj = match objective.kind {
        ObjectiveKind::Maximize => std::f64::MIN,
        ObjectiveKind::Minimize => std::f64::MAX,
    };
    let initial_best = best_obj;

    let mut vars = Vec::new();
    for bound in bounds {
        let value = bound.min;
        let var = Var { value, bound };
        vars.push(var);
    }

    info!("Setup problem in vector form.  Brute force searching...");
    check_combinations(&mut solution, &mut best_obj, &mut vars, &constraints, &objective, 0);

    if initial_best == best_obj {
        return SolverError::infeasible("No Solution exists.");
    }

    let coeffs = solution.iter().map(|c| *c as f64).collect();

    Ok(Solution::new(coeffs, Some(best_obj)))
}

fn check_combinations(
    solution: &mut Vec<i32>,
    best_value: &mut f64,
    vars: &mut Vec<Var>,
    constraints: &[Constraint],
    objective: &Objective,
    cur_index: usize,
) {
    for val in vars[cur_index].bound.min..=vars[cur_index].bound.max {
        vars[cur_index].value = val;
        debug!("Checking with coefficients: {:?}", vars);

        if meets_constraints(vars, constraints) {
            let test = get_constraint_value(vars, &objective.coeffs);
            debug!("  Constraints met, got objective value: {}", test);

            let best = match objective.kind {
                ObjectiveKind::Minimize => test < *best_value,
                ObjectiveKind::Maximize => test > *best_value,
            };

            if best {
                debug!("  Values tested are new best.");
                *best_value = test;
                for i in 0..vars.len() {
                    solution[i] = vars[i].value;
                }
            }
        }

        if cur_index < vars.len() - 1 {
            check_combinations(solution, best_value, vars, constraints, objective, cur_index + 1);
        }
    }
}

fn meets_constraints(vars: &[Var], constraints: &[Constraint]) -> bool {
    let tol = std::f32::EPSILON as f64;
    for constraint in constraints {
        let constant = constraint.constant;
        let computed = get_constraint_value(vars, &constraint.coeffs);
        trace!("  Computed constraint value of {}, test {:?} against {}",
            computed, constraint.kind, constant);
        let met = match constraint.kind {
            ConstraintKind::GreaterThanOrEqualTo => computed > constant - tol,
            ConstraintKind::EqualTo => (computed - constant).abs() < tol,
            ConstraintKind::LessThanOrEqualTo => computed < constant + tol,
        };

        if !met { return false; }
    }

    true
}

fn get_constraint_value(vars: &[Var], constraint: &[f64]) -> f64 {
    let mut total = 0.0;
    for i in 0..vars.len() {
        total += vars[i].value as f64 * constraint[i];
    }

    total
}
