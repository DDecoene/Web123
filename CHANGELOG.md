# Changelog

## Unreleased

- Full recursive-descent formula parser: arithmetic with precedence and
  parens, ranges, named ranges, and `@SUM`/`@AVG`/`@IF`/`@VLOOKUP`.
- Dependency graph with dirty-propagation recalculation, first-class
  error values, and circular-reference detection.
- Keyboard-driven 26×100 grid UI: READY/EDIT/POINT/GoTo mode indicator,
  `F2`/`F5`/`F9`, POINT-mode formula construction, and a minimal slash
  menu (`/Range Name`).
- Rust/WASM spreadsheet core with a minimal cell store and two-argument
  `@SUM` formula evaluation.
- TypeScript shell wiring two input cells to the WASM core with a live
  recalculated sum.
- Rust build pipeline (`wasm-pack`) integrated into `npm run dev`/`npm run build`, replacing the earlier TypeScript/Node/WebSocket/SQLite plan.
