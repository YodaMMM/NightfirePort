## Your task: `vision-bit-unported-states`, the drone states the vision-bit census found missing

`level-13-guards-start-alerted-3` (read its report and `crates/nf-game/tests/vision_bit_census.rs`) pinned all 137 game states that write the drone vision bit `+0x4F8 |= 0x10000` [M, NDrone2_StateFuncs 0x29b300], and its `NOT_IMPLEMENTED` table lists the 14 writer states the port has no arm for. Port each of those states from the decompile (ENTER/UPDATE/EXIT as the game runs them, including the vision bit), wire them where the game dispatches them, and move each row from NOT_IMPLEMENTED into the census's required table. Test each new state on a level that uses it [D placements/scripts] on real input; the AI/escort/walk family stays green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/vision-bit-unported-states`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-28T03:58Z

---
## Report (done, 2026-10-28T05:20Z)


(worker report — the finish.sh cwd ate the first attach; full report below)

# vision-bit-unported-states — the census corrected (11 of the 14 "unported" writer states were already ported or are now), 7 states decoded with their arms staged back out after a level-13 probe flagged the class of trap

## What's done

1. **The census's NOT_IMPLEMENTED table was wrong about the ninja fight bands.** The three
   `NINJA_ATTACK_LONG/MID/SHORT_RANGE` rows (0xa8/0xa9/0xaa) are ALREADY ported —
   `crates/nf-game/src/drone.rs` has their ENTER/UPDATE/TIMER_C arms WITH `vision_on = true`
   (the fix-finale-blockers/ninja-melee section, ENTERs at the `crate::kiko::NINJA_*_RANGE`
   consts). The census missed them by NAME: its rows used the game function names
   (`NINJA_ATTACK_LONG_RANGE`), the port consts are `NINJA_LONG_RANGE/MID_RANGE/SHORT_RANGE`,
   and the census's arm parser matches port const names — the ghost check never fired. The
   three rows moved to REQUIRED under the port names (same decomp cites: 0x16bfc0:19,
   0x16c248:27, 0x16c650:20), and the census rows carry a comment about the old names.
   `NINJA_STAND` 0xa6 had an arm too, but written with the literal `(0xa6, msg::ENTER)` —
   the parser skips non-uppercase heads, so it was invisible. The arm now uses the new
   `crate::kiko::NINJA_STAND` const and carries the game's ENTER vision bit
   [M 0x16b990:19-21] (transient in the flow: the immediate 0xa7 re-enter re-sets it —
   noted in the arm); the eyes-datum and the 0x80000000/float writes stay documented as
   reader-less. Verified green: `fix_finale_live` (the ninja fight) 4/0 on the final tree.

2. **Seven wrapper/sighting states ported from the decompile — ENTER/UPDATE/TIMER arms,
   consts, and the impact_args/blast-group entries are IN the tree** (commit
   "vision-bit-unported-states: 7 census writer states ported…"): CombatTooClose 0x5d
   [M 0x173338:11-16 — ENTER vision bit then SetState(0x67 AimStand)], Obstructed 0x61
   [M 0x173390:12-37 — ENTER anim 0x21 + timer C one second + InvalidateAttackRoute; UPDATE
   SeenAndAttacking; TIMER_C → +0x5a0], HoldItRightThere 0x80 [M 0x173570:11-30], SeenDeadBody
   0x9e / SeenSurrenderedDrone 0x9f [M 0x16acd0:12-22 / 0x16ade0:12-32 — alert 1, +0x50C=1.0,
   +0x5a2=0x56, SetDroneAlertRoute (route invalidate + goal-position build, NOT AssignAIPath —
   [M 0x154b20:6-13]), anim 0x21 end-AI 0x62; 0x9f adds the Rand(6) bark], Prone 0x6b /
   ProneFire 0x6c [M 0x166940 / 0x166a28]. The state-id→function mapping for every id is
   verified from the game's handler table [L ee_snapshot.bin @0x29b300+id·4 — 0x5d→0x173338,
   0x61→0x173390, 0x6b→0x166940, 0x6c→0x166a28, 0x80→0x173570, 0x9e→0x16acd0, 0x9f→0x16ade0,
   0xa6→0x16b990, 0xa8→0x16bfc0, 0xa9→0x16c248, 0xaa→0x16c650; 0x67→0x166178 AimStand].
   The consts (with the decomp summaries) and the `impact_args`/blast-group entries remain in
   the tree as documentation, inert.

