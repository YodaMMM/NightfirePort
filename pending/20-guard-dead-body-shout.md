## Your task: `guard-dead-body-shout`, guards react when another guard shouts about a body or a surrender

`react_to_drone_alert_msg` in crates/nf-game/src/mp/bots.rs ports the SetState tail for alerts 0x15 and 0x13 ([M 0x1437a0:267-326]: the dead-body shout lands receivers in `NDrone2_DSTATE_SeenDeadBody` 0x16acd0 = state 0x9e, and the surrender shout from `Surrender_Anim` 0x1626a0:74 lands them in `NDrone2_DSTATE_SeenSurrenderedDrone` 0x16ade0 = state 0x9f), but only guard_ai_other4_live.rs calls it. The state bodies came in through done/20-vision-bit-unported-states.md. First find whether anything on master actually moves a single-player guard into 0x9e or 0x9f: who raises alert 0x15 and alert 0x13 in the game (read every caller), who does in the port, and whether the receivers' gates (props 0x29 and 0x4f, shouting range, `+0x4F8 & 0x800000`, the `+0x50C` share, the mode bytes) are checked on the live path. List in the report what the live path has and lacks, with addresses. If a live path already matches, say so and delete the unused copy. If not, wire the shared rule into the alert delivery, in the game's order, keeping the count of Rand draws the same (0x9f has its Rand(6) bark). List which levels have guards with those props [D]. Pin through real play: a guard finds a body and a nearby guard with the prop goes to the body's alert position, one out of range does not, and a surrendering guard's shout turns a nearby guard. Run guard_ai_other4_live, the ai binaries, walkreal_07000001_live, walkreal_07000009_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-dead-body-shout`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

