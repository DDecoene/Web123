// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use std::collections::{HashMap, HashSet};

use crate::functions::{evaluate, CellLookup};
use crate::model::{CellAddr, CellValue, ErrorKind, Expr};
use crate::parser::parse_input;

pub struct SpreadsheetCore {
    stored: HashMap<CellAddr, CellValue>,
    raw_inputs: HashMap<CellAddr, String>,
    computed: HashMap<CellAddr, CellValue>,
    dependents: HashMap<CellAddr, HashSet<CellAddr>>,
    named_ranges: HashMap<String, (CellAddr, CellAddr)>,
}

impl SpreadsheetCore {
    pub fn new() -> Self {
        SpreadsheetCore {
            stored: HashMap::new(),
            raw_inputs: HashMap::new(),
            computed: HashMap::new(),
            dependents: HashMap::new(),
            named_ranges: HashMap::new(),
        }
    }

    pub fn set_cell(&mut self, reference: &str, input: &str) {
        let addr = match CellAddr::parse(reference) {
            Some(a) => a,
            None => return,
        };
        self.remove_edges_for(addr);
        let value = parse_input(input);
        self.stored.insert(addr, value.clone());
        self.raw_inputs.insert(addr, input.to_string());
        self.add_edges_for(addr, &value);
        self.recalculate_from(addr);
    }

    pub fn display(&self, addr: CellAddr) -> String {
        match self.computed.get(&addr) {
            None | Some(CellValue::Empty) => String::new(),
            Some(CellValue::Number(n)) => format_number(*n),
            Some(CellValue::Text(s)) => s.clone(),
            Some(CellValue::Error(_)) => "ERR".to_string(),
            Some(CellValue::Formula(_)) => String::new(),
        }
    }

    pub fn raw_input(&self, addr: CellAddr) -> String {
        self.raw_inputs.get(&addr).cloned().unwrap_or_default()
    }

    pub fn is_error(&self, addr: CellAddr) -> bool {
        matches!(self.computed.get(&addr), Some(CellValue::Error(_)))
    }

    pub fn define_named_range(&mut self, name: String, from: CellAddr, to: CellAddr) {
        self.named_ranges.insert(name, (from, to));
    }

    pub fn recalculate_all(&mut self) {
        let dirty: HashSet<CellAddr> = self.stored.keys().copied().collect();
        self.recalculate_set(dirty);
    }

    fn remove_edges_for(&mut self, addr: CellAddr) {
        if let Some(CellValue::Formula(expr)) = self.stored.get(&addr).cloned() {
            for referenced in referenced_cells(&expr, &self.named_ranges) {
                if let Some(set) = self.dependents.get_mut(&referenced) {
                    set.remove(&addr);
                }
            }
        }
    }

    fn add_edges_for(&mut self, addr: CellAddr, value: &CellValue) {
        if let CellValue::Formula(expr) = value {
            for referenced in referenced_cells(expr, &self.named_ranges) {
                self.dependents.entry(referenced).or_default().insert(addr);
            }
        }
    }

    fn recalculate_from(&mut self, start: CellAddr) {
        let dirty = self.transitive_dependents(start);
        self.recalculate_set(dirty);
    }

    fn recalculate_set(&mut self, dirty: HashSet<CellAddr>) {
        let order = self.topological_order(&dirty);
        for addr in &order {
            let value = self.evaluate_cell(*addr);
            self.computed.insert(*addr, value);
        }
        for addr in &dirty {
            if !order.contains(addr) {
                self.computed.insert(*addr, CellValue::Error(ErrorKind::CircularRef));
            }
        }
    }

    fn transitive_dependents(&self, start: CellAddr) -> HashSet<CellAddr> {
        let mut visited = HashSet::new();
        let mut stack = vec![start];
        while let Some(addr) = stack.pop() {
            if visited.insert(addr) {
                if let Some(deps) = self.dependents.get(&addr) {
                    for &d in deps {
                        stack.push(d);
                    }
                }
            }
        }
        visited
    }

    fn topological_order(&self, dirty: &HashSet<CellAddr>) -> Vec<CellAddr> {
        let mut in_degree: HashMap<CellAddr, usize> = dirty.iter().map(|&a| (a, 0)).collect();
        let mut edges: HashMap<CellAddr, Vec<CellAddr>> = HashMap::new();
        for &addr in dirty {
            if let Some(CellValue::Formula(expr)) = self.stored.get(&addr) {
                for referenced in referenced_cells(expr, &self.named_ranges) {
                    if dirty.contains(&referenced) {
                        edges.entry(referenced).or_default().push(addr);
                        *in_degree.get_mut(&addr).unwrap() += 1;
                    }
                }
            }
        }
        let mut queue: Vec<CellAddr> = in_degree.iter().filter(|(_, &d)| d == 0).map(|(&a, _)| a).collect();
        let mut order = Vec::new();
        while let Some(addr) = queue.pop() {
            order.push(addr);
            if let Some(next) = edges.get(&addr) {
                for &n in next {
                    let d = in_degree.get_mut(&n).unwrap();
                    *d -= 1;
                    if *d == 0 {
                        queue.push(n);
                    }
                }
            }
        }
        order
    }

