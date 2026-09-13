# Web123

Lotus 1-2-3, rebuilt clean-room for the browser — with the one thing 1984 couldn't
do: multiple people in the same worksheet at the same time.

This is the third of a trinity. [WebWordStar](https://github.com/DDecoene/WebWordStar)
brought back the word processor. [WebBaseIII](https://github.com/DDecoene/WebBaseIII)
brought back the database. In 1984 the office ran on those two plus a spreadsheet,
and this repo is that spreadsheet.

## The idea

Not emulation. A reimplementation of the 1-2-3 experience, interface kept sacred:

- The slash menu. Press `/` and get Worksheet, Range, Copy, Move, File, Print,
  Graph, Data, Quit — the menu tree that taught a generation what menus were.
- @functions: `@SUM`, `@IF`, `@AVG`, `@VLOOKUP` and friends.
- Keyboard-only cell navigation. F2 to edit, F5 to go to, F9 to recalc.
- The mode indicator in the corner: READY, EDIT, POINT, MENU.

And the modern part, same as the siblings: real-time collaboration. Several people
in one worksheet, peer cursors with name tags, edits that converge, undo scoped to
your own changes.

Later, when the trinity becomes a suite: period-authentic interchange. WebBaseIII
does `COPY TO report TYPE WKS`, Web123 opens it. Files, not APIs.

## Status

Design phase. Nothing to run yet.

## Stack (planned)

Rust, compiled to WebAssembly, for the entire spreadsheet core: formula
engine, dependency graph and recalculation, the CRDT document (via
Automerge), WebRTC networking, and WKS-style file interchange. TypeScript is
a thin rendering and input shell around it — no spreadsheet logic lives in
JavaScript.

Collaboration is peer-to-peer and local-first: documents sync directly
between browsers over WebRTC, merge cleanly when peers who edited offline
reconnect, and don't depend on any always-on server to function. The only
outside dependency is a public, WebTorrent-style signaling tracker used
solely to establish the initial peer connection — it never sees document
content. An on-prem server (the original vision) remains optional, useful
for durability, but never required.

This departs from the trinity's original shared TypeScript/Node/WebSockets/
SQLite recipe — Web123 is the odd one out, trading the shared backend model
for the "no infrastructure to run" goal.

## Want in?

Watch the repo, open an issue, or come argue about which @functions shipped in
Release 1A. If your fingers still remember `/WEY`, I want to hear from you.

## Trademark & affiliation

Web123 is an independent, clean-room reimplementation built for educational and
software-preservation purposes, in the same spirit as its siblings
[WebWordStar](https://github.com/DDecoene/WebWordStar) and
[WebBaseIII](https://github.com/DDecoene/WebBaseIII). It is **not** affiliated with,
endorsed by, or connected to Lotus Development Corporation, IBM, HCL, or any past or
present holder of the "Lotus 1-2-3" trademark. The name "Lotus 1-2-3" appears here
only to describe the historical program whose interface this project reimplements. No
original Lotus 1-2-3 code or documentation text is used: behavior is implemented from
publicly documented interfaces and independent testing against files.

## License

[AGPL-3.0-only](LICENSE). Source files carry SPDX license headers.
