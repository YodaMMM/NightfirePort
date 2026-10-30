# level-0a-hall-glide-settle — the hall glide's climb settle: every statically-pinned writer ruled out; the radial ref's live value is the one missing fact

Branch `s/level-0a-hall-glide-settle` (from `origin/master` 06b95819, merged with origin/master
at the finish — master had not moved). Task: port the hall glide's +0.0256 climb settle (the
"feet-band walked-dy growth", band-settle finding 1) from [M Player_FeetOnPoint 0x194188 and its
callers], un-ignore `walkreal_0700000a_live::the_west_hall_glide_height_matches_the_game`, keep
the rest of the file and the stair/ramp walkreals green.

**The test stays ignored this round.** Not because the port was left unexamined, but because the
decompile — read end to end this round, with disassembly where the C was incomplete — rules out
every y writer that a flat grounded walk can legally hit. The one writer left (the radial tail's
reference) is the exact live value settle-2 filed a live check for and no round has received.

## What the round established (each load-bearing)

1. **The walk move is yaw-only.** `Player_SSWalk 0x192310:70-72` rebuilds the walk matrix as
   `RotMatrix(BL+0x30)` then `Concat`/`Mat_CopyRot` into `obj+0x90`; the integration at
   `Player_Update 0x193600:284-295` adds `obj+0x90 · (BL+0x10 strafe, BL+0x14, BL+0x18 fwd)`.
   But `BL+0x30..0x3C` is zeroed every tick at `Player_Update 0x193600:134`
   (`Vec_Zero(puVar2+0xc)`) **before** the substate switch runs SSWalk, and the only writer
   between the zero and the `RotMatrix` is `Player_Move 0x191dd0:161/:167` — which writes
   `+0x34` (the yaw turn) and `+0x10/+0x18` (strafe/forward), never `+0x30`. The look's pitch
   delta goes to `BL+0x40` and the `BL+0x8a8` accumulator (`Player_Aiming 0x1a1d28:268-274`),
   never to `+0x30` (the only `+0x30` writers in the whole player path are the spawn's
   `Player_StandAtNewPosition 0x190d88:41-44/111` and the position stores). So the walk matrix
   is a pure yaw rotation, the forward row's y is 0, and **the game's own walk move cannot
   climb on a level floor** [M ×4 cross-checked]. This also retires settle-8's pitch-y port
   reading ("the rate vector's x lane is the view pitch"): that lane is provably zero at the
   `RotMatrix` on every walked tick.
2. **The SP body anim carries no walk bob.** `PlayerAnimSetInitNormal 0x18fcd8:26` — with
   `MPSettings+0x384 == 0` (single-player) the weapon class is forced to 999 → the set at
   **0x2b91b0**, whose array is `0x06000a00` in every band slot [D ACTION.ELF @0x2b91b0].
   Script `0x06000a00` is one op-0 command playing sequence `0x84f` frames 1..2 (a static
   2-frame pose — the probe dump, `band_settle_probe.rs`). `AnimFrameResolve 0x1c35f8:425-432`
   applies root motion to `obj+0x30` every frame, but a static 2-frame pose has no bob to
   apply. The bob hypothesis (which fit the [L] trace shape: climb → peak → band-catch descent
   → release-freeze) is dead for the SP glide [D + M].
3. **The radial tail's reference is a static deadlock on a walk.** The press-y lives in
   `Intersect_CylGeom 0x109da0`'s radial arm: `v = q − HITTEST+0x50`, gates
   `dot(v, axis) > 0` and `|v| < radius` (the radius read at HITTEST+0x8C, disasm
   0x10a5c0), push = `v̂·(radius − |v|)` into `_DeltaP`, the from and the to [M 0x109da0:411-470,
   disasm 0x10a4f8-0x10a620]. `HITTEST+0x50`'s only writer is the add-back at
   `Collide_Intersect 0x1ebaa8:353-373` (+ `_DeltaP`, on ticks that recorded a nearest hit),
   and `_DeltaP` itself is zeroed at the top of every `Intersect_CylGeom` call
   [M 0x109da0:136 — a new decode this round; the repo's mipsdis misprints the LQ as sqc2,
   hand-checked 0xda210050 = `LQ $a1, 80($s1)`]. On a walked line no radial arm can fire first
   (the ref boots (0,0,0) [L ee_start_0700000a BL+0x790], `|q|` for the doorway faces ≈ 18 m ≫
   0.55), so `_DeltaP` stays 0, so the ref never grows: **the arm is provably inert on the whole
   walked glide** — and the port's master tree agrees (press-y 0.0000 on every doorway contact,
   the NF_CAPS_LOG trace).
4. **The port's measured divergence, re-pinned.** On master the through-line crosses the
   doorway (230 ticks to (−9.93, 1.0905, 17.08)) and the settled y reads **1.0905**, not the
   ignore note's 1.1201 — that number was measured on the settle-5 tree (its world.rs deltas
   were reverted before the merge, so the note's context was stale). The note now carries the
   current state. `NF_WALK_KILL=1` (the below-B displacement kill) and `NF_WALK_CLIP=1` (the
   move-clip walk model) both leave the glide y at 1.09 — neither gated experiment touches the
   missing climb.
