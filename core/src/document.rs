// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use automerge::{transaction::Transactable, AutoCommit, ObjId, ObjType, ReadDoc, Value, ValueRef, ScalarValueRef};

use crate::engine::SpreadsheetCore;
use crate::model::CellAddr;

/// A fixed, empty document skeleton (root map with "cells" and
/// "named_ranges" child maps already created) that every new
/// `DocumentStore` loads from, rather than creating those maps fresh.
/// This gives every replica identical Automerge object identity for
/// both maps from birth, which is required for two independently-
/// created documents to converge correctly when merged — without this,
/// each replica's maps would be separate objects that conflict (and
/// one side's entire map becomes unreachable) on merge.
const SCHEMA_SEED: &[u8] = &[
    133, 111, 74, 131, 250, 67, 98, 205, 0, 124, 1, 16, 181, 236, 54, 249, 149, 224, 190, 204,
    92, 120, 216, 147, 75, 250, 226, 6, 1, 34, 237, 145, 221, 227, 155, 70, 223, 125, 187, 106,
    133, 160, 206, 23, 133, 209, 216, 30, 83, 44, 253, 87, 143, 181, 64, 184, 6, 21, 58, 21, 3,
    6, 1, 2, 3, 2, 19, 2, 35, 2, 64, 2, 86, 2, 7, 21, 20, 33, 2, 35, 2, 52, 1, 66, 2, 86, 2, 128,
    1, 2, 127, 0, 127, 1, 127, 2, 127, 0, 127, 0, 127, 7, 126, 5, 99, 101, 108, 108, 115, 12,
    110, 97, 109, 101, 100, 95, 114, 97, 110, 103, 101, 115, 2, 0, 2, 1, 2, 2, 0, 2, 0, 2, 0, 0,
];

pub struct DocumentStore {
    doc: AutoCommit,
    cells: ObjId,
    named_ranges: ObjId,
    core: SpreadsheetCore,
}

impl DocumentStore {
    pub fn new() -> Self {
        let mut doc = AutoCommit::load(SCHEMA_SEED).expect("schema seed is a valid Automerge document");
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

    // kept for a future P2P sync plan to call directly
    #[allow(dead_code)]
    pub fn save_bytes(&mut self) -> Vec<u8> {
        self.doc.save()
    }

    /// Boots a store from whatever's in IndexedDB, or an empty document if
    /// there's nothing there yet (first run, or storage unavailable).
    pub async fn load_from_storage() -> Self {
        match crate::storage::load().await {
            Some(bytes) => match DocumentStore::load_bytes(&bytes) {
                Ok(store) => store,
                Err(_e) => {
                    #[cfg(target_arch = "wasm32")]
                    web_sys::console::error_1(
                        &"Web123: stored document was corrupt or unreadable; starting a fresh document. The unreadable bytes were preserved under a backup key.".into(),
                    );
                    crate::storage::save_corrupt_backup(&bytes).await;
                    DocumentStore::new()
                }
            },
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
    ///
    /// Re-resolves `cells`/`named_ranges` first so this is self-sufficient
    /// regardless of which caller invoked it (`load_bytes`, `fork`, or
    /// `merge`) — the cached ObjIds may be stale after loading a doc from
    /// somewhere else.
    fn replay_all(&mut self) {
        self.cells = get_or_create_map(&mut self.doc, "cells");
        self.named_ranges = get_or_create_map(&mut self.doc, "named_ranges");
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

// kept for a future P2P sync plan to call directly
#[allow(dead_code)]
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
            cells: resolve_map(&doc, "cells"),
            named_ranges: resolve_map(&doc, "named_ranges"),
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
    pub fn merge(&mut self, other: &mut DocumentStore) -> Result<(), automerge::AutomergeError> {
        self.doc.merge(&mut other.doc)?;
        self.replay_all();
        Ok(())
    }
}

/// Looks up a map that's expected to already exist (on a forked or loaded
/// doc) — panics rather than creating one, since a missing map there means
/// the doc's schema is broken, not that it's the first time we've seen it.
fn resolve_map(doc: &AutoCommit, key: &str) -> ObjId {
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

        a_store.merge(&mut b_store).unwrap();

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

        a_store.merge(&mut b_store).unwrap();
        b_store.merge(&mut a_store).unwrap();

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

        a_store.merge(&mut b_store).unwrap();
        b_store.merge(&mut a_store).unwrap();

        // Automerge's default conflict resolution (last-writer-wins by
        // change ordering) must pick the *same* value on every replica —
        // that's the property under test, not which specific value wins.
        assert_eq!(a_store.raw_input(a("A1")), b_store.raw_input(a("A1")));
    }

    #[test]
    fn independently_created_documents_converge_on_merge() {
        // Unlike the tests above, these two stores are NOT forked from one
        // another — each comes from its own `DocumentStore::new()` call, the
        // way two users who each open the app fresh (empty IndexedDB) would
        // end up with two independently-created documents. Before the
        // schema-seed fix, each store's "cells"/"named_ranges" maps were
        // distinct Automerge objects, so merging them would silently drop
        // one side's entire cell map instead of converging.
        let mut a_store = DocumentStore::new();
        let mut b_store = DocumentStore::new();

        a_store.set_cell("A1", "from-a");
        b_store.set_cell("B1", "from-b");

        a_store.merge(&mut b_store).unwrap();
        b_store.merge(&mut a_store).unwrap();

        assert_eq!(a_store.raw_input(a("A1")), "from-a");
        assert_eq!(a_store.raw_input(a("B1")), "from-b");
        assert_eq!(b_store.raw_input(a("A1")), "from-a");
        assert_eq!(b_store.raw_input(a("B1")), "from-b");
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

    #[test]
    fn load_bytes_rejects_corrupt_data() {
        assert!(DocumentStore::load_bytes(b"not a valid automerge document").is_err());
    }
}
