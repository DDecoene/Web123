// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use crate::engine::SpreadsheetCore;
use crate::model::CellAddr;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Ready,
    Edit,
    Point,
    SlashMenu,
    GoTo,
}

#[derive(Clone, Debug, PartialEq)]
enum SlashStage {
    Closed,
    MenuOpen,
    PickingRange { anchor: CellAddr, end: CellAddr },
    NamingRange { range: (CellAddr, CellAddr), buffer: String },
}

pub struct Editor {
    active: CellAddr,
    mode: Mode,
    edit_buffer: String,
    point_cursor: CellAddr,
    goto_buffer: String,
    slash_stage: SlashStage,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            active: CellAddr { col: 0, row: 0 },
            mode: Mode::Ready,
            edit_buffer: String::new(),
            point_cursor: CellAddr { col: 0, row: 0 },
            goto_buffer: String::new(),
            slash_stage: SlashStage::Closed,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn active_cell(&self) -> CellAddr {
        self.active
    }

    pub fn edit_buffer(&self) -> &str {
        &self.edit_buffer
    }

    pub fn point_cell(&self) -> CellAddr {
        self.point_cursor
    }

    pub fn handle_key(&mut self, key: &str, core: &mut SpreadsheetCore) {
        match self.mode {
            Mode::Ready => self.handle_key_ready(key, core),
            Mode::Edit => self.handle_key_edit(key, core),
            Mode::Point => self.handle_key_point(key, core),
            Mode::SlashMenu => self.handle_key_slash(key, core),
            Mode::GoTo => self.handle_key_goto(key),
        }
    }

    fn move_active(&mut self, d_col: i32, d_row: i32) {
        let new_col = (self.active.col as i32 + d_col).max(0) as u32;
        let new_row = (self.active.row as i32 + d_row).max(0) as u32;
        self.active = CellAddr { col: new_col, row: new_row };
    }

    fn handle_key_ready(&mut self, key: &str, core: &mut SpreadsheetCore) {
        match key {
            "ArrowUp" => self.move_active(0, -1),
            "ArrowDown" => self.move_active(0, 1),
            "ArrowLeft" => self.move_active(-1, 0),
            "ArrowRight" => self.move_active(1, 0),
            "F2" => {
                self.edit_buffer = core.raw_input(self.active);
                self.mode = Mode::Edit;
            }
            "F5" => {
                self.goto_buffer.clear();
                self.mode = Mode::GoTo;
            }
            "F9" => core.recalculate_all(),
            "/" => {
                self.slash_stage = SlashStage::MenuOpen;
                self.mode = Mode::SlashMenu;
            }
            k if is_value_start(k) => {
                self.edit_buffer = k.to_string();
                self.mode = Mode::Edit;
            }
            _ => {}
        }
    }

    fn handle_key_edit(&mut self, key: &str, core: &mut SpreadsheetCore) {
        match key {
            "Enter" => {
                core.set_cell(&self.active.to_string(), &self.edit_buffer);
                self.edit_buffer.clear();
                self.move_active(0, 1);
                self.mode = Mode::Ready;
            }
            "Escape" => {
                self.edit_buffer.clear();
                self.mode = Mode::Ready;
            }
            "Backspace" => {
                self.edit_buffer.pop();
            }
            "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight"
                if reference_expected(&self.edit_buffer) =>
            {
                self.point_cursor = self.active;
                self.mode = Mode::Point;
                self.handle_key_point(key, core);
            }
            k if k.chars().count() == 1 => self.edit_buffer.push_str(k),
            _ => {}
        }
    }

    fn handle_key_point(&mut self, key: &str, core: &mut SpreadsheetCore) {
        match key {
            "ArrowUp" => {
                self.point_cursor = CellAddr {
                    col: self.point_cursor.col,
                    row: self.point_cursor.row.saturating_sub(1),
                }
            }
            "ArrowDown" => {
                self.point_cursor = CellAddr { col: self.point_cursor.col, row: self.point_cursor.row + 1 }
            }
            "ArrowLeft" => {
                self.point_cursor = CellAddr {
                    col: self.point_cursor.col.saturating_sub(1),
                    row: self.point_cursor.row,
                }
            }
            "ArrowRight" => {
                self.point_cursor = CellAddr { col: self.point_cursor.col + 1, row: self.point_cursor.row }
            }
            "Enter" => {
                self.edit_buffer.push_str(&self.point_cursor.to_string());
                core.set_cell(&self.active.to_string(), &self.edit_buffer);
                self.edit_buffer.clear();
                self.move_active(0, 1);
                self.mode = Mode::Ready;
            }
            k if k.chars().count() == 1 && !k.chars().next().unwrap().is_alphanumeric() => {
                self.edit_buffer.push_str(&self.point_cursor.to_string());
                self.edit_buffer.push_str(k);
                self.mode = Mode::Edit;
            }
            _ => {}
        }
    }

