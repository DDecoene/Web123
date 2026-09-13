// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use crate::model::{CellAddr, CellValue, Expr, FnName, Op};

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, c: char) -> Result<(), String> {
        self.skip_ws();
        if self.peek() == Some(c) {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!("expected '{}' at position {}", c, self.pos))
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_term()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some('+') => {
                    self.pos += 1;
                    let right = self.parse_term()?;
                    left = Expr::BinaryOp(Op::Add, Box::new(left), Box::new(right));
                }
                Some('-') => {
                    self.pos += 1;
                    let right = self.parse_term()?;
                    left = Expr::BinaryOp(Op::Sub, Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_factor()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some('*') => {
                    self.pos += 1;
                    let right = self.parse_factor()?;
                    left = Expr::BinaryOp(Op::Mul, Box::new(left), Box::new(right));
                }
                Some('/') => {
                    self.pos += 1;
                    let right = self.parse_factor()?;
                    left = Expr::BinaryOp(Op::Div, Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        match self.peek() {
            Some('(') => {
                self.pos += 1;
                let inner = self.parse_expr()?;
                self.expect(')')?;
                Ok(inner)
            }
            Some('@') => self.parse_function_call(),
            Some('"') => self.parse_string(),
            Some(c) if c.is_ascii_digit() || c == '-' => self.parse_number(),
            Some(c) if c.is_ascii_alphabetic() => self.parse_cell_ref_or_range(),
            _ => Err(format!("unexpected character at position {}", self.pos)),
        }
    }

    fn parse_number(&mut self) -> Result<Expr, String> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.pos += 1;
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit() || c == '.') {
            self.pos += 1;
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        s.parse::<f64>()
            .map(Expr::Number)
            .map_err(|_| format!("bad number at position {}", start))
    }

    fn parse_string(&mut self) -> Result<Expr, String> {
        self.pos += 1;
        let start = self.pos;
        while matches!(self.peek(), Some(c) if c != '"') {
            self.pos += 1;
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        self.expect('"')?;
        Ok(Expr::Text(s))
    }

    fn parse_ident(&mut self) -> String {
        let start = self.pos;
        while matches!(self.peek(), Some(c) if c.is_ascii_alphanumeric()) {
            self.pos += 1;
        }
        self.chars[start..self.pos].iter().collect()
    }

    fn parse_cell_ref_or_range(&mut self) -> Result<Expr, String> {
        let ident = self.parse_ident();
        if let Some(addr) = CellAddr::parse(&ident) {
            self.skip_ws();
            if self.peek() == Some(':') {
                self.pos += 1;
                self.skip_ws();
                let ident2 = self.parse_ident();
                let addr2 = CellAddr::parse(&ident2)
                    .ok_or_else(|| format!("bad cell reference '{}'", ident2))?;
                return Ok(Expr::Range(addr, addr2));
            }
            return Ok(Expr::CellRef(addr));
        }
        Ok(Expr::NamedRange(ident))
    }

    fn parse_function_call(&mut self) -> Result<Expr, String> {
        self.pos += 1;
        let name = self.parse_ident();
        let fn_name = match name.to_uppercase().as_str() {
            "SUM" => FnName::Sum,
            "AVG" => FnName::Avg,
            "IF" => FnName::If,
            "VLOOKUP" => FnName::VLookup,
            other => return Err(format!("unknown function '{}'", other)),
        };
        self.expect('(')?;
        let mut args = Vec::new();
        self.skip_ws();
        if self.peek() != Some(')') {
            if fn_name == FnName::If {
                args.push(self.parse_condition()?);
            } else {
                args.push(self.parse_expr()?);
            }
            loop {
                self.skip_ws();
                if self.peek() == Some(',') {
                    self.pos += 1;
                    args.push(self.parse_expr()?);
                } else {
                    break;
                }
            }
        }
        self.expect(')')?;
        Ok(Expr::FunctionCall(fn_name, args))
    }

    fn parse_condition(&mut self) -> Result<Expr, String> {
        let left = self.parse_expr()?;
        self.skip_ws();
        let op = match self.peek() {
            Some('>') => Op::Gt,
            Some('<') => Op::Lt,
            Some('=') => Op::Eq,
            _ => return Err(format!("expected comparison operator at position {}", self.pos)),
        };
        self.pos += 1;
        let right = self.parse_expr()?;
        Ok(Expr::BinaryOp(op, Box::new(left), Box::new(right)))
    }
}

fn parse_expr_str(src: &str) -> Result<Expr, String> {
    let mut p = Parser { chars: src.chars().collect(), pos: 0 };
    let expr = p.parse_expr()?;
    p.skip_ws();
    if p.pos != p.chars.len() {
        return Err(format!("unexpected trailing input at position {}", p.pos));
    }
    Ok(expr)
}

pub fn parse_input(input: &str) -> CellValue {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return CellValue::Empty;
    }
    if trimmed.starts_with('=') || trimmed.starts_with('@') {
        let src = trimmed.strip_prefix('=').unwrap_or(trimmed);
        return match parse_expr_str(src) {
            Ok(expr) => CellValue::Formula(expr),
            Err(_) => CellValue::Formula(Expr::Invalid),
        };
    }
    match trimmed.parse::<f64>() {
        Ok(n) => CellValue::Number(n),
        Err(_) => CellValue::Text(trimmed.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::CellAddr;

    fn a(reference: &str) -> CellAddr {
        CellAddr::parse(reference).unwrap()
    }

    #[test]
    fn plain_number_and_text() {
        assert_eq!(parse_input("42"), CellValue::Number(42.0));
        assert_eq!(parse_input("hello"), CellValue::Text("hello".to_string()));
        assert_eq!(parse_input(""), CellValue::Empty);
        assert_eq!(parse_input("   "), CellValue::Empty);
    }

    #[test]
    fn arithmetic_with_precedence_and_parens() {
        assert_eq!(
            parse_input("=A1+B1*2"),
            CellValue::Formula(Expr::BinaryOp(
                Op::Add,
                Box::new(Expr::CellRef(a("A1"))),
                Box::new(Expr::BinaryOp(
                    Op::Mul,
                    Box::new(Expr::CellRef(a("B1"))),
                    Box::new(Expr::Number(2.0)),
                )),
            ))
        );
        assert_eq!(
            parse_input("=(A1+A2)/2"),
            CellValue::Formula(Expr::BinaryOp(
                Op::Div,
                Box::new(Expr::BinaryOp(
                    Op::Add,
                    Box::new(Expr::CellRef(a("A1"))),
                    Box::new(Expr::CellRef(a("A2"))),
                )),
                Box::new(Expr::Number(2.0)),
            ))
        );
    }

    #[test]
    fn sum_over_a_range() {
        assert_eq!(
            parse_input("@SUM(A1:A10)"),
            CellValue::Formula(Expr::FunctionCall(
                FnName::Sum,
                vec![Expr::Range(a("A1"), a("A10"))],
            ))
        );
    }

    #[test]
    fn sum_over_two_plain_cells_still_works_like_plan_1() {
        assert_eq!(
            parse_input("@SUM(A1,A2)"),
            CellValue::Formula(Expr::FunctionCall(
                FnName::Sum,
                vec![Expr::CellRef(a("A1")), Expr::CellRef(a("A2"))],
            ))
        );
    }

    #[test]
    fn if_with_comparison_condition_and_string_branches() {
        assert_eq!(
            parse_input("@IF(A1>10,\"big\",\"small\")"),
            CellValue::Formula(Expr::FunctionCall(
                FnName::If,
                vec![
                    Expr::BinaryOp(
                        Op::Gt,
                        Box::new(Expr::CellRef(a("A1"))),
                        Box::new(Expr::Number(10.0)),
                    ),
                    Expr::Text("big".to_string()),
                    Expr::Text("small".to_string()),
                ],
            ))
        );
    }

    #[test]
    fn named_range_identifier_that_is_not_a_cell_reference() {
        assert_eq!(
            parse_input("@SUM(SALES)"),
            CellValue::Formula(Expr::FunctionCall(
                FnName::Sum,
                vec![Expr::NamedRange("SALES".to_string())],
            ))
        );
    }

    #[test]
    fn unknown_function_and_trailing_garbage_become_invalid() {
        assert_eq!(parse_input("@BOGUS(A1)"), CellValue::Formula(Expr::Invalid));
        assert_eq!(parse_input("=A1+"), CellValue::Formula(Expr::Invalid));
        assert_eq!(parse_input("=A1 A2"), CellValue::Formula(Expr::Invalid));
    }
}
