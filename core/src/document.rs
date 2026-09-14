// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use automerge::{transaction::Transactable, AutoCommit, ObjId, ObjType, ReadDoc, Value, ValueRef, ScalarValueRef};

use crate::engine::SpreadsheetCore;
use crate::model::CellAddr;

pub struct DocumentStore {
    doc: AutoCommit,
    cells: ObjId,
    named_ranges: ObjId,
    core: SpreadsheetCore,
}

impl DocumentStore {
    pub fn new() -> Self {
        let mut doc = AutoCommit::new();
        let cells = get_or_create_map(&mut doc, "cells");
        let named_ranges = get_or_create_map(&mut doc, "named_ranges");
        DocumentStore { doc, cells, named_ranges, core: SpreadsheetCore::new() }
    }

    pub fn set_cell(&mut self, reference: &str, input: &str) {
        let Some(addr) = CellAddr::parse(reference) else { return };
        let key = addr.to_string();
        if input.trim().is_empty() {
            let _ = self.doc.delete(&self.cells, key.as_str());
        } else {
            let _ = self.doc.put(&self.cells, key.as_str(), input);
        }
        self.core.set_cell(reference, input);
        self.spawn_save();
    }

    pub fn display(&self, addr: CellAddr) -> String {
        self.core.display(addr)
    }

    pub fn raw_input(&self, addr: CellAddr) -> String {
        self.core.raw_input(addr)
    }

    pub fn is_error(&self, addr: CellAddr) -> bool {
        self.core.is_error(addr)
    }

    pub fn recalculate_all(&mut self) {
        self.core.recalculate_all();
    }

    pub fn define_named_range(&mut self, name: String, from: CellAddr, to: CellAddr) {
        let value = format!("{from}:{to}");
        let _ = self.doc.put(&self.named_ranges, name.as_str(), value.as_str());
        self.core.define_named_range(name, from, to);
        self.spawn_save();
    }

    pub fn save_bytes(&mut self) -> Vec<u8> {
        self.doc.save()
    }

    /// Boots a store from whatever's in IndexedDB, or an empty document if
    /// there's nothing there yet (first run, or storage unavailable).
    pub async fn load_from_storage() -> Self {
        match crate::storage::load().await {
            Some(bytes) => DocumentStore::load_bytes(&bytes).unwrap_or_else(|_| DocumentStore::new()),
            None => DocumentStore::new(),
        }
    }

    /// Fire-and-forget: doesn't block the caller on the IndexedDB write.
    /// A no-op on the host test target (see `storage::save`'s native stub).
    fn spawn_save(&self) {
        // Cloning the doc lets us call `save()` (which needs `&mut self` in
        // automerge 0.11) without requiring `spawn_save` itself to take
        // `&mut self` and complicating its call sites.
        let mut doc = self.doc.clone();
        let bytes = doc.save();
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            crate::storage::save(&bytes).await;
        });
        #[cfg(not(target_arch = "wasm32"))]
        let _ = bytes;
    }

    pub fn load_bytes(bytes: &[u8]) -> Result<Self, automerge::AutomergeError> {
        let mut doc = AutoCommit::load(bytes)?;
        let cells = get_or_create_map(&mut doc, "cells");
        let named_ranges = get_or_create_map(&mut doc, "named_ranges");
        let mut store = DocumentStore { doc, cells, named_ranges, core: SpreadsheetCore::new() };
        store.replay_all();
        Ok(store)
    }

    /// Rebuilds `core` from scratch by reading every key currently in the
    /// Automerge doc. Named ranges are replayed before cells so a formula
    /// cell referencing a name resolves on its first `set_cell` call rather
    /// than needing a later recalc pass.
    fn replay_all(&mut self) {
        let mut core = SpreadsheetCore::new();
        for (name, value) in map_entries(&self.doc, &self.named_ranges) {
            if let Some((from, to)) = parse_range(&value) {
                core.define_named_range(name, from, to);
            }
        }
        for (addr, input) in map_entries(&self.doc, &self.cells) {
            core.set_cell(&addr, &input);
        }
        self.core = core;
    }
}

impl DocumentStore {
    /// Produces an independent copy that can diverge from `self` — stands
    /// in for "peer B" in tests, since there's no networking yet to create
    /// a real second peer. Automerge 0.11's `AutoCommit::fork` takes
    /// `&mut self` (it needs to flush any open transaction before cloning
    /// the underlying doc), so this does too.
    pub fn fork(&mut self) -> Self {
        let doc = self.doc.fork();
        // fork() clones the document contents but not our cached ObjIds'
        // validity guarantees across instances, so re-resolve them the
        // same way `load_bytes` does.
        let mut store = DocumentStore {
            cells: get_or_create_map_immut(&doc, "cells"),
            named_ranges: get_or_create_map_immut(&doc, "named_ranges"),
            doc,
            core: SpreadsheetCore::new(),
        };
        store.replay_all();
        store
    }

    /// Merges `other`'s changes into `self` and replays the result. `other`
    /// is left unchanged by Automerge's merge semantics on `self`'s side
    /// only, so callers that want both sides converged call `merge` on
    /// both stores with each other.
    pub fn merge(&mut self, other: &mut DocumentStore) {
        let _ = self.doc.merge(&mut other.doc);
        self.replay_all();
    }
}

