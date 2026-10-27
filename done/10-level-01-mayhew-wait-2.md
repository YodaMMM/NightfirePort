## Your task: `level-01-mayhew-wait-2`, Mayhew waits at the door and the Double Cross exit still works

Start from `origin/s/level-01-mayhew-wait` (the escort holds at the kind-8 route's ally-wait node 33, flag 0x20, until the ambush beyond the door is cleared [M NDrone2_ReachedDestNode 0x153358:85-94]) and merge origin/master. On the full suite it turned two tests red: `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact` ("the AND-touch must fire with Bond AND the contact on the plate") and `fix_mission_live::the_07000001_exit_works_end_to_end`. Both walks reach the exit without clearing the ambush, so the contact now stays at node 33. Make the walks do what the game requires (clear the ambush the hold waits for, on real input) or show from the game's rule that the hold releases another way [M], keep the hold, and get both green plus escort_follow_07000001_live, mayhew_hold_live and fix_npc_doors_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-01-mayhew-wait-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-27T05:54Z

---
## Report (done, 2026-10-27T07:16Z)


---
## Report from the worker (done, 2026-10-27T07:17Z)

# level-01-mayhew-wait-2 — report

Task: after the ally-wait hold (the kind-8 route's ally-wait node 33, flag 0x20, release ch
0x99 [M NDrone2_ReachedDestNode 0x153358:85-94]) landed on `s/level-01-mayhew-wait`, two
full-suite tests went red: `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact`
and `fix_mission_live::the_07000001_exit_works_end_to_end`. Branch `s/level-01-mayhew-wait-2`
= origin/s/level-01-mayhew-wait (14769917) merged with origin/master (f8c4c1d1); merge was
clean, no sim code changed — the hold stays exactly as it landed.

## What the game requires, and what the walks now do

The hold releases only on ch 0x99: the three ambush guards wake on ch 0x0e (SP #276 via the
meet plate → the 0x33 mirror [M Trigger_Update 0x1b1a78 case 5]), die, and their death
channels 0x13/0x11/0x12 feed the AND mux (pl 318, out 0x99 [D placement]) — 0x21's UPDATE
(0x15efc8 case 3) then exits to Lead. The game's own recording holds the contact in 0x21 at
(2.94,7.22) for 1262 frames while the ambush runs [L tests/traces/01-doors.trace]: the PLAYER
walks ahead into the ambush and clears it. No alternate release rule exists in the decompile:
Drone+0x12C (the mission-wait channel) has no writer, so 0x99 is the only release [M]. The
walks were changed, not the hold.

Root cause of the red (from the failing run): the meet scene's weapon hand leaves Bond clip
32 spare 0, the hall dodge fights run dry (guard 6's fight hit "out of ammo" at +634 ticks),
and the resupply/approach nav from inside the hall wedges on the hall's south wall (the
STUCK dumps show segment_to_target ≈0.095 for every southward resupply hop and ≈0.47 for the
guard-11 approach; all gave up at the 3000-tick cap). The ambush stayed alive, ch 0x99 never
rose, the contact held at node 33, and the exit died with it.

## Fixes (test/harness only)

- `crates/nf-view/tests/common/walker.rs` — appended two shared ladders (additive only):
  `hall_restage` (meet-room corridor → east-door stage — the two proven legs) and
  `hall_ammo_ladder` (out the east door (7.9,5.3,11.1) [D placement] → the corridor boxes →
  back inside).
- `walkreal_07000001_live.rs` — a pre-hall top-up (< 48 rounds; a player tops up at the
  corridor boxes before the hall), a per-guard re-stage when the approach starts > 9 away
  (the guards hunt along the hall's south entry), and the per-guard dry ladder.
- `fix_mission_live.rs` — a new ambush phase before the escort ride: Bond walks ahead to the
  meet plate (the TOUCH_ONCE centre + quad sweep — Bond riding 3.9 behind a parked escort
  never crosses it), waits ch 0x0e, kills the three named guards through the real fire path
  (aim/trigger/reload inputs only; the friendly-fire shield stays on the contact), waits
  ch 0x99, then the existing ride loop runs unchanged.

## Verification

- `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact` — PASS:
  ch 0x32/0x33 at the meet plate, the hall AND fired (ch 153 on), the contact at the
  AND-touch, level_exit to 07000002 (71401 ticks; 14 self-recovering lenient wedges logged).
- `fix_mission_live::the_07000001_exit_works_end_to_end` — PASS: the ambush cleared (guards
  6/7/11 shot down through the real fire path), ch 0x99 on → the hold released (Mayhew 0x1d
  at (3.9,6.3,6.3)), Mayhew walked 121 u, ch10 fired, level_exit to 07000002.
- Must-stay-green: `mayhew_hold_live` PASS (5.7 s), `escort_follow_07000001_live` PASS
  (23.0 s), `fix_npc_doors_live` PASS (308.6 s).
- Full suite: the 4-crate runner could not complete in this sandbox — background processes
  are killed between tool calls (nohup and setsid both confirmed dead twice), and the
  foreground window budget went to the tests this task owns. What ran instead, all on the
  final tree (merge 165be89c + the fix commit):
  - `fix_mission_live` — 4/4 PASS (7.2 s), incl. `the_07000001_exit_works_end_to_end`.
  - `walkreal_07000001_live` — 3/3 PASS (61.1 s), incl. the fixed contact walk (61 s) and
    both sisters.
  - `mayhew_hold_live` PASS (5.7 s), `escort_follow_07000001_live` PASS (23.0 s),
    `fix_npc_doors_live` PASS (308.6 s) — the driver's three must-stay-green.
  - `cargo test --offline --release --no-run -p nf-view` — every nf-view test binary builds
    (0 errors), so the shared walker.rs append breaks no other test's compile. nf-game /
    nf-formats / nf-audio are untouched by this diff (test-only files in nf-view); their
    binaries were built earlier on this tree with the same command the suite uses.
  - LEFT for the next message (or the driver): `NF_ISO=<abs> nohup python3
    tools/test/suite.py > suite.out 2>&1 &` on this branch once a sandbox that keeps
    background processes is available; the per-binary failures to watch are the usual
    disc_golden three on the slim ISO.

Suite: the 5 driver-named tests + the 2 touched binaries' sisters all green; full-runner
total line: not obtainable in this sandbox (background jobs killed; see above).

## Notes for the driver

- The sandbox kills background processes between tool calls (nohup/setsid included —
  confirmed twice), so the suite runs in foreground windows; every named test was also run
  directly from its binary (commands above).
- Environment: rust stable via rustup; ALSA headers for nf-audio came from libasound2-dev
  extracted into a user prefix (PKG_CONFIG_PATH + a local .cargo/config.toml rustflags -L,
  both uncommitted sandbox setup, not task files).
- Nothing to render: no draw code changed.

## Live checks for the main agent

None required — no sim code changed. If you want belt-and-braces on the hold itself, the
existing pins cover it (mayhew_hold_live, the 01-doors.trace park).

## Usage

not visible to me
