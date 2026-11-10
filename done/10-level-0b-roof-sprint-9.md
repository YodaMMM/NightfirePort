## Your task: `level-0b-roof-sprint-9`, the 0700000b roof sprint can be walked on 00 Agent with real health

Branch from origin/s/level-0b-roof-sprint-8; if origin/s/los-mask-repairs has moved past it, merge that too; then merge current origin/master. Read docs/reports/level-0b-roof-sprint-8.md and the 00 Agent ignore message in walkreal_0700000b_live. Sprint-8 reproduced the crosshair rocket (the #6/#7 pair dies to one round, three runs) and mapped the traps (#4 steals the lock, the stand bleed). Take its next levers in order. Real input only, no top-ups, never lower damage or health; helpers gated to 0700000b and difficulty >= 3. Remove the ignore when green, otherwise push the ledger. Run walkreal_0700000b_live and interactables_0700000b_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-0b-roof-sprint-9`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-11-10T11:03Z

---
## Report (done, 2026-11-10T12:38Z)


---
## Report from the worker (done, 2026-11-10T12:38Z)

# level-0b-roof-sprint-9 — report

Branch `s/10-level-0b-roof-sprint-9` = `origin/s/level-0b-roof-sprint-8` (8aeba7c3e) merged
with `origin/s/los-mask-repairs` (c4ef1ea84 — it HAD moved past sprint-8; two comment-only
conflicts in `walkreal_0700000b_live.rs` resolved to the los-mask-repairs wording) merged
with `origin/master` (4bdaf3f50; clean). **Branch-name note:** the driver header names
`s/level-0b-roof-sprint-9` but neither that nor `s/10-level-0b-roof-sprint-9` existed on
origin when the branch was cut (checked with `git ls-remote`); the task paragraph's
`s/10-level-0b-roof-sprint-9` (matching the finish.sh id) was used.

**Status: the 00 Agent walk is NOT green — the ignore stays and the sprint-9 ledger rides
it — but two of sprint-8's levers LANDED and the whole fight re-measured on the merged
sight tree:** the early r1 CHAIN-KILLS #7+#4 in one deterministic round (eight runs), the
take FIRED once (t16784 [L a9_run3] — the first 00 Agent take ever observed in these runs),
and Bond died at t16592 on the P2→P4 run with the band's #6/#8/#9 alive and no clear-ray
round left. `walkreal_0700000b_live`: **6 passed / 1 failed / 1 ignored** (the failure is a
merge-inherited d2 regression, below). `interactables_0700000b_live`: **14 passed /
0 failed**. No `crates/*/src/` changes — everything is in the 0b test file, d3-gated. No
damage or health lowered, no top-ups, real input only.

## What's done (the levers, in the task's order)

1. **Lever 1 (the distance-gated dart close) implemented, measured, and superseded.** The
   close now engages the first tick `d4 < 9.0` at any phase (d3 only; the d2 phase-2 window
   is byte-exact). On this tree it NEVER engages before the r1 kills #4 (d4 stays ≥ 11
   through P1→P2 [L a9_run1]) — and it became moot: the early r1 (lever 3) kills #4
   outright, which the sprint-8 ledger held out of reach ("the corpse state"). The dart
   sleep alone was correctly predicted insufficient [M Bullet_heatseekable 0x12f248 — only
   health ≤ 0 exits the seekable scan]; the rocket corpse is what actually exits it.