fn get_or_create_map_immut(doc: &AutoCommit, key: &str) -> ObjId {
    match doc.get(automerge::ROOT, key) {
        Ok(Some((Value::Object(ObjType::Map), id))) => id,
        _ => panic!("expected '{key}' map to already exist on a forked/loaded doc"),
    }
}

fn get_or_create_map(doc: &mut AutoCommit, key: &str) -> ObjId {
    if let Ok(Some((Value::Object(ObjType::Map), id))) = doc.get(automerge::ROOT, key) {
        return id;
    }
    doc.put_object(automerge::ROOT, key, ObjType::Map)
        .expect("root map put_object cannot fail on a fresh key")
}

fn map_entries(doc: &AutoCommit, obj: &ObjId) -> Vec<(String, String)> {
    doc.map_range(obj, ..)
        .filter_map(|item| {
            if let ValueRef::Scalar(scalar) = item.value {
                match scalar {
                    ScalarValueRef::Str(s) => Some((item.key.to_string(), s.to_string())),
                    _ => None,
                }
            } else {
                None
            }
        })
        .collect()
}

fn parse_range(s: &str) -> Option<(CellAddr, CellAddr)> {
    let (from_str, to_str) = s.split_once(':')?;
    Some((CellAddr::parse(from_str)?, CellAddr::parse(to_str)?))
}

#[cfg(test)]
mod merge_tests {
    use super::*;

    fn a(reference: &str) -> CellAddr {
        CellAddr::parse(reference).unwrap()
    }

    #[test]
    fn edits_to_different_cells_on_two_forks_both_survive_a_merge() {
        let mut a_store = DocumentStore::new();
        a_store.set_cell("A1", "1");
        let mut b_store = a_store.fork();

        a_store.set_cell("B1", "2");
        b_store.set_cell("C1", "3");

        a_store.merge(&mut b_store);

        assert_eq!(a_store.display(a("A1")), "1");
        assert_eq!(a_store.display(a("B1")), "2");
        assert_eq!(a_store.display(a("C1")), "3");
    }

    #[test]
    fn offline_edits_on_both_sides_converge_after_merging_in_both_directions() {
        let mut a_store = DocumentStore::new();
        let mut b_store = a_store.fork();

        a_store.set_cell("A1", "10");
        b_store.set_cell("B1", "20");

        a_store.merge(&mut b_store);
        b_store.merge(&mut a_store);

        assert_eq!(a_store.display(a("A1")), b_store.display(a("A1")));
        assert_eq!(a_store.display(a("B1")), b_store.display(a("B1")));
    }

    #[test]
    fn conflicting_edits_to_the_same_cell_resolve_deterministically_on_both_sides() {
        let mut a_store = DocumentStore::new();
        let mut b_store = a_store.fork();

        // Both peers edit A1 independently while offline from each other.
        a_store.set_cell("A1", "from-a");
        b_store.set_cell("A1", "from-b");

        a_store.merge(&mut b_store);
        b_store.merge(&mut a_store);

        // Automerge's default conflict resolution (last-writer-wins by
        // change ordering) must pick the *same* value on every replica —
        // that's the property under test, not which specific value wins.
        assert_eq!(a_store.raw_input(a("A1")), b_store.raw_input(a("A1")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(reference: &str) -> CellAddr {
        CellAddr::parse(reference).unwrap()
    }

    #[test]
    fn set_cell_is_immediately_visible_through_display() {
        let mut store = DocumentStore::new();
        store.set_cell("A1", "3");
        store.set_cell("A2", "4");
        store.set_cell("B1", "@SUM(A1,A2)");
        assert_eq!(store.display(a("B1")), "7");
    }

    #[test]
    fn raw_input_and_is_error_pass_through_to_core() {
        let mut store = DocumentStore::new();
        store.set_cell("A1", "=1/0");
        assert_eq!(store.raw_input(a("A1")), "=1/0");
        assert!(store.is_error(a("A1")));
    }

    #[test]
    fn save_then_load_round_trips_to_identical_display_state() {
        let mut store = DocumentStore::new();
        store.set_cell("A1", "3");
        store.set_cell("A2", "4");
        store.define_named_range("SALES".to_string(), a("A1"), a("A2"));
        store.set_cell("B1", "@SUM(SALES)");

        let bytes = store.save_bytes();
        let loaded = DocumentStore::load_bytes(&bytes).unwrap();

        assert_eq!(loaded.display(a("B1")), "7");
        assert_eq!(loaded.raw_input(a("B1")), "@SUM(SALES)");
    }

    #[test]
    fn clearing_a_cell_removes_it_from_the_doc_so_a_replay_leaves_it_empty() {
        let mut store = DocumentStore::new();
        store.set_cell("A1", "5");
        store.set_cell("A1", "");

        let bytes = store.save_bytes();
        let loaded = DocumentStore::load_bytes(&bytes).unwrap();
        assert_eq!(loaded.display(a("A1")), "");
    }

    #[test]
    fn named_range_replay_resolves_a_formula_defined_before_the_name_existed() {
        let mut store = DocumentStore::new();
        store.set_cell("A1", "3");
        store.set_cell("A2", "4");
        // B1 references SALES before SALES is defined — matches the
        // existing Plan 2 guarantee that this later resolves without F9.
        store.set_cell("B1", "@SUM(SALES)");
        store.define_named_range("SALES".to_string(), a("A1"), a("A2"));

        let bytes = store.save_bytes();
        let loaded = DocumentStore::load_bytes(&bytes).unwrap();
        assert_eq!(loaded.display(a("B1")), "7");
    }
}
