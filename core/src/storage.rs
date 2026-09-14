// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

//! IndexedDB persistence. Real on `wasm32` (via `rexie`); a no-op stub on
//! the host target so `cargo test` keeps compiling and running without a
//! browser.

#[cfg(target_arch = "wasm32")]
mod wasm_impl {
    use rexie::{ObjectStore, Rexie, TransactionMode};
    use wasm_bindgen::JsValue;

    const DB_NAME: &str = "web123";
    const STORE_NAME: &str = "documents";
    const DOC_KEY: &str = "web123-doc";

    async fn open() -> Option<Rexie> {
        Rexie::builder(DB_NAME)
            .version(1)
            .add_object_store(ObjectStore::new(STORE_NAME))
            .build()
            .await
            .ok()
    }

    pub async fn load() -> Option<Vec<u8>> {
        let db = open().await?;
        let tx = db.transaction(&[STORE_NAME], TransactionMode::ReadOnly).ok()?;
        let store = tx.store(STORE_NAME).ok()?;
        // `Store::get` takes the key by value and returns `Result<Option<JsValue>>`,
        // so the two `?`s peel off "transaction/lookup failed" and "no row" in turn.
        let value = store.get(JsValue::from_str(DOC_KEY)).await.ok()??;
        Some(js_sys::Uint8Array::new(&value).to_vec())
    }

    pub async fn save(bytes: &[u8]) {
        let Some(db) = open().await else { return };
        let Ok(tx) = db.transaction(&[STORE_NAME], TransactionMode::ReadWrite) else { return };
        let Ok(store) = tx.store(STORE_NAME) else { return };
        let array: JsValue = js_sys::Uint8Array::from(bytes).into();
        let key = JsValue::from_str(DOC_KEY);
        let _ = store.put(&array, Some(&key)).await;
        let _ = tx.done().await;
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native_stub {
    pub async fn load() -> Option<Vec<u8>> {
        None
    }

    pub async fn save(_bytes: &[u8]) {}
}

#[cfg(target_arch = "wasm32")]
pub use wasm_impl::{load, save};
#[cfg(not(target_arch = "wasm32"))]
pub use native_stub::{load, save};
