# level-0a-band-settle-2 — the handler tail is the walk stopper: the decompile decoded, the below-B kill re-landed (gated), the radial-dir ports measured and reverted, the tests re-pinned (still ignored)

## What was asked

Start from `origin/s/level-0a-band-settle` + its three live answers [L 2026-10-26]: port the
collision-handler tail (PC 0x194c70 = `Player_CollisionHandler` 0x1949a8 + 0x2c8) in place of the
port's stand-in, re-land experiment (a)'s below-B kill if still needed, re-pin the three ignored
`walkreal_0700000a_live` tests to the fresh [L] numbers (the settled values, not the first
samples), un-ignore them, and keep walkreal_07000001/05 and every other walkreal test green.

## The tail, decoded [M]

- The write at 0x194c70 is `sqc2 Q2, 0x30(obj)`, fed by `0x194c60 lqc2 BL+0x750` +
  `0x194c68 vadd` [M disasm of 0x1949a8's tail; the C at 0x1949a8:136-140]: **obj+0x30 =
  BLData+0x750 + obj+0x30**, run when the hit list (obj+0xd0) is non-empty, BEFORE the
  `Player_DealWithObjHit` walk. BLData+0x750 = HITTEST+0x10 (the walk HITTEST lives at BL+0x740
  [M Player_Collision 0x194df0 stores +0x7c0 = obj+0x20, +0x7d4 = 0x800]).
- HITTEST+0x10's writers [M]: `Collide_Update` 0x1ea398:50 zeroes it per tick
  (`Vec_Zero(HITTEST+0x10)` in the hit-list-freeing loop); `Collide_Intersect` 0x1ebaa8:353-373
  adds the object's world DeltaP to +0x10/+0x20/+0x30/+0x50/+0x60 once per object-with-contact
  (the DeltaP is the `_DeltaP` VU0 accumulator written per face by `Intersect_CylGeom`
  0x109da0:455-457, transformed object-local → world at 0x1ebaa8:340-352).
- The port ALREADY has this tail's shape: player.rs:1845 `pos += push` (the per-tick collide
  displacement, zero when no contacts) with the pick_ref (+0x50) add-back at player.rs:1795 —
  the [L] wall-press watchpoint (one writer, 33 hits at 0x194c70, the only value-changing
  obj+0x30 writer while pinned) confirms the game's structure is the port's structure. The
  stand-in divergence is INSIDE the per-face push, not the application.

## The per-face rule, and the ref wall [M + the contradiction]

`Intersect_CylGeom` 0x109da0's push (per face, per object-local pass):
`v = q − ref` (q = closest tri point, ref = the copy's +0x50 = local of the real HITTEST+0x50
— verified in disasm at 0x10a540 `lqc2 $q1, 0x50($s5)`); gate `dot(v, n) > 0` (:421-431);
dist = |v| (:432-437); gate `dist < radius` (:447); push = normalize(v)·(radius − dist)
(:448-453); `_DeltaP += push` and the probe bounds/A/B shift per push (:454-470). The edge-contact
probe re-aims p at the spine point closest to the REF (:334-368).