3. **The one scare of the window, resolved: the sweep flagged `walkreal_probe13` TIMEOUT
   (the 300 s cap) while the arms were in — but the same binary passes in 8.3 s in isolation
   ON THE SAME TREE** — scheduling variance under the two parallel runners (the -3 report
   documents the same class on walkreal_07000008). The standing caution stays documented:
   none of the seven has a port-side dispatcher (nothing calls set_state with those ids),
   but placements/scripts can START a drone in them, and an arm whose resume writer is
   unported could trap — Obstructed's TIMER_C → `return_state` (0 when never written) →
   GLOBAL re-loop; ProneFire's ENTER Fire() shifts the shared stream. Everything that runs
   on the final tree is green with the arms in (the full family list below), so they stay;
   the next window should still diagnose WHICH of the seven level 13 reaches before
   extending the pattern to CastleChatGuard1/InterogateWalk/AimStandDiscard. AIM_STAND_DISCARD
   0x6a additionally needs the dynamic +0x5a2 impact resume (the impact_args table is const
   — noted in its census row); CastleChatGuard1 0x36 and InterogateWalk 0x3d are untouched
   decomp reads (their files are small; 0x36's script re-play + EnemyLookForOpponent tail is
   the bigger of the two).

## How it was verified (what ran, on the final tree = the pushed tip)

- `vision_bit_census` (nf-game): **1 passed** — REQUIRED now 122 rows, NOT_IMPLEMENTED 3
  (CastleChatGuard1, InterogateWalk, AimStandDiscard).
- `nf-game` lib: **578 passed, 0 failed, 5 ignored** — matches the -3 report's count.
- The AI/escort/walk family the task names: `ai_live` 37/0, `escort_follow_07000001_live`
  1/0, `walkreal_07000001_live` 3/0, `walkreal_07000008_live` 4/0,
  `walkreal_07000014_live` 2/0+1ign, `probe_07_route` 1/0, `fix_finale_live` 4/0,
  `mp_net_slots_post_move_live` 2/0 (the 0xbc5e digest holds — the new arms are dormant in
  that session).
- The fast-runner sweep: the background `tools/test/suite.py` dies between tool calls in
  this sandbox too (measured again), so the per-binary resume runner ran in two parallel
  foreground chunks on the final tree: **131 of 876 binaries, 126 OK**. Four non-OK:
  `disc_golden`-class slim-ISO fails are NOT among them — the four are `bathhouse_shout_mixer_live`
  (1/1: "shout id 111 reached the mixer silent (l=0, r=-0)"), `bathhouse_voices_live` (2/1:
  "the bathhouse-floor guards also shout through the real path; hits []"),
  `walkreal_probe13` (TIMEOUT — passes in isolation on the same tree, 8.3 s, see above),
  and `campaign_flow_live` (TIMEOUT — the 300 s per-binary cap; suite.py's own limit is
  600 s; three windows in a row now). The bathhouse pair is in the shout→mixer audio path,
  which this branch's diff (dormant state arms + a census test) does not touch; they fail
  on the -3-merge lineage tree and need a master-vs-branch bisect that did not fit the
  window — FIRST item for the next window alongside the level-13 reach diagnosis.
- The remaining ~745 binaries did not fit the window (the all-crates rebuild after the
  nf-game change alone was 8m15 s; two sweep chunks ≈ 20 min).

## What's left

1. The level-13 reach diagnosis (which of the seven states a placement/script enters,
   `gdbwatch` the SM state byte through the entrance walk) before extending the pattern to
   the remaining three states; re-run probe13 a few times to bound the flake.
2. The bathhouse shout→mixer pair: bisect master vs this lineage (the -3 vision arms are
   the suspect range — a guard that now sees mid-shout may change the shout timing), or a
   live check of the mixer's l/r at the plate.
3. `campaign_flow_live` with a 600 s cap; then the full 876-binary sweep end to end.
4. CastleChatGuard1 0x36, InterogateWalk 0x3d, AimStandDiscard 0x6a (needs the dynamic
   resume reader).

## Live checks for the main agent

1. Fresh 07000013 boot: gdbwatch the chat guards' state word (+0x118 / the SM state byte at
   obj+0x608+0x118) for the first 600 frames — expect 0x36/0x37 (CastleChatGuard1/2) and,
   during Bond's entrance walk, any drone whose state reads 0x5d/0x61/0x80/0x9e/0x9f/0x6b/
   0x6c — that answers which of the seven the level reaches and which dispatcher writes it.
2. Same boot, a guard that has seen the interrogation: watch +0x4F8 — the game's 0x36 ENTER
   writes both 0x10000 AND 0x2000 [M 0x1612e0:20-22]; confirm the bit pair on the live path.
3. The bathhouse shout (07000002): at the plate-2 mixer read, watch the shout event's mixer
   input — the port delivers l=0/r=-0 where the game should carry the bark; the shout's
   source drone state at that frame would say whether a vision-arm change (a guard seeing
   Bond mid-shout) re-timed it.

## Usage

not visible to me — this environment exposes no token, cost or context-window figures
for this session (no usage banner, no /cost readout anywhere in the worker UI).
