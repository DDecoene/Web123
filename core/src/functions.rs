// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use crate::model::{collect_range_addrs, CellAddr, CellValue, ErrorKind, Expr, FnName, Op};

pub trait CellLookup {
    fn value_of(&self, addr: CellAddr) -> CellValue;
    fn named_range(&self, name: &str) -> Option<(CellAddr, CellAddr)>;
}

pub fn evaluate(expr: &Expr, lookup: &dyn CellLookup) -> CellValue {
    match expr {
        Expr::Invalid => CellValue::Error(ErrorKind::BadRef),
        Expr::Number(n) => CellValue::Number(*n),
        Expr::Text(s) => CellValue::Text(s.clone()),
        Expr::CellRef(addr) => lookup.value_of(*addr),
        Expr::Range(_, _) => CellValue::Error(ErrorKind::BadRef),
        Expr::NamedRange(_) => CellValue::Error(ErrorKind::BadRef),
        Expr::BinaryOp(op, l, r) => eval_binary(*op, evaluate(l, lookup), evaluate(r, lookup)),
        Expr::FunctionCall(name, args) => eval_function(*name, args, lookup),
    }
}

fn as_number(v: &CellValue) -> f64 {
    match v {
        CellValue::Number(n) => *n,
        _ => 0.0,
    }
}

fn values_equal(l: &CellValue, r: &CellValue) -> bool {
    match (l, r) {
        (CellValue::Text(a), CellValue::Text(b)) => a == b,
        _ => as_number(l) == as_number(r),
    }
}

fn eval_binary(op: Op, l: CellValue, r: CellValue) -> CellValue {
    if let CellValue::Error(_) = &l {
        return CellValue::Error(ErrorKind::PropagatedError);
    }
    if let CellValue::Error(_) = &r {
        return CellValue::Error(ErrorKind::PropagatedError);
    }
    match op {
        Op::Add => CellValue::Number(as_number(&l) + as_number(&r)),
        Op::Sub => CellValue::Number(as_number(&l) - as_number(&r)),
        Op::Mul => CellValue::Number(as_number(&l) * as_number(&r)),
        Op::Div => {
            let rn = as_number(&r);
            if rn == 0.0 {
                CellValue::Error(ErrorKind::DivByZero)
            } else {
                CellValue::Number(as_number(&l) / rn)
            }
        }
        Op::Gt => CellValue::Number(if as_number(&l) > as_number(&r) { 1.0 } else { 0.0 }),
        Op::Lt => CellValue::Number(if as_number(&l) < as_number(&r) { 1.0 } else { 0.0 }),
        Op::Eq => CellValue::Number(if values_equal(&l, &r) { 1.0 } else { 0.0 }),
    }
}

fn resolve_range(expr: &Expr, lookup: &dyn CellLookup) -> Option<(CellAddr, CellAddr)> {
    match expr {
        Expr::Range(from, to) => Some((*from, *to)),
        Expr::NamedRange(name) => lookup.named_range(name),
        _ => None,
    }
}

fn eval_function(name: FnName, args: &[Expr], lookup: &dyn CellLookup) -> CellValue {
    match name {
        FnName::Sum => eval_aggregate(args, lookup, true),
        FnName::Avg => eval_aggregate(args, lookup, false),
        FnName::If => eval_if(args, lookup),
        FnName::VLookup => eval_vlookup(args, lookup),
    }
}

fn eval_aggregate(args: &[Expr], lookup: &dyn CellLookup, is_sum: bool) -> CellValue {
    let mut total = 0.0;
    let mut count = 0u32;
    for arg in args {
        if let Some((from, to)) = resolve_range(arg, lookup) {
            for addr in collect_range_addrs(from, to) {
                let v = lookup.value_of(addr);
                if let CellValue::Error(_) = v {
                    return CellValue::Error(ErrorKind::PropagatedError);
                }
                if let CellValue::Number(n) = v {
                    total += n;
                    count += 1;
                }
            }
        } else {
            let v = evaluate(arg, lookup);
            if let CellValue::Error(_) = v {
                return CellValue::Error(ErrorKind::PropagatedError);
            }
            if let CellValue::Number(n) = v {
                total += n;
                count += 1;
            }
        }
    }
    if is_sum {
        CellValue::Number(total)
    } else if count == 0 {
        CellValue::Error(ErrorKind::DivByZero)
    } else {
        CellValue::Number(total / count as f64)
    }
}

