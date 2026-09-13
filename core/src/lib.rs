// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

mod model;
mod parser;
mod functions;
mod engine;
mod editor;

use wasm_bindgen::prelude::*;

use engine::SpreadsheetCore;
use model::CellAddr;

#[wasm_bindgen]
pub struct Spreadsheet {
    core: SpreadsheetCore,
}

#[wasm_bindgen]
impl Spreadsheet {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Spreadsheet {
        Spreadsheet { core: SpreadsheetCore::new() }
    }

    #[wasm_bindgen(js_name = setCell)]
    pub fn set_cell(&mut self, reference: &str, input: &str) {
        self.core.set_cell(reference, input);
    }

    #[wasm_bindgen(js_name = getDisplay)]
    pub fn get_display(&self, reference: &str) -> String {
        match CellAddr::parse(reference) {
            Some(addr) => self.core.display(addr),
            None => String::new(),
        }
    }
}
