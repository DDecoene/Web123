<!-- GitFlow: base this PR on the milestone's release/vX.Y.Z branch, NOT main. -->

## What & why

<!-- Short description of the change and the motivation. -->

Closes #

## Definition of done

- [ ] Branched off the milestone's `release/vX.Y.Z`; PR is based on that release branch (not `main`); issue assigned to the matching milestone
- [ ] `npm test` (vitest) green
- [ ] `npx playwright test` (e2e) green — every user-facing command/feature has a Playwright case
- [ ] `package.json` version matches the milestone
- [ ] `CHANGELOG.md` updated under the milestone heading
- [ ] `README.md` reflects what was built
- [ ] `CLAUDE.md` (architecture / command tables / roadmap) updated
- [ ] Screenshots retaken if the UI changed

<!-- Do NOT add Co-Authored-By: Claude (or any AI attribution) to commits or this PR body. -->
