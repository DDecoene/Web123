// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
enum CellValue {
    Empty,
    Number(f64),
    Formula { left: String, right: String },
}

pub struct SpreadsheetCore {
    cells: HashMap<String, CellValue>,
}

impl SpreadsheetCore {
    pub fn new() -> Self {
        SpreadsheetCore { cells: HashMap::new() }
    }

    pub fn set_cell(&mut self, reference: &str, input: &str) {
        self.cells.insert(reference.to_string(), parse_input(input));
    }

    pub fn get_display(&self, reference: &str) -> String {
        match self.cells.get(reference) {
            None | Some(CellValue::Empty) => String::new(),
            Some(CellValue::Number(n)) => format_number(*n),
            Some(CellValue::Formula { left, right }) => {
                format_number(self.number_of(left) + self.number_of(right))
            }
        }
    }

    fn number_of(&self, reference: &str) -> f64 {
        match self.cells.get(reference) {
            Some(CellValue::Number(n)) => *n,
            _ => 0.0,
        }
    }
}

fn format_number(n: f64) -> String {
    format!("{n}")
}

fn parse_input(input: &str) -> CellValue {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return CellValue::Empty;
    }
    if let Some(inner) = trimmed.strip_prefix("@SUM(").and_then(|s| s.strip_suffix(')')) {
        let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
        return match parts.as_slice() {
            [left, right] => CellValue::Formula { left: left.to_string(), right: right.to_string() },
            _ => CellValue::Empty,
        };
    }
    match trimmed.parse::<f64>() {
        Ok(n) => CellValue::Number(n),
        Err(_) => CellValue::Empty,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_cell_displays_as_empty_string() {
        let core = SpreadsheetCore::new();
        assert_eq!(core.get_display("A1"), "");
    }

    #[test]
    fn numeric_cell_displays_its_value() {
        let mut core = SpreadsheetCore::new();
        core.set_cell("A1", "42");
        assert_eq!(core.get_display("A1"), "42");
    }

    #[test]
    fn sum_formula_adds_two_referenced_cells() {
        let mut core = SpreadsheetCore::new();
        core.set_cell("A1", "3");
        core.set_cell("A2", "4");
        core.set_cell("B1", "@SUM(A1,A2)");
        assert_eq!(core.get_display("B1"), "7");
    }

    #[test]
    fn sum_formula_treats_missing_or_non_numeric_cell_as_zero() {
        let mut core = SpreadsheetCore::new();
        core.set_cell("A1", "5");
        core.set_cell("B1", "@SUM(A1,A2)");
        assert_eq!(core.get_display("B1"), "5");
    }

    #[test]
    fn re_setting_a_cell_overwrites_its_previous_value() {
        let mut core = SpreadsheetCore::new();
        core.set_cell("A1", "1");
        core.set_cell("A1", "9");
        assert_eq!(core.get_display("A1"), "9");
    }
}
