#[cfg(target_os = "solana")]
pub use solana_sdk_macro::wasm_bindgen_stub as wasm_bindgen;
/// Re-export of [wasm-bindgen].
///
/// [wasm-bindgen]: https://rustwasm.github.io/docs/wasm-bindgen/
#[cfg(not(target_os = "solana"))]
pub use wasm_bindgen::prelude::wasm_bindgen;
