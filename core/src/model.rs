// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CellAddr {
    pub col: u32,
    pub row: u32,
}

impl CellAddr {
    pub fn parse(s: &str) -> Option<CellAddr> {
        let upper = s.to_uppercase();
        let split_idx = upper.find(|c: char| c.is_ascii_digit())?;
        let (col_str, row_str) = upper.split_at(split_idx);
        if col_str.is_empty() || row_str.is_empty() {
            return None;
        }
        if !col_str.chars().all(|c| c.is_ascii_uppercase()) {
            return None;
        }
        let mut col: u32 = 0;
        for c in col_str.chars() {
            col = col * 26 + (c as u32 - 'A' as u32 + 1);
        }
        let col = col - 1;
        let row: u32 = row_str.parse().ok()?;
        if row == 0 {
            return None;
        }
        Some(CellAddr { col, row: row - 1 })
    }
}

impl std::fmt::Display for CellAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut col = self.col + 1;
        let mut letters = Vec::new();
        while col > 0 {
            let rem = (col - 1) % 26;
            letters.push((b'A' + rem as u8) as char);
            col = (col - 1) / 26;
        }
        letters.reverse();
        write!(f, "{}{}", letters.into_iter().collect::<String>(), self.row + 1)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorKind {
    CircularRef,
    DivByZero,
    PropagatedError,
    BadRef,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CellValue {
    Empty,
    Number(f64),
    Text(String),
    Formula(Expr),
    Error(ErrorKind),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Number(f64),
    Text(String),
    CellRef(CellAddr),
    Range(CellAddr, CellAddr),
    NamedRange(String),
    BinaryOp(Op, Box<Expr>, Box<Expr>),
    FunctionCall(FnName, Vec<Expr>),
    Invalid,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Gt,
    Lt,
    Eq,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FnName {
    Sum,
    Avg,
    If,
    VLookup,
}

pub fn collect_range_addrs(from: CellAddr, to: CellAddr) -> Vec<CellAddr> {
    let (c0, c1) = (from.col.min(to.col), from.col.max(to.col));
    let (r0, r1) = (from.row.min(to.row), from.row.max(to.row));
    let mut out = Vec::new();
    for row in r0..=r1 {
        for col in c0..=c1 {
            out.push(CellAddr { col, row });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_letter_column_and_row() {
        assert_eq!(CellAddr::parse("A1"), Some(CellAddr { col: 0, row: 0 }));
        assert_eq!(CellAddr::parse("Z10"), Some(CellAddr { col: 25, row: 9 }));
    }

    #[test]
    fn parses_multi_letter_column() {
        assert_eq!(CellAddr::parse("AA1"), Some(CellAddr { col: 26, row: 0 }));
    }

    #[test]
    fn lowercase_input_is_accepted() {
        assert_eq!(CellAddr::parse("a1"), Some(CellAddr { col: 0, row: 0 }));
    }

    #[test]
    fn rejects_row_zero_and_malformed_input() {
        assert_eq!(CellAddr::parse("A0"), None);
        assert_eq!(CellAddr::parse("1A"), None);
        assert_eq!(CellAddr::parse(""), None);
        assert_eq!(CellAddr::parse("A"), None);
    }

    #[test]
    fn to_string_round_trips() {
        assert_eq!(CellAddr { col: 0, row: 0 }.to_string(), "A1");
        assert_eq!(CellAddr { col: 25, row: 9 }.to_string(), "Z10");
        assert_eq!(CellAddr { col: 26, row: 0 }.to_string(), "AA1");
    }

    #[test]
    fn collects_addresses_in_a_rectangular_range_regardless_of_corner_order() {
        let forward = collect_range_addrs(
            CellAddr { col: 0, row: 0 },
            CellAddr { col: 1, row: 1 },
        );
        let reversed = collect_range_addrs(
            CellAddr { col: 1, row: 1 },
            CellAddr { col: 0, row: 0 },
        );
        assert_eq!(
            forward,
            vec![
                CellAddr { col: 0, row: 0 },
                CellAddr { col: 1, row: 0 },
                CellAddr { col: 0, row: 1 },
                CellAddr { col: 1, row: 1 },
            ]
        );
        assert_eq!(forward, reversed);
    }
}
