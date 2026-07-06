# Contributing to Web123

Thanks for your interest! Web123 is a clean-room, browser-based reimplementation of
Lotus 1-2-3 with real-time collaborative editing. Contributions are welcome — bug
reports, fixes, features, and documentation alike.

## Getting started

```bash
git clone https://github.com/DDecoene/Web123.git
cd Web123
nvm use            # Node 22 (see .nvmrc)
npm install
npm run dev        # Vite on http://localhost:5275 + WS server on :5276
```

Open `http://localhost:5275` — you'll be redirected to a fresh worksheet. Visit
`/demo` for a seeded feature-showcase worksheet.

## Running tests

```bash
npm test                # Vitest unit + integration tests
npx playwright test     # Playwright end-to-end browser tests
```

Both suites must pass before a PR can merge (CI enforces this).

## Branching model — GitFlow with milestone release branches

- **`main`** holds only released, tagged code. Never target `main` with feature PRs.
- **`release/vX.Y.Z`** is the integration branch for a milestone. All work scoped to
  that milestone lands here.
- **`feature/<name>`** branches off the open `release/vX.Y.Z` branch and PRs back
  into it.
- **`hotfix/vX.Y.(Z+1)`** branches off `main` for urgent fixes.

So: base your PR on the current open `release/vX.Y.Z` branch, not `main`.

## Definition of done

A PR is ready when:

1. It's based on the correct release branch (see above).
2. `npm test` and `npx playwright test` are green. **Every user-facing command or
   feature ships with a Playwright e2e case in the same PR** — unit coverage alone
   is not enough.
3. `CHANGELOG.md` has an entry under the milestone's version heading.
4. `README.md` command tables and the feature list reflect what was built.
5. Screenshots in `docs/screenshots/` are retaken if the UI changed.

## Clean-room policy (legal ground rules)

Web123 is an independent, unaffiliated reimplementation of the Lotus 1-2-3
interface for educational and preservation purposes (see the README's
"Trademark & affiliation" section). To keep it legally clean, every
contribution must follow these rules — they are hard requirements, and PRs
that violate them will be declined:

1. **Never copy or closely paraphrase text** from Lotus 1-2-3 manuals, help
   screens, error messages, or any other original Lotus documentation. All UI
   text, help text, and error messages must be written independently, in your
   own words.
2. **Never use disassembled or decompiled Lotus 1-2-3 binaries, or any original
   source code, as an implementation reference.** Acceptable sources are:
   publicly documented behavior (the slash-menu tree, @function signatures,
   keybindings, file-format descriptions), your own testing against `.wks`/`.wk1`
   files, and your own design decisions where the original behavior is ambiguous.
   If you don't know how a feature originally worked, say so in the issue or PR —
   don't fill the gap from anything resembling leaked or proprietary source.
3. **Frame file-format work as interoperability engineering** — reading and
   writing `.wks`/`.wk1` files based on their observed structure — never as
   reproducing Lotus's implementation.
4. **Don't imply affiliation or endorsement.** Nothing in code, docs, commit
   messages, or announcements may suggest the project is official, licensed,
   authorized by, or affiliated with any past or present Lotus 1-2-3 trademark
   holder.
5. **Keep license notices intact.** Source files under `server/` and `src/`
   carry an SPDX `AGPL-3.0-only` header — keep it when editing and add it to
   new source files.
6. If a feature can only be implemented correctly by consulting proprietary
   or leaked material, **stop and open an issue to discuss it** instead of
   proceeding.

## Filing issues

Use GitHub issues. For bugs, include the steps to reproduce, what you expected, and
what happened — a worksheet URL pattern and the exact keystrokes involved (the slash
command path or @function) help a lot for a keyboard-first application.

## License

By contributing you agree that your contributions are licensed under the
[AGPL-3.0-only](LICENSE) license that covers the project.
