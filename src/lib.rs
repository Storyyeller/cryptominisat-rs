// Copyright (c) 2016 Robert Grosse

// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

extern crate libc;
use libc::size_t;
use std::slice;

pub const MAX_NUM_VARS: size_t = (1 << 28) - 1;

// cryptominisat types
enum SATSolver {} // opaque pointer

#[repr(C)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Lit(u32);
impl Lit {
    pub fn new(var: u32, negated: bool) -> Option<Lit> {
        if var < (1 << 31) {
            Some(Lit(var << 1 | (negated as u32)))
        } else { None }
    }
    pub fn var(&self) -> u32 { self.0 >> 1 }
    pub fn isneg(&self) -> bool { self.0 & 1 != 0 }
}
impl std::ops::Not for Lit {
    type Output = Lit; fn not(self) -> Lit { Lit(self.0 ^ 1) }
}

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Lbool {
    True = 0,
    False = 1,
    Undef = 2,
}
impl Lbool {
    pub fn from(b: bool) -> Lbool { if b { Lbool::True } else { Lbool::False } }
}

#[repr(C)]
struct slice_from_c<T>(*const T, size_t);
unsafe fn to_slice<'a, T>(raw: slice_from_c<T>) -> &'a [T] { slice::from_raw_parts(raw.0, raw.1) }

#[link(name = "cryptominisat5")]
extern {
    fn cmsat_new() -> *mut SATSolver;
    fn cmsat_free(this: *mut SATSolver);
    fn cmsat_nvars(this: *const SATSolver) -> u32;
    fn cmsat_add_clause(this: *mut SATSolver, lits: *const Lit, num_lits: size_t) -> bool;
    fn cmsat_add_xor_clause(this: *mut SATSolver, vars: *const u32, num_vars: size_t, rhs: bool) -> bool;
    fn cmsat_new_vars(this: *mut SATSolver, n: size_t);
    fn cmsat_solve(this: *mut SATSolver) -> Lbool;
    fn cmsat_solve_with_assumptions(this: *mut SATSolver, assumptions: *const Lit, num_assumptions: size_t) -> Lbool;
    fn cmsat_get_model(this: *const SATSolver) -> slice_from_c<Lbool>;
    fn cmsat_get_conflict(this: *const SATSolver) -> slice_from_c<Lit>;
    fn cmsat_set_num_threads(this: *mut SATSolver, n: u32);
}

pub struct Solver(*mut SATSolver);
impl Drop for Solver {
    fn drop(&mut self) { unsafe{cmsat_free(self.0)}; }
}
impl Solver {
    // wrappers
    pub fn new() -> Solver { Solver(unsafe{cmsat_new()}) }
    pub fn nvars(&self) -> u32 { unsafe{cmsat_nvars(self.0)} }
    pub fn add_clause(&mut self, lits: &[Lit]) -> bool { unsafe{cmsat_add_clause(self.0, lits.as_ptr(), lits.len())} }
    pub fn add_xor_clause(&mut self, vars: &[u32], rhs: bool) -> bool { unsafe{cmsat_add_xor_clause(self.0, vars.as_ptr(), vars.len(), rhs)} }
    pub fn new_vars(&mut self, n: size_t) { unsafe{cmsat_new_vars(self.0, n)} }
    pub fn solve(&mut self) -> Lbool { unsafe{cmsat_solve(self.0)} }
    pub fn solve_with_assumptions(&mut self, assumptions: &[Lit]) -> Lbool { unsafe{cmsat_solve_with_assumptions(self.0, assumptions.as_ptr(), assumptions.len())} }
    pub fn get_model(&self) -> &[Lbool] { unsafe{to_slice(cmsat_get_model(self.0))} }
    pub fn get_conflict(&self) -> &[Lit] { unsafe{to_slice(cmsat_get_conflict(self.0))} }
    pub fn set_num_threads(&mut self, n: u32) { unsafe{cmsat_set_num_threads(self.0, n)} }

    // helper functions defined in terms of the above
    pub fn new_var(&mut self) -> Lit {
        let n = self.nvars();
        self.new_vars(1);
        Lit::new(n as u32, false).unwrap()
    }
    pub fn is_true(&self, lit: Lit) -> bool {
        self.get_model()[lit.var() as usize] == Lbool::from(!lit.isneg())
    }
    pub fn add_xor_literal_clause(&mut self, lits: &[Lit], mut rhs: bool) -> bool {
        let mut vars = Vec::with_capacity(lits.len());
        for lit in lits {
            vars.push(lit.var()); rhs ^= lit.isneg();
        }
        self.add_xor_clause(&vars, rhs)
    }
}
