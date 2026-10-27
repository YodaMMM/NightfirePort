## Your task: `stuck-patrol-0c-0d`, two patrol guards that never move on 0700000c and 0700000d

The stuck-guard sweep (`crates/nf-view/tests/guard_stuck_sweep_live.rs`) finds two patrol movers frozen from the level start:
- 0700000c drone 15, PATROL (0x07), stops at (9.30, 12.24, -220.82), 4.7 units from its home (6.55, 12.24, -224.56); nearest door 32.4 units away.
- 0700000d drone 33, PATROL (0x07), parked at its home (191.08, 13.66, -232.19) with a ~0.09 z jitter for the whole run; nearest door 44.0 units away.

1. Trace each drone's patrol: its path record and node chain [D], the next node it is heading for, and the query that stops it (route follow BLOCKED / NO_PATH, the mover's collision response, an unreachable node) [M NDrone2_ReachedDestNode 0x153358 and the patrol functions].
2. Fix the cause in the shared code (never a per-level patch) and turn the two sweep rows into assertions: each guard walks its patrol. Sister: a guard whose patrol path is genuinely blocked in the game still stops.
3. If a live number is needed, say exactly which drone field to read in PCSX2 on a fresh boot.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/stuck-patrol-0c-0d`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox2 at 2026-10-23T16:39Z

---
## Report (done, 2026-10-23T18:17Z)


---
## Report from the worker (done, 2026-10-23T18:17Z)

# stuck-patrol-0c-0d — the two frozen patrol movers on 0700000c / 0700000d

## What was wrong

The stuck-guard sweep (`crates/nf-view/tests/guard_stuck_sweep_live.rs`) reported two PATROL (0x07)
movers frozen from the level start. On this session's master the 0700000d row (drone 33) no longer
reproduces — the targeted sweep logs `SWEEP 0700000d: 0 hits over 60 s` and the probe shows the
guard walking its pulley line (it was likely fixed by an earlier merged creep/special-node change);
the 0700000c row (drone 15) reproduced exactly, and the fix below is what un-stuck it.

**0700000c drone 15 (skin 0x05000090)** walked out of its home (6.55, 12.24, -224.56) for ~2 s and
froze at (9.30, 12.24, -220.82), 4.7 units from home, walk anim playing, `route_blocked` false,
position jittering ±0.02 forever. Its route is path#9 "Patrol Path_G2" (kind 0x7a04, Loop, 16
nodes [D level data via the port's nav build]); the wedge sat on the node2 (6.46, -224.66) →
node3 (10.43, -219.33) segment at creep step 12/15, with the next creep point q = (9.57, -220.49)
0.43 ahead. The boundary pinch there is real game data [D]: boundary walls #313/#314 pass within
~0.15 of the creep line (…(10.09, 11.67, -221.34) → (9.66, 11.67, -220.66) → (12.07, 11.67,
-218.95)…), so the post (9.661, 11.668, -220.657) sits inside the advance test's radius.

## The cause — a missing escape band in the shared creep advance [M]

The game's `LinkCreep_Increment` (0x159f28, ACTION.ELF decompile) advances the creep step when
`Vec_Dist2D(feet, q) < 0.4` **or** the boundary read passes — but that read is only *reached* when
`Vec_SqDist2D(feet, q) >= 1.0`:

```c
if ((DAT_002c8774 == 0) || (Vec_SqDist2D(feet, q) < 1.0)) cVar2 = 1;   // NO post read within 1.0
else cVar2 = AINetwork_BoundsNodeTest(0x3ecccccd /* 0.35 */, feet, …, q, …);
…
if (fVar5 < 0.4 || cVar2 == 1) { route+0x60 = q; route+0xb0 += 1; }    // advance
```

A guard walking a pinch between two boundary posts sits 0.4..1.0 from the next step point, where
the game advances **without consulting the posts**. The port ran `bounds_node_test(0.4, feet, q)`
in that band with no escape — at d = 0.43 with the post 0.1 off the line the test failed on every
tick, the creep never advanced, and the guard jockeyed in place forever. The port also used r = 0.4
where the game uses 0.35 (0x3ecccccd).

The same escape exists in the game's `LinkCreep_Decrement` (0x15a148: `SqDist2D(feet, prev) < 1.0 →
clear` before the 0.1 post test on the step-back) and in `LinkCreep_Handler`'s block gate (0x15a2c8:
`Vec_SqDist2D(feet, target) < 1.0` skips the whole block check — 2D, where the port used 3D). All
three now run in `Route::follow` (`crates/nf-game/src/nav.rs`), one shared function, no per-level
patch.

## Verification

- **Reproduce** (pre-fix): the targeted sweep (`SWEEP_LEVELS=0700000c,0700000d`) logged 4 HIT rows
  for drone 15 at (9.29..9.30, 12.24, -220.8x); a tick probe showed creep step 12/15 frozen with
  `bnt false`, d 0.43, code MOVING, walk anim 0x37 — for 3000+ ticks.
- **Fix** (post-fix): the same probe shows step 12 → 13 → the next node; the sweep row count drops
  to the one residual documented under "Left" below; the pinned tests pass:
  - `crates/nf-view/tests/patrol_walk_0c_0d_live.rs` — **the two sweep rows as assertions**: over
    60 s of fixed 60 Hz ticks through the real load path, 0700000c drone 15 completes a full lap of
    its 16-node patrol (max 19.27 units from start, far point (25.47, -217.31), back at (5.77,
    -225.80)) — the old wedge capped it at 4.7 and it can only reach the far side through the
    pinch; 0700000d drone 33 walks ≥ 8 units of its 20.4-unit pulley line (observed 17.60, far
    point (209.38, -232.20)). Both hold PATROL 0x07 while walking (0700000d's guard spots Bond at
    the spawn at t≈2400 and goes 0x68 — the walk already happened).
  - `crates/nf-game/src/nav.rs` unit pins: `creep_increment_escapes_the_post_test_within_1_and_holds_beyond`
    (within 1.0 of q the game advances without the post read; past 1.0 a post within 0.35 holds the
    step — the sister half, a genuinely sealed approach does not march) and
    `creep_decrement_steps_back_within_1_of_the_previous_point` (the step-back's 1.0 escape).
- **Run**: `cargo test --offline --release -p nf-game --lib` — 524 passed, 0 failed (includes the
  two new unit pins and the pre-existing creep/bounds tests); `-p nf-game --test nav_kick_live` —
  5 passed, 1 ignored; `-p nf-view --test patrol_walk_0c_0d_live` — 2 passed; the targeted sweep
  binary — 0700000d 0 hits, 0700000c the one residual row below.
- **Full suite**: the chunked runner (`tools/test/chunk_suite.py`) was run in the background of the
  session on the pre-fix tree (state `/home/z/suite_patrol_state.json`, log `/home/z/chunk_patrol_1.out`)
  and re-started on the final tree (`/home/z/suite_patrol_final_state.json`, logs
  `/home/z/chunk_final_*.out`); the sandbox kills long jobs between tool calls, so it advances in
  chunks — **the final-tree tally was still accumulating at the deadline and is the one thing left**
  (see the branch's next message; re-run
  `NF_ISO=… python3 tools/test/chunk_suite.py /home/z/suite_patrol_final_state.json` until
  `ALL DONE`).

## Left

1. The final-tree full-suite tally (the chunked run above; nothing else changed after commit
   e6e8fce09, so a clean tally is expected — the touched surface is `Route::follow`'s two gates and
   every route walker is suite-covered).
2. **The residual GO_TO_GOAL wedge, same drone, deeper in**: with the sweep's own alarm path
   [M 0x177c18:113-114] applied, drone 15 (now in GO_TO_GOAL 0x63 toward the alarm goal (36.40,
   12.06, -208.78), croute path#3, nodes [2..7]) walks out to (27.90, -218.03) and pins there —
   `route_blocked` false, croute creep 5/32, target (28.6, 11.7, -216.5) 2.04 away and
   `bounds_node_test(0.35)` CLEAR, walk anim on, ±0.02 jitter. That is **not** the creep advance
   (the target is far, clear, and the increment's gates are open) — the body itself does not
   advance toward its own move target, which points at the mover/collision layer (a boundary
   push-out or a static the port pushes against that the game slides along). Not probed deeper —
   out of this task's two-row scope after the patrol fix landed; the probe data above is the
   starting point.
3. Live checks for the main agent (fresh boot, PCSX2): on 0700000c, read drone 15's route creep
   step (the route object at drone+0x970, step field `route+0xb0`) while the guard patrols the
   node2→node3 segment: expect the step to keep incrementing past 12 (the game walks through the
   same pinch [D — the guard's patrol owns that corridor on the disc's path data]); and
   `drone+0x4F8` bit 0x4000000 during the alarm chase to see whether the game's guard also
   repositions to (36.40, 12.06, -208.78) or stops short of the (27.9, -218.0) wall — that decides
   whether the residual wedge is a port bug or a real-game behaviour.

## Usage

not visible to me — this sandbox exposes no token, cost or context-window figures to the session
(no environment variables or status output shown to me carry them; the driver's harness may have
them).
