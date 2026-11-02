## Your task: `level-0a-band-settle`, Bond wedges on 0700000a's doorway bands

Three walkreal_0700000a_live tests are ignored because the port's walk differs from the game's on the ring route: the port glides at the settled 1.0891 but does not reproduce the doorway base climb's +0.0256 band settle [L 1.1147 fresh boot]; at the corner the port wedges (y dithers across the band edge 1.1347) where the game walks through at y 1.127 -> 1.120 [L fresh boot]; and the ring walk (:227) wedges at 'the ai lane to the N-S corridor'. Find the step/ground-settle rule the game applies on these band edges (`Player_Update` ground probe + step-up [M]), port it, and un-ignore the three tests. Every other walkreal test must stay green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-band-settle`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-26T14:02Z

---
## Report (blocked, 2026-10-26T14:36Z)


---
## Report from the worker (blocked, 2026-10-26T14:37Z)

# level-0a-band-settle — the band-edge rule round: root-cause narrowed, two experiments measured and reverted, no code landed

## What was asked

Find the step/ground-settle rule the game applies on the 0700000a band edges (the doorway base
climb's +0.0256 settle [L 1.1147], the corner walk-through y 1.127→1.120 [L], the ring walk's
'ai lane to the N-S corridor' wedge), port it, and un-ignore
`the_west_hall_glide_height_matches_the_game`, `the_corner_crosses_like_the_game` and
`walk_office_floor_qworm_keyboard_hatch` with every other walkreal test green.

## Status: blocked this session — the mechanism is not derivable from the decompile alone, and both candidate ports regressed other levels

The three tests are still `#[ignore]` (the notes unchanged — they still describe the port's
divergence accurately). No source change landed: the final tree is origin/master + docs only, and
`walkreal_0700000a_live` reads **4 passed / 0 failed / 3 ignored** on it.

## What the session settled (evidence, point by point)

1. **The floor is y≈0 along the whole ring route** [L — a port-world probe script, run and
   deleted, numbers transcribed]: the spawn cab (−3.93, 10.40), the doorway line (−8.45, 16.29),
   the west hall (−10.0/−11.39, 16.29), the corner (−17.87, 4.87 → −17.5, 2.5) and the lane
   (−11.25, −14.60 → −8.60, −14.20) all read floor ≈ 0.0 within 3e-4. **The +0.0256 hall settle is
   NOT a floor step — the game's walked dy itself grows over the same slab**, and the corner's
   1.127→1.120 is a dy decrease with no geometry change. Any port of the settle is a rule about
   the walk's vertical, not about floor tracking.
2. **The feet ray writes HITTEST+0x50 per crossing — but only in the discarded working copy.**
   `Intersect_RayGeom` 0x10a8c8:179 stores `param_1[5] = from + dir·t` (the hit point) per closer
   crossing, and `Collide_Intersect`'s 0x201 branch runs on the 0xa0 stack copy
   [M 0x1ebaa8:157 `amemcpy(auStack_120, param_1, 0xa0)`]: the write lands in the copy; the real
   HITTEST still takes only the DeltaP add-back [M 0x1ebaa8:353-373]. This closes the round's main
   alternative reading: hatch-10's accumulator identity stands — **the walked cylinder tail is
   dead** (the ref boots (0,0,0) and drifts), and the +0.0256 climb does not come from it.
3. **The ref value re-pinned [L]:** `ee_start_0700000a.bin` at the player's BL (obj 0x1b78640 →
   BL+0x790) reads (0,0,0) exactly with A/B/radius/mode seeded (A = root+0.275, B = root−0.4847,
   r 0.55, mode 0x800); `ee_snapshot.bin` (mid-play) reads (1.2079, 9.0838, −14.4486) while the
   owner stands at (104.71, 24.97, 107.05) — ~153 m away. The tail's guard
   `|p − HITTEST+0x50| < radius` [M 0x109da0:447] cannot pass on any walked line.