    fn evaluate_cell(&self, addr: CellAddr) -> CellValue {
        match self.stored.get(&addr) {
            None | Some(CellValue::Empty) => CellValue::Empty,
            Some(CellValue::Number(n)) => CellValue::Number(*n),
            Some(CellValue::Text(s)) => CellValue::Text(s.clone()),
            Some(CellValue::Formula(expr)) => evaluate(expr, self),
            Some(CellValue::Error(k)) => CellValue::Error(k.clone()),
        }
    }
}

impl CellLookup for SpreadsheetCore {
    fn value_of(&self, addr: CellAddr) -> CellValue {
        self.computed.get(&addr).cloned().unwrap_or(CellValue::Empty)
    }
    fn named_range(&self, name: &str) -> Option<(CellAddr, CellAddr)> {
        self.named_ranges.get(name).copied()
    }
}

fn format_number(n: f64) -> String {
    format!("{n}")
}

fn referenced_cells(
    expr: &Expr,
    named_ranges: &HashMap<String, (CellAddr, CellAddr)>,
) -> Vec<CellAddr> {
    let mut out = Vec::new();
    collect_refs(expr, named_ranges, &mut out);
    out
}

fn collect_refs(
    expr: &Expr,
    named_ranges: &HashMap<String, (CellAddr, CellAddr)>,
    out: &mut Vec<CellAddr>,
) {
    match expr {
        Expr::CellRef(addr) => out.push(*addr),
        Expr::Range(from, to) => out.extend(crate::model::collect_range_addrs(*from, *to)),
        Expr::NamedRange(name) => {
            if let Some((from, to)) = named_ranges.get(name) {
                out.extend(crate::model::collect_range_addrs(*from, *to));
            }
        }
        Expr::BinaryOp(_, l, r) => {
            collect_refs(l, named_ranges, out);
            collect_refs(r, named_ranges, out);
        }
        Expr::FunctionCall(_, args) => {
            for a in args {
                collect_refs(a, named_ranges, out);
            }
        }
        Expr::Number(_) | Expr::Text(_) | Expr::Invalid => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(reference: &str) -> CellAddr {
        CellAddr::parse(reference).unwrap()
    }

    #[test]
    fn sum_of_two_cells_matches_plan_1_behavior() {
        let mut core = SpreadsheetCore::new();
        core.set_cell("A1", "3");
        core.set_cell("A2", "4");
        core.set_cell("B1", "@SUM(A1,A2)");
        assert_eq!(core.display(a("B1")), "7");
    }

    #[test]
    fn editing_a_dependency_recalculates_the_dependent_automatically() {
        let mut core = SpreadsheetCore::new();
        core.set_cell("A1", "3");
        core.set_cell("A2", "4");
        core.set_cell("B1", "@SUM(A1,A2)");
        core.set_cell("A1", "10");
        assert_eq!(core.display(a("B1")), "14");
    }

    #[test]
    fn multi_level_dependency_chain_recalculates_in_order() {
        let mut core = SpreadsheetCore::new();
        core.set_cell("A1", "1");
        core.set_cell("B1", "=A1+1");
        core.set_cell("C1", "=B1+1");
        assert_eq!(core.display(a("C1")), "3");
        core.set_cell("A1", "10");
        assert_eq!(core.display(a("C1")), "12");
    }

    #[test]
    fn circular_reference_produces_err_on_every_cell_in_the_cycle() {
        let mut core = SpreadsheetCore::new();
        core.set_cell("A1", "=B1");
        core.set_cell("B1", "=A1");
        assert_eq!(core.display(a("A1")), "ERR");
        assert_eq!(core.display(a("B1")), "ERR");
        assert!(core.is_error(a("A1")));
        assert!(core.is_error(a("B1")));
    }

    #[test]
    fn division_by_zero_propagates_to_dependents() {
        let mut core = SpreadsheetCore::new();
        core.set_cell("A1", "=1/0");
        core.set_cell("B1", "=A1+1");
        assert_eq!(core.display(a("A1")), "ERR");
        assert_eq!(core.display(a("B1")), "ERR");
    }

    #[test]
    fn named_range_defined_via_define_named_range_is_usable_in_formulas() {
        let mut core = SpreadsheetCore::new();
        core.set_cell("A1", "3");
        core.set_cell("A2", "4");
        core.define_named_range("SALES".to_string(), a("A1"), a("A2"));
        core.set_cell("B1", "@SUM(SALES)");
        assert_eq!(core.display(a("B1")), "7");
    }

    #[test]
    fn re_setting_a_cell_overwrites_its_previous_value_and_edges() {
        let mut core = SpreadsheetCore::new();
        core.set_cell("A1", "1");
        core.set_cell("B1", "=A1");
        core.set_cell("B1", "9");
        core.set_cell("A1", "100");
        assert_eq!(core.display(a("B1")), "9");
    }
}
