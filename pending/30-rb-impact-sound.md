## Your task: `rb-impact-sound`, rigid props make their impact sound and carry their light

docs/reports/unread-params-props.md on origin/s/unread-params-props says the 0xD8 rigid body params p3 (carry-light) and p4 (impact sound) are unread and ride the body evolution. crates/nf-game/src/rigid_body.rs now has a section of ported evolution functions, while its header still calls the evolution a tracked gap, so first find what really runs on master: read `RB_Update` 0x1d6e38 and `RB_RestingContact` 0x1d7168, compare them with the port, and correct the header. Read what `RB_Create` 0x1d6bf0 does with p3 and p4, then which reader plays the sound (when, how often, and at what speed gate) and how the light follows the body. Census both params per level [D]. Port what is missing, with sounds through the normal sound path and no change to the count of Rand draws. A light on a moving prop is a visual, so list frames for a render check under the report's live checks for the main agent. Pin through real fire on a level from the census: a shot prop that hits the floor plays its sound once per the game's rule, and an undisturbed one stays silent. Run the rigid body binaries (grep tests for `rigid_body`), walkreal for the level you pin, and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/rb-impact-sound`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