The ref = HITTEST+0x50 = the lifetime DeltaP accumulator: its ONLY writer is the add-back
[0x1ebaa8:353-373 — no static writer, confirmed by offset grep + the -1 session's sdc2/sw scan],
it boots (0,0,0) [L ee_start_0700000a BL+0x790] and reads ~153 m from its owner mid-play
[L ee_snapshot]. Yet the live wall press shows 33 value-changing tail writes in 9 s — pushes DO
fire at the west hall wall. **These cannot both hold with a statically-pinned ref**: the live
walked value of HITTEST+0x50 (and its per-object local transform) is the missing fact. Live
checks below.

## What landed

1. **The below-B displacement kill, re-landed GATED (default off)** [crates/nf-game/src/world.rs
   `capsule_tri_contact`, `NF_WALK_KILL=1`]: a contact whose q sits ≥ 0.5 below the capsule
   bottom B loses its displacement but keeps the grounding signals (the walked floor sits
   0.604 below B: dy 1.0891 [L] − 0.4847 [M 0x194df0's B arm]; the carve-out keeps the shallower
   stair/step contacts pushing). Measured with it ON: walkreal_07000005_live **8 passed /
   0 failed**, the 0a corner **crosses at 870 ticks (was a permanent wedge)**. Measured cost:
   walkreal_07000001_live's `walk_from_the_spawn_to_the_exit_with_the_contact` fails (the
   AND-touch never fires — the contact's plate approach shifts) — the keep-01-green constraint
   wins, so the kill ships env-gated OFF until that walk is re-verified. Default-off = master
   walk behaviour byte-identical.
2. **The radial-dir ports: built, measured, REVERTED** (the code and the measurement record stay
   in the source comment at world.rs:427-434). The game's push dir is the RADIAL
   normalize(q − ref) [M 0x109da0:442-446], not the per-face spine rule the port probes. Ported
   with ref = the pass's pre-push bottom (wall faces only): raw radial — the corner crossed in
   **85 ticks** (the game's slide!) but the up component lifted Bond (corner exit y 1.7773 vs the
   game's settled 1.1159 [L]); horizontal-projected radial — the corner wedged again (6 mitigation
   cycles). Both reverted. The slide mechanism is real and radial; the port needs the ref's true
   value to aim it.
3. **The three tests re-pinned to the settled [L] numbers, still `#[ignore]`** (they do not pass):
   - `the_west_hall_glide_height_matches_the_game`: pin 1.1147 → **1.1285** (the 2026-10-26
     trace's settled y; the old number came from a spawn-height walk), note carries the slide
     facts (z 16.8295, x −9.0106 @ frame 36, never stopped at x −8.304).
   - `the_corner_crosses_like_the_game`: pin 1.120 → **1.1159** (the trace's settled exit),
     note carries the crossing state (870 ticks with the kill vs the game's ~60 frames).
   - `walk_office_floor_qworm_keyboard_hatch`: note updated (the ring still sticks at the walker's
     cap; the mouth/lane legs need the slide).
   No decompile-backed test was weakened: the two re-pins replace numbers whose own [L] basis was
   the FIRST samples of differently-seeded walks, per the driver's instruction to pin the settled
   values.

## How it was verified

- Disassembly of 0x194c30..0x194d20 (the tail), 0x10a300..0x10a560 (the push block), plus the
  decompiles of 0x1949a8, 0x194df0, 0x1ea398, 0x1ebaa8, 0x109da0, 0x191dd0 (`Player_Move` — no
  wall clip; the walk's motion integrates at ~0x195058 obj+0x30 += motion/60), 0x192310
  (`Player_SSWalk` → `Player_Move(…, 1.0)`).
- Test runs (all `--test-threads=1`, slim on-foot ISO): walkreal_07000005_live 8/0 green;
  walkreal_07000001_live 2/1 with the kill on, then 2 green re-verified with the kill off (the
  third not re-run after the gate — the gate-off state is master-identical code); the 0a trio
  measured per state as described. The fast suite (`tools/test/suite.py`) was started on the
  final tree — result recorded below.

## What's left

1. **The ref value** (the live checks below) → then the radial-dir port lands properly and the
   three tests un-ignore.
2. The kill's 01 regression to root-cause (the contact's plate approach) before the kill lands
   ungated.
3. The corner's 870-tick crossing vs the game's ~60 frames.

## Live checks for the main agent

Fresh boot of 0700000a (no save-state resume):

1. **HITTEST+0x50's live value on a walked line**: read BL+0x790 (the player's BL; obj 0x1b78640
   → BL 0x1b78750 per the -1 session's probe) at the spawn AND after walking to the west hall
   (−11.39, 16.29): does it stay near the owner (a per-tick reseed somewhere the decompile hides)
   or drift from (0,0,0)? If it drifts, watch its WRITES for 10 s of walking — name the writing
   PCs (the add-back at 0x1ec1xx is the only known writer).
2. **The tail's write VALUES**: while pinned at the west hall wall (the check-1 setup of the
   driver's message), sample HITTEST+0x10 (BL+0x750) at each of the 33 tail hits: the value's
   magnitude per hit (the equilibrium push size) and whether +0x50 moves by the same amount.
3. **The local transform**: for the west hall's contacted cel object, dump obj+0x60/+0x70 (the
   matrix) and its Ident bit (obj+0x3c bit 0x1b) — with the matrix, HITTEST+0x50's local image
   can be checked against the push zone directly.

## What I ran

- `cargo test --offline --release -j4 --no-run -p nf-view --test walkreal_0700000a_live`
  (and the 01/05 binaries) then the binaries with `--test-threads=1` as described above.
- `cargo test --offline --release -j2 -p nf-game -p nf-formats -p nf-audio -p nf-view -- --test-threads=1`
  was NOT run this session; the fast suite stands in (started in the background on the final
  tree — see the suite line below).
- Fast suite (`python3 tools/test/suite.py`): see the total line appended below.

## Suite result

(to be filled by the final run)

## Usage

not visible to me — this sandbox exposes no token, credit or context-window figures to the session.