fn eval_if(args: &[Expr], lookup: &dyn CellLookup) -> CellValue {
    if args.len() != 3 {
        return CellValue::Error(ErrorKind::BadRef);
    }
    let cond = evaluate(&args[0], lookup);
    if let CellValue::Error(_) = cond {
        return CellValue::Error(ErrorKind::PropagatedError);
    }
    let truthy = matches!(cond, CellValue::Number(n) if n != 0.0);
    if truthy {
        evaluate(&args[1], lookup)
    } else {
        evaluate(&args[2], lookup)
    }
}

fn eval_vlookup(args: &[Expr], lookup: &dyn CellLookup) -> CellValue {
    if args.len() != 3 {
        return CellValue::Error(ErrorKind::BadRef);
    }
    let key = evaluate(&args[0], lookup);
    if let CellValue::Error(_) = key {
        return CellValue::Error(ErrorKind::PropagatedError);
    }
    let (from, to) = match resolve_range(&args[1], lookup) {
        Some(r) => r,
        None => return CellValue::Error(ErrorKind::BadRef),
    };
    let col_index = match evaluate(&args[2], lookup) {
        CellValue::Number(n) if n >= 1.0 => n as usize,
        _ => return CellValue::Error(ErrorKind::BadRef),
    };
    let (c0, c1) = (from.col.min(to.col), from.col.max(to.col));
    let (r0, r1) = (from.row.min(to.row), from.row.max(to.row));
    let num_cols = c1 - c0 + 1;
    if col_index == 0 || col_index as u32 > num_cols {
        return CellValue::Error(ErrorKind::BadRef);
    }
    for row in r0..=r1 {
        let candidate = lookup.value_of(CellAddr { col: c0, row });
        if let CellValue::Error(_) = candidate {
            return CellValue::Error(ErrorKind::PropagatedError);
        }
        if values_equal(&candidate, &key) {
            let target = CellAddr { col: c0 + (col_index as u32 - 1), row };
            return lookup.value_of(target);
        }
    }
    CellValue::Error(ErrorKind::BadRef)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct FakeSheet {
        cells: HashMap<CellAddr, CellValue>,
        named: HashMap<String, (CellAddr, CellAddr)>,
    }

    impl CellLookup for FakeSheet {
        fn value_of(&self, addr: CellAddr) -> CellValue {
            self.cells.get(&addr).cloned().unwrap_or(CellValue::Empty)
        }
        fn named_range(&self, name: &str) -> Option<(CellAddr, CellAddr)> {
            self.named.get(name).copied()
        }
    }

    fn a(reference: &str) -> CellAddr {
        CellAddr::parse(reference).unwrap()
    }

    fn sheet(pairs: &[(&str, CellValue)]) -> FakeSheet {
        FakeSheet {
            cells: pairs.iter().map(|(r, v)| (a(r), v.clone())).collect(),
            named: HashMap::new(),
        }
    }

    #[test]
    fn arithmetic_precedence_via_evaluate() {
        let s = sheet(&[]);
        let expr = Expr::BinaryOp(
            Op::Add,
            Box::new(Expr::Number(1.0)),
            Box::new(Expr::BinaryOp(Op::Mul, Box::new(Expr::Number(2.0)), Box::new(Expr::Number(3.0)))),
        );
        assert_eq!(evaluate(&expr, &s), CellValue::Number(7.0));
    }

    #[test]
    fn division_by_zero_is_an_error() {
        let s = sheet(&[]);
        let expr = Expr::BinaryOp(Op::Div, Box::new(Expr::Number(1.0)), Box::new(Expr::Number(0.0)));
        assert_eq!(evaluate(&expr, &s), CellValue::Error(ErrorKind::DivByZero));
    }

    #[test]
    fn sum_treats_missing_and_text_cells_as_zero() {
        let s = sheet(&[("A1", CellValue::Number(5.0)), ("A2", CellValue::Text("x".to_string()))]);
        let expr = Expr::FunctionCall(FnName::Sum, vec![Expr::Range(a("A1"), a("A3"))]);
        assert_eq!(evaluate(&expr, &s), CellValue::Number(5.0));
    }

    #[test]
    fn sum_propagates_an_error_cell_in_range() {
        let s = sheet(&[("A1", CellValue::Number(5.0)), ("A2", CellValue::Error(ErrorKind::DivByZero))]);
        let expr = Expr::FunctionCall(FnName::Sum, vec![Expr::Range(a("A1"), a("A2"))]);
        assert_eq!(evaluate(&expr, &s), CellValue::Error(ErrorKind::PropagatedError));
    }

    #[test]
    fn avg_of_empty_range_is_div_by_zero_error() {
        let s = sheet(&[]);
        let expr = Expr::FunctionCall(FnName::Avg, vec![Expr::Range(a("A1"), a("A3"))]);
        assert_eq!(evaluate(&expr, &s), CellValue::Error(ErrorKind::DivByZero));
    }

    #[test]
    fn avg_divides_by_count_of_numeric_cells_only() {
        let s = sheet(&[("A1", CellValue::Number(2.0)), ("A2", CellValue::Number(4.0))]);
        let expr = Expr::FunctionCall(FnName::Avg, vec![Expr::Range(a("A1"), a("A3"))]);
        assert_eq!(evaluate(&expr, &s), CellValue::Number(3.0));
    }

    #[test]
    fn if_picks_the_correct_branch() {
        let s = sheet(&[("A1", CellValue::Number(20.0))]);
        let expr = Expr::FunctionCall(
            FnName::If,
            vec![
                Expr::BinaryOp(Op::Gt, Box::new(Expr::CellRef(a("A1"))), Box::new(Expr::Number(10.0))),
                Expr::Text("big".to_string()),
                Expr::Text("small".to_string()),
            ],
        );
        assert_eq!(evaluate(&expr, &s), CellValue::Text("big".to_string()));
    }

    #[test]
    fn vlookup_exact_match_returns_target_column() {
        let s = sheet(&[
            ("A1", CellValue::Text("apple".to_string())),
            ("B1", CellValue::Number(1.0)),
            ("A2", CellValue::Text("pear".to_string())),
            ("B2", CellValue::Number(2.0)),
        ]);
        let expr = Expr::FunctionCall(
            FnName::VLookup,
            vec![Expr::Text("pear".to_string()), Expr::Range(a("A1"), a("B2")), Expr::Number(2.0)],
        );
        assert_eq!(evaluate(&expr, &s), CellValue::Number(2.0));
    }

    #[test]
    fn vlookup_no_match_is_bad_ref_error() {
        let s = sheet(&[("A1", CellValue::Text("apple".to_string()))]);
        let expr = Expr::FunctionCall(
            FnName::VLookup,
            vec![Expr::Text("missing".to_string()), Expr::Range(a("A1"), a("B1")), Expr::Number(2.0)],
        );
        assert_eq!(evaluate(&expr, &s), CellValue::Error(ErrorKind::BadRef));
    }

    #[test]
    fn named_range_resolves_through_lookup() {
        let mut s = sheet(&[("A1", CellValue::Number(3.0)), ("A2", CellValue::Number(4.0))]);
        s.named.insert("SALES".to_string(), (a("A1"), a("A2")));
        let expr = Expr::FunctionCall(FnName::Sum, vec![Expr::NamedRange("SALES".to_string())]);
        assert_eq!(evaluate(&expr, &s), CellValue::Number(7.0));
    }

    #[test]
    fn vlookup_propagates_error_from_key_column() {
        let s = sheet(&[
            ("A1", CellValue::Error(ErrorKind::DivByZero)),
            ("B1", CellValue::Number(1.0)),
            ("A2", CellValue::Text("pear".to_string())),
            ("B2", CellValue::Number(2.0)),
        ]);
        let expr = Expr::FunctionCall(
            FnName::VLookup,
            vec![Expr::Text("pear".to_string()), Expr::Range(a("A1"), a("B2")), Expr::Number(2.0)],
        );
        assert_eq!(evaluate(&expr, &s), CellValue::Error(ErrorKind::PropagatedError));
    }
}