    fn handle_key_goto(&mut self, key: &str) {
        match key {
            "Enter" => {
                if let Some(addr) = CellAddr::parse(&self.goto_buffer) {
                    self.active = addr;
                }
                self.goto_buffer.clear();
                self.mode = Mode::Ready;
            }
            "Escape" => {
                self.goto_buffer.clear();
                self.mode = Mode::Ready;
            }
            "Backspace" => {
                self.goto_buffer.pop();
            }
            k if k.chars().count() == 1 => self.goto_buffer.push_str(k),
            _ => {}
        }
    }

    fn handle_key_slash(&mut self, key: &str, core: &mut SpreadsheetCore) {
        match &mut self.slash_stage {
            SlashStage::MenuOpen => match key {
                "r" | "R" => {
                    self.slash_stage =
                        SlashStage::PickingRange { anchor: self.active, end: self.active };
                }
                _ => {
                    self.slash_stage = SlashStage::Closed;
                    self.mode = Mode::Ready;
                }
            },
            SlashStage::PickingRange { anchor, end } => match key {
                "ArrowUp" => end.row = end.row.saturating_sub(1),
                "ArrowDown" => end.row += 1,
                "ArrowLeft" => end.col = end.col.saturating_sub(1),
                "ArrowRight" => end.col += 1,
                "Enter" => {
                    let range = (*anchor, *end);
                    self.slash_stage = SlashStage::NamingRange { range, buffer: String::new() };
                }
                "Escape" => {
                    self.slash_stage = SlashStage::Closed;
                    self.mode = Mode::Ready;
                }
                _ => {}
            },
            SlashStage::NamingRange { range, buffer } => match key {
                "Enter" => {
                    core.define_named_range(buffer.clone(), range.0, range.1);
                    self.slash_stage = SlashStage::Closed;
                    self.mode = Mode::Ready;
                }
                "Escape" => {
                    self.slash_stage = SlashStage::Closed;
                    self.mode = Mode::Ready;
                }
                "Backspace" => {
                    buffer.pop();
                }
                k if k.chars().count() == 1 => buffer.push_str(k),
                _ => {}
            },
            SlashStage::Closed => {
                self.mode = Mode::Ready;
            }
        }
    }
}

fn is_value_start(k: &str) -> bool {
    k.chars().next().map_or(false, |c| c.is_ascii_alphanumeric() || c == '=' || c == '@' || c == '-')
        && k.chars().count() == 1
}

