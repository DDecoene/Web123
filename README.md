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

TypeScript, Node, WebSockets, SQLite. Same recipe as the siblings.

## Want in?

Watch the repo, open an issue, or come argue about which @functions shipped in
Release 1A. If your fingers still remember `/WEY`, I want to hear from you.

## License

AGPL-3.0
