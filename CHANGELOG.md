# Changelog

## Unreleased

- Rust/WASM spreadsheet core with a minimal cell store and two-argument
  `@SUM` formula evaluation.
- TypeScript shell wiring two input cells to the WASM core with a live
  recalculated sum.
- Rust build pipeline (`wasm-pack`) integrated into `npm run dev`/`npm run build`, replacing the earlier TypeScript/Node/WebSocket/SQLite plan.
