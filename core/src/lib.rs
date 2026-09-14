// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

mod model;
mod parser;
mod functions;
mod engine;
mod editor;
mod document;
mod storage;

use wasm_bindgen::prelude::*;

use document::DocumentStore;
use editor::{Editor, Mode};
use model::CellAddr;

#[wasm_bindgen]
pub struct Spreadsheet {
    store: DocumentStore,
    editor: Editor,
}

#[wasm_bindgen]
impl Spreadsheet {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Spreadsheet {
        Spreadsheet { store: DocumentStore::new(), editor: Editor::new() }
    }

    #[wasm_bindgen(js_name = setCell)]
    pub fn set_cell(&mut self, reference: &str, input: &str) {
        self.store.set_cell(reference, input);
    }

    #[wasm_bindgen(js_name = getDisplay)]
    pub fn get_display(&self, reference: &str) -> String {
        match CellAddr::parse(reference) {
            Some(addr) => self.store.display(addr),
            None => String::new(),
        }
    }

    #[wasm_bindgen(js_name = handleKey)]
    pub fn handle_key(&mut self, key: &str) {
        self.editor.handle_key(key, &mut self.store);
    }

    #[wasm_bindgen(js_name = getMode)]
    pub fn get_mode(&self) -> String {
        if self.editor.mode() == Mode::Ready && self.store.is_error(self.editor.active_cell()) {
            return "ERROR".to_string();
        }
        mode_name(self.editor.mode()).to_string()
    }

    #[wasm_bindgen(js_name = getActiveCell)]
    pub fn get_active_cell(&self) -> String {
        self.editor.active_cell().to_string()
    }

    #[wasm_bindgen(js_name = getEditBuffer)]
    pub fn get_edit_buffer(&self) -> String {
        self.editor.edit_buffer().to_string()
    }

    #[wasm_bindgen(js_name = getPointCell)]
    pub fn get_point_cell(&self) -> String {
        self.editor.point_cell().to_string()
    }
}

fn mode_name(m: Mode) -> &'static str {
    match m {
        Mode::Ready => "READY",
        Mode::Edit => "EDIT",
        Mode::Point => "POINT",
        Mode::SlashMenu => "MENU",
        Mode::GoTo => "GOTO",
    }
}