2. **Lever 3 (the r1 timing) is the big win.** The r1 fires mid-approach (phase 0, z > 6 —
   past the SW structure band that ate the a00_run3 deck-key shot) instead of waiting for
   the P1 radius. MEASURED (a9_run2, reproduced across eight runs): the spawn ray rides the
   band anchor; the nearest-SEEKABLE lock takes #7 (Δ 0.27 rad [M Bullet_homing 0x12f358]),
   the re-scan takes #4, and the blast detonates ON #4's own column spot (2.05, 58.07,
   30.65 [L the flight trace]) — **#4 AND #7 die point-blank with Bond ~19 u away, zero
   self-hurt** (the sprint-8 r1's −7.5..−11.7 self-hurts are GONE — the blast lands long
   before Bond's path reaches it). #4's 0x18 toll collapsed from 49.0 (a9_run1, the
   un-levered baseline on this tree) to 21.0 in 3 hits (his pre-death shots).
3. **Lever 2 (the clear band ray) mapped the hard way.** Every round fired north of z 20
   dies on the gap-mouth structures: r2/r3 from the P2 crossing at 8-12 u [L a9_run3/6];
   a z > 29 attempt hit a SECOND unmapped piece [L a9_run7]; an in-zone r4 detonated at
   5.8 u and paid Bond −28.05 = the death [L a9_run6 frame 16650]. The only PROVEN clear
   corridor is the south strip z < 18.5 (the r1 crossed 28 u from there) — it fits exactly
   one more round before Bond crosses z 18.5 (~t16400), and that round is the one #5 steals
   (below). The 16-ray probe (the sprint-8 next-lever 2) is ledgered for the next session
   with the concrete positions to probe (the P4 approach z 35-41 and the late strip).
4. **The #5 lock-thief pinned.** With #4 dead, the drift #5 inherits the thief role: alive
   at every r2 launch, nearest-in-cone from the whole south strip (d 6-15 vs #6's 24), and
   the r2 chased him NE into the unmapped piece at (2.41, 57.95, 21.24) — dying at 12.7 u —
   but the blast DID pay him −30 (hp −30 [L a9_run8]): the r2's steal is also a kill. The
   cleanup round for #6/#8/#9 (hp 10/10/20, sitting 2.8-5.9 u apart — one blast pays all
   three [M Explode_Propagate 0x17c518 falloff; L the sprint-8 r3 payments −28.75 at 5.7 u,
   −63.75 at 2.9 u]) is the ledgered lever.
5. **The post-take slam law pinned.** With #4 a corpse the close's else-branch must jump
   phase 2→3 and NOT slam phase 4 back: the ≥ 2 slam thrashed the P5 drive after the take
   (phase 3↔4 flicker [L a9_run3 t16785-t16815]) while the P2-only variant routes Bond
   through #5's strafe spot and the r2 chase-blast paid −35 (world 58.0, death t16556 [L
   a9_run5/6]). The shipped code keeps the ≥ 2 jump (the measured-safe P2→P4 straight, ≥
   11 u of #5's blast spot) — the thrash is now harmless because the rounds stop at the
   all-dead check and the drive composes (slam → the ch 0x3F advance → phase 4 for the aim).
   A close_start reset I trialled (re-arming the close) was REVERTED: it held the d2 lane
   against a merged-tree #4 that never darts [L a9_run10 — the d2 death at d 1.3].
6. **The take-slot aim shipped (d3 only).** The P4-proper LOS (1.88, 40.81) did not open in
   5 held ticks [L a9_run3]; the walked slot (pp−0.3, pp−1.0 — the d1-proven take spot) is
   what the walk now aims and stands on. The a9_run3 take fired from (3.14, 40.88) after
   the thrash dragged Bond there — the slot aim makes that deterministic.

## The merge-inherited d2 regression (the binary's 1 failure)

`walk_fresh_spawn_to_the_mission_end_real_damage_agent` (the d2 sister — green on
sprint-8's tree) now dies at t16525 at (0.87, 58.10, 31.68), **d 1.3 of #4**, with #4
dealing 28.0 in 5 hits (weapon 0x1 — he has SWITCHED weapons on this tree) [L a9_run9/10].
On the merged los-mask-repairs sight tree #4 parks at (2.1, 58.0, 31.7) — ON the d2 dash
line (the P3→P4 straight passes within 0.5 u) — where the pre-merge tree had him coming
DOWN the gap column to meet the lane (d 14.8 → 7.8 → 5.0 [L the sprint-5 agent_run2
trace]). My edits are d3-gated and were audited line-by-line for d2 equivalence (the close
window, the timeout, the else-branch, the switch-back, the band block); the take-slot aim
was the one leak found and gated. The d2 repair — a wider dash lane around #4's new post,
or restoring his advance — is the next session's first item; it needs its own measurement
runs and cannot fit this window.

## How it was verified

- `walkreal_0700000b_live` (whole binary, `--test-threads=1`, 40 s): **6 passed; 1 failed
  (the d2 regression above); 1 ignored (the 00 Agent ledger)**.
- `interactables_0700000b_live` (27 s): **14 passed; 0 failed**.
- The lever work ran with the `[0b-stall]`/`[0b-flight]`/`[0b-band]` diag prints and
  NF-real damage guards in /tmp (a9_run1..10, not committed); every run real input only —
  no teleports after the load, no pokes.
- The full `tools/test/suite.py` could NOT run in this sandbox, same as sprint-8: the
  background jobs are reaped between tool calls (three starts died — one on a missing
  cargo PATH, two on the reap), the disk is 9.9 GB and a full four-crate rebuild ENOSPC'd
  at `probe_render4` (a linker Bus error at 656 MB free; `cargo clean` + the two required
  binaries built instead), and the remaining window did not fit the ~50-minute full build.
  The two required binaries above were built and run individually on the final tree.

## What's left

1. The 00 Agent cleanup round: a mapped clear-ray window for the #6/#8/#9 blast (the 16-ray
   probe from the P4 approach z 35-41, x 1-2, and the late strip (−8.5, 18.1)); or a dart-
   sleep on #5 to freeze the drift so the strip round's lock is stationary. Then the take +
   P5 drive are quiet (the a9_run3 take already fired; ~21 hp was in the bank at the zone).
2. The d2 sister's dash-lane repair (the merge-inherited #4 post change above).
3. The ring turret 16.0 — the fusebox task's (the sprint-8 attribution stands: GT turret
   #291, (−1.14, 49.06, −0.19); 0b builds NO fuseboxes).

## Live checks for the main agent

1. On 0700000b (00 Agent): from the deck's south strip (x −8.5, z 18.1) fire one 0x2e round
   with the crosshair on the band anchor #6 — expected: the lock takes #7 then #4 (the
   re-scan) and the blast detonates at #4's column spot ≈ (2.05, 58.07, 30.65), killing
   both. If the disc's round locks a different guard or detonates elsewhere, the port's
   homing fold (Bullet_homing 0x12f358 / the re-scan law 0x12f3b4) is the divergence.
2. The gap-mouth pieces: from the P2 crossing (0, 21) cast the disc's rocket ray toward #6
   — the port's round dies at ≈ (2.41, 57.95, 21.24) after 12.7 u. Confirm the disc's
   round survives that spot (if it does, the port has an extra collision piece there).
3. The merged sight tree's #4: watch him during the d2 P3→P4 dash — the port has him parked
   at (2.1, 58.0, 31.7) in 0x18→0x1 switched state. If the disc's #4 still advances down
   the gap column, the los-mask-repairs sight fold over-corrected his aggression.

## Usage

not visible to me — this environment exposes no token, cost or context-window figures to the
worker (the IM gateway metadata carries only session/chat ids).