fn reference_expected(buffer: &str) -> bool {
    match buffer.chars().last() {
        None => true,
        Some(c) => matches!(c, '(' | ',' | '+' | '-' | '*' | '/'),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(reference: &str) -> CellAddr {
        CellAddr::parse(reference).unwrap()
    }

    #[test]
    fn starts_in_ready_mode_on_a1() {
        let editor = Editor::new();
        assert_eq!(editor.mode(), Mode::Ready);
        assert_eq!(editor.active_cell(), a("A1"));
    }

    #[test]
    fn typing_a_value_and_enter_commits_it_and_moves_down() {
        let mut core = SpreadsheetCore::new();
        let mut editor = Editor::new();
        editor.handle_key("5", &mut core);
        assert_eq!(editor.mode(), Mode::Edit);
        editor.handle_key("Enter", &mut core);
        assert_eq!(editor.mode(), Mode::Ready);
        assert_eq!(editor.active_cell(), a("A2"));
        assert_eq!(core.display(a("A1")), "5");
    }

    #[test]
    fn f2_reopens_an_existing_formula_for_editing() {
        let mut core = SpreadsheetCore::new();
        let mut editor = Editor::new();
        editor.handle_key("5", &mut core);
        editor.handle_key("Enter", &mut core);
        editor.handle_key("ArrowUp", &mut core);
        assert_eq!(editor.active_cell(), a("A1"));
        editor.handle_key("F2", &mut core);
        assert_eq!(editor.mode(), Mode::Edit);
        assert_eq!(editor.edit_buffer(), "5");
        editor.handle_key("Backspace", &mut core);
        editor.handle_key("9", &mut core);
        editor.handle_key("Enter", &mut core);
        assert_eq!(core.display(a("A1")), "9");
    }

    #[test]
    fn f5_goto_jumps_the_active_cell() {
        let mut core = SpreadsheetCore::new();
        let mut editor = Editor::new();
        editor.handle_key("F5", &mut core);
        assert_eq!(editor.mode(), Mode::GoTo);
        for c in "C10".chars() {
            editor.handle_key(&c.to_string(), &mut core);
        }
        editor.handle_key("Enter", &mut core);
        assert_eq!(editor.mode(), Mode::Ready);
        assert_eq!(editor.active_cell(), a("C10"));
    }

    #[test]
    fn point_mode_splices_cell_references_into_a_sum_formula() {
        let mut core = SpreadsheetCore::new();
        let mut editor = Editor::new();
        editor.handle_key("3", &mut core);
        editor.handle_key("Enter", &mut core); // A1 = 3, active -> A2
        editor.handle_key("4", &mut core);
        editor.handle_key("Enter", &mut core); // A2 = 4, active -> A3

        editor.handle_key("F5", &mut core);
        for c in "B1".chars() {
            editor.handle_key(&c.to_string(), &mut core);
        }
        editor.handle_key("Enter", &mut core); // active -> B1

        for c in "@SUM(".chars() {
            editor.handle_key(&c.to_string(), &mut core);
        }
        editor.handle_key("ArrowLeft", &mut core); // enters POINT at B1, moves to A1
        assert_eq!(editor.mode(), Mode::Point);
        assert_eq!(editor.point_cell(), a("A1"));
        editor.handle_key(",", &mut core); // commits "A1," and returns to EDIT

        editor.handle_key("ArrowLeft", &mut core); // re-enters POINT at B1, moves to A1
        editor.handle_key("ArrowDown", &mut core); // moves to A2
        assert_eq!(editor.point_cell(), a("A2"));
        editor.handle_key(")", &mut core); // commits "A2)" and returns to EDIT

        editor.handle_key("Enter", &mut core);
        assert_eq!(core.display(a("B1")), "7");
    }

    #[test]
    fn slash_range_name_defines_a_named_range_usable_in_a_formula() {
        let mut core = SpreadsheetCore::new();
        let mut editor = Editor::new();
        editor.handle_key("3", &mut core);
        editor.handle_key("Enter", &mut core); // A1 = 3, active -> A2
        editor.handle_key("4", &mut core);
        editor.handle_key("Enter", &mut core); // A2 = 4, active -> A3
        editor.handle_key("ArrowUp", &mut core);
        editor.handle_key("ArrowUp", &mut core); // active -> A1

        editor.handle_key("/", &mut core);
        assert_eq!(editor.mode(), Mode::SlashMenu);
        editor.handle_key("r", &mut core); // /Range
        editor.handle_key("ArrowDown", &mut core); // extend selection to A2
        editor.handle_key("Enter", &mut core); // confirm range, prompt for name
        for c in "SALES".chars() {
            editor.handle_key(&c.to_string(), &mut core);
        }
        editor.handle_key("Enter", &mut core); // name committed
        assert_eq!(editor.mode(), Mode::Ready);

        editor.handle_key("F5", &mut core);
        for c in "B1".chars() {
            editor.handle_key(&c.to_string(), &mut core);
        }
        editor.handle_key("Enter", &mut core);
        for c in "@SUM(SALES)".chars() {
            editor.handle_key(&c.to_string(), &mut core);
        }
        editor.handle_key("Enter", &mut core);
        assert_eq!(core.display(a("B1")), "7");
    }

    #[test]
    fn circular_reference_reports_error_state_via_is_error() {
        let mut core = SpreadsheetCore::new();
        core.set_cell("A1", "=B1");
        core.set_cell("B1", "=A1");
        assert!(core.is_error(a("A1")));
    }
}