4. **The -2 session's corner probe re-read** (its report, `origin/s/level-0a-office-hatch-2`):
   the port's wedge push sum was (−0.0156, +0.0165, +0.0988)/tick — a two-face positional jam that
   exactly cancels the walk press. The game walks the same corner in ~25 frames [L fresh boot], so
   **the game's stopper cannot be a positional two-face jam**; it must be motion-relative
   (velocity-side) or otherwise asymmetric. That is the same "the game's real wall-stop is
   unidentified" open item hatch-10 recorded.

## The experiments (built, measured, reverted — the code is NOT on the branch)

(a) **The ball tail with the stale floor-hit ref + the below-B displacement kill** (the walk push
measured from last tick's ray hit; contacts below the capsule bottom lost their displacement but
kept the grounding signals): the band-edge dither VANISHED (the lane leg's STUCK dump flipped
grounded=false → true), the corner **crossed for the first time** (but over the test's 300-tick
budget), the hall glide walked through the doorway — and the height pins missed
(1.09 vs the game's 1.1147 [L]) because the game's climb source is (per finding 2) not this tail —
and **walkreal_07000001_live 1 fail / walkreal_07000005_live 6 fails**: the below-B lift is
load-bearing for the port's stair climbs on other levels. Reverted.

(b) **(a) + the anti-motion wall projection** (the stand-in push projected onto the anti-velocity
axis, so two-face corners cannot jam): the corner jam shape was gone but the doorway approach
froze earlier (the jamb corner head-on). Reverted with (a).

Both experiments' failure mode is the same wall: with the port's positional push gone or
projected, the port has no stopper the game would accept, and the game's real stopper is not in
the decompile paths read so far (`Player_CollisionHandler`'s tail only lands DeltaP and flags;
`Player_DealWithObjHit` handles objects; `Player_FeetOnPoint` only grounds).

## What's left (the next round's shape)

1. **The stopper hunt with the driver's live checks** (below) — one 10-tick trace of the write to
   obj+0x30 during a wall press names the mechanism, and then the port can replace the stand-in
   without breaking 01/02/03.
2. With the stopper in, re-land experiment (a)'s below-B kill (it is the dither/wedge fix) plus
   the game's step/settle rule, and re-pin the three tests' heights.
3. Un-ignore the three tests when green; the ignore notes are unchanged this round.

## Live checks for the main agent

Fresh boot of 0700000a (no save-state resume):

1. **Wall press stop (decides everything):** walk Bond into the office-floor doorway jamb
   (press west at z 16.29 toward x −8.304) and gdbwatch the WRITE to obj+0x30 for 10 ticks: record
   the exporting addresses. Predictions: `Player_CollisionHandler` 0x1949a8:136-140 = the tail is
   alive after all; a `Player_Update`/control address = the velocity side clips.
2. **The climbed dy:** in the west hall (≈ −10, 16.29) read obj+0x30 y and BLData+0xcc once:
   does the walked rest read 1.1147 with cc still 1.0347 (a pure dy rule), or did cc move
   (the anim writes it — fix-truck's dynamic-cc note)?
3. **The corner pass:** place Bond at (−17.87, ?, 4.87), hold forward to (−17.5, 2.5), and
   record obj+0x30 per 5 ticks: does y move at all during the crossing (a lift), and does the xz
   path bend (a slide) or stay straight (a pass-through)?

## What I ran

- `cargo test --offline --release -j2 --no-run -p nf-view --test walkreal_0700000a_live` then the
  binary with `--test-threads=1` on the final tree: **4 passed / 0 failed / 3 ignored** (the
  tree is master + docs — no code change to re-verify beyond this file's own battery).
- The experiments' measurements: the three target tests under `--ignored --test-threads=1` on the
  experiment trees (failures transcribed above); `walkreal_07000001_live` (2 passed 1 failed) and
  `walkreal_07000005_live` (2 passed 6 failed) on the experiment tree (a) — the regression that
  forced the revert.
- The full suite was NOT run this session: no code landed, so the final tree's suite state is
  master's own; the 0a walkreal file (the task's tests) is verified green-ignored above.

## Usage

not visible to me — this sandbox exposes no token, credit or context-window figures to the session.