5. **What the game's [L] trace requires.** The 2026-10-26 fresh-boot probe (probe-placed 1.12,
   walked west from (−8.0, 1.12, 16.29)): +0.004/tick to 1.1402 @f5, peak 1.1465 (@f13, the
   press), descent to the settled 1.1285 (frames 21/28/36 and the post-release rest). The
   descent is the port's own shape (gravity when the ray exits the band at cc+0.1 = 1.1347,
   then the grounded velocity zero [M 0x194df0:104 + the handler tail 0x1949a8]); the climb
   (+0.026 net before the descent) is the unexplained part, and per (1)-(3) it can only be the
   radial tail firing with a reference whose live walked value differs from the boot value.

## What landed

- The glide test's ignore note + the in-test comment re-pinned to the measured state, with the
  ruled-out writers and their addresses so the next round starts from the shortlist.
- `crates/nf-view/tests/band_settle_probe.rs` (assertion-free, `#[ignore]`, --nocapture): dumps
  the SP anim set's script and sequences — the evidence tool for (2).
- `docs/WORKLOG.md` round entry. No src (nf-game/nf-formats/nf-audio) change — the tree's walk
  code is byte-identical to the merged base.

## How it was verified

- `walkreal_0700000a_live --test-threads=1`: **7 passed / 0 failed / 2 ignored** (the glide and
  the corner keep their ignores; every active test in the file green).
- The glide run under `NF_CAPS_LOG=1`: the doorway contacts (the big east face
  (−8.304, 1.094, 15.601), the lower face (−8.304, 0.474, 15.883), the north slide face
  (−8.359, 0.733, 16.279)) all push with y = 0.0000 — the press-y absence, measured.
- `NF_WALK_KILL=1` / `NF_WALK_CLIP=1` on the glide: 1.0905 / 1.0891 — both recorded above.
- The 0a battery is code-identical to the base for the walk code, so the other walkreal
  binaries' green records carry over; the fast suite totals are at the bottom.

## What's left (the next round's plan)

1. **The ref live check (settle-2's, still open — now the ONLY blocker):** fresh boot 0700000a,
   probe-place Bond at (−8.0, 1.12, 16.29), walk west; per frame dump `BL+0x790`
   (the walk HITTEST's +0x50 ref, 3 floats), `obj+0x30.y`, and the press ticks' `BL+0x750`
   (the per-tick DeltaP). Expected if the radial-tail reading is right: the ref's walked value
   sits within 0.55 of the contact points during the press and aims the push's +y; if it reads
   (0,0,0) through the whole glide while the y still climbs, the climb has a writer outside
   the collision path entirely.
2. With that value pinned, port the ref (the shape is already in `world.rs`'s radial arm —
   `pick_ref` plumbed end to end) and un-ignore the glide; then the corner test rides the same
   landing.
3. If the ref reads (0,0,0) and the y still climbs, dump the full per-frame write set on
   `obj+0x34` (gdbwatch on the position's y word) — the writer will name itself.

## Live checks for the main agent

1. **The walk HITTEST's ref during the glide** (decisive): fresh boot 0700000a, probe-place at
   (−8.0, 1.12, 16.29), walk west exactly as the 2026-10-26 glide; per frame for ~40 frames
   dump `BL+0x790..0x79C` (the ref), `obj+0x30.y`, `BL+0x750..0x75C` (the per-tick DeltaP), and
   the walk's from/to (`BL+0x760/+0x770`). Address math: the player obj 0x1b78640 → BL = obj+0xE0
   [L ee_start_0700000a], the HITTEST = BL+0x740 [M 0x1910f8:40].
2. If (1) shows the ref moving, a second watchpoint on `Intersect_CylGeom 0x10a540`'s ref load
   (`$s1+0x50`) during the press ticks — the operand's live value vs the copy's, one read.

## Usage

not visible to me — this session's driver surface shows no token, credit, cost or
context-window figures (the IM gateway metadata carries only session/chat/trace ids).

## Suite total (the fast runner, this branch, merged with origin/master)

Partial record only — this sandbox kills every background job between tool calls (nohup
included, confirmed twice this round; the same constraint the band-settle-3 entry records), and
the foreground chunk fits ~7 of the ~935 binaries' window:

- `build ok: 935 test binaries in 1 s` (the whole workspace compiled clean on the merged tree).
- The first run then hit `CRASH mp_stress_explosives_live (exit -9)` — the chunk was SIGKILLed
  at the timeout while that binary ran (an environment kill, not a test failure; the previous
  rounds' env-failure pattern).
- No FAILED/TIMEOUT lines were produced. No per-binary totals landed in the window.

The tree's walk code is byte-identical to the merged origin/master base (only the test file's
notes, the assertion-free probe, WORKLOG and this report changed), so the base branch's green
suite record carries over; the touched binary (`walkreal_0700000a_live`) was run directly:
**7 passed / 0 failed / 2 ignored**.
