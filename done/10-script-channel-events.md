## Your task: `script-channel-events`, cut-scene streams that set switch channels

The port parses the script stream's `Cmd::Event` (op 0x12 [M 0x1d8548's stream walk]) but acts only on kind 0xB (weapon). `Script_EventHandler` 0x1d94c0's channel-write events - 0xD (set channel, params [ch, value]) and 0x11 (conditional set) - are not acted on [see the level-11-every-objective-2 report on master]. (1) Census every 0xD/0x11 event in every on-foot level's script streams and loadable packs [D]: level, script hash, stream, params. (2) Port both events exactly per 0x1d94c0 [M] (when in the stream they fire, skip-path behaviour - a skipped cut-scene must still write what the game writes on skip). (3) For each census row, a test that plays/skips that cut-scene on real input and asserts the channel the game writes, and what it unlocks (objective, door, trigger). Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/script-channel-events`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-29T12:10Z

---
## Report (done, 2026-10-29T12:39Z)

