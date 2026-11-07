# recovery-loop-07 — why Bond left his room's box on 07000007

Task: branch `s/recovery-loop-07` from `origin/s/void-recovery-2` (670687740). The
void-recovery-2 report's §4 item 1: a recovery LOOP at [94.84, ~43.1, −72.19] on the
use_census_07000007 walk — dozens of fires from tick 3688, the walker's E press at the door
out of range. The working theory was "the walk's push-out puts him outside cel 15's box".
**The theory was wrong: nothing pushes him anywhere.** This report is the finish record.

## 1. What actually happens at (94.84, ~43.1, −72.19)

Instrumented per-tick trace of the real census walk (`NF_BOND_TRACE`, a scratch patch since
removed; the [recovery] log is the shipped `NF_RECOVERY_LOG=1`):

- **The floor is there.** A straight-down `segment_hit` probe at (94.839, *, −72.25) hits the
  corridor floor at **y = 48.9999** (normal (0,1,0)) — in the flat world AND scoped to cel 15
  [M probe run on this branch]. Cel 15's box is [80.75, 42.75, −73.25001]..[98.49949,
  54.249928, −59.171986] [M `debug_scope_cells`], and (94.839, 48.5, −72.24957) is inside it —
  `find_cel` links it [M trace, tick 3681].
- **Bond is placed UNDER the floor.** The census's door stand-in teleported (a raw
  `sim.bond.pos` write) to door 4's pos − 0.5y + 1.0z. Door 4's pos is (94.839035, **48.99966**,
  −73.24957) — its y IS the floor height — so the raw write starts the root at 48.49966, half a
  unit below the floor surface. The floor ray points DOWN and one-sidedly sees no face above
  the root; no capsule contact fires either (`g false`, `pre_collide == pos` every tick — no
  push, no displacement).
- **He falls out of the box.** Carrying the previous fall's terminal velocity (the raw write
  keeps `vel`, which was already [0, −45, 0] from earlier buried placements — every placement
  in this walk falls through the same way, first at tick 1), he drops 0.75/tick from 48.5, and
  at tick 3688 pos.y 42.49966 leaves cel 15's box (min 42.75). No portal, `find_cel` misses
  (42.5 < every room's box there), so `control_handle_cel_change`'s recovery clamps him into
  the old box: y := min + r = 43.8076, z := min.z + r = −72.1924 [M 0x133288:186-221, r =
  1.0576]. `vel` survives the recovery, gravity holds −45, and the loop repeats every 2 ticks:
  **2416 fires** to the end of the walk, Bond pinned ~5.2 under the floor while the door sits
  5 units above his head — the E press lands out of reach.

So the difference from the game is not the push-out and not the box: it is the PLACEMENT
STAND-IN. The game never writes obj+0x30 raw. `Player_StandAtNewPosition` 0x190d88 lifts the
probe 0.1y (`fStack_3c = obj+0x34 + 0.1` :104-106), runs `build_PointOnFloor` 0x1c95a8 — a
down ray (0,−1,0) × 3.0f, IN-OUT, scoped to the FindCel cel, obj filter NULL — and adds **1.1
unconditionally** (`fStack_3c += 1.1` :110-112; on a PointOnFloor miss the y is therefore
placement + 1.2), storing obj+0x30 while obj+0x40 keeps the RAW param position (:21-28)
[M decompile `work/decomp/ACTION.ELF/00190d88…` + `001c95a8…`]. At the census spot: probe
48.6, ray misses (nothing within 3.0 below — probed), y := 48.5 + 1.2 = **49.7** → the floor
band ray grounds him on the corridor floor, head ~50.4, door in reach. The void-recovery-2
report's own [S] deviation ("the port keeps the caller's y here — the placement tests' spots
stand near floors and settle by physics") is exactly the assumption this spot breaks.

## 2. The fix

- `Sim::place_bond` [crates/nf-view/src/scene.rs] now takes the game's snap: probe = p+0.1y,
  `segment_hit_scoped(cel, probe, probe − 3.0y)`, hit → y := hit + 1.1, miss → y := p + 1.2
  [M 0x190d88:104-116]. `prev_pos` keeps the raw param (obj+0x40, :21-28). The snap delta is
  ≤ 1.9 in y, below the >3.0 poke arm's threshold — a placed Bond relinks as a placement, not
  a poke.
- `use_key_census_live.rs` places through `place_bond` (yaw preserved) instead of raw
  `bond.pos` writes — `place_near`, `use_any_door`, and the 07000012 monitor stand-in. The raw
  write was the walkreal stand-in predating `place_bond`; it skips the vel clear and the snap.

**use_key_census_live: 20/20 green** on this branch (run twice, before/after removing the
scratch trace instrumentation; `--test-threads=1`). The 07000007 loop is gone — zero
recovery fires in that walk now.

## 3. The rest of the gate, and two honest failures

| binary | result |
|---|---|
| void_recovery_07000003_live | PASS (exit 0) |
| fix_death_restart_live | PASS (exit 0) |
| interact_07000007_live | 7/7 |
| interactables_07000002_live | 18/18 |
| use_key_census_live | 20/20 |
| fix_blast_scripts_live | **2/3** — `the_destroyable_smash_plays_its_type_script` FAILED ("the type-0 destroyable never smashed") |
| interact_07000006_live | **8/9** — `the_well_shaft_is_open_and_the_hurt_volume_is_game_inert` FAILED (centre y 41.21, expected 39.4..40.4) |

Both failures are the same story as the census's, in reverse: those two tests place Bond
somewhere the OLD no-snap stand-in let physics settle him into a calibrated band (the well
mouth: placed at 40.5, he fell onto the barrel body #65 and rested ≈ 39.9–40.4; with the
game's snap the down ray hits at ≈ 40.11 — the barrel top — and the game's own math stands him
at 41.21, 0.04 above the capsule rest, i.e. ON the barrel but higher than the test's band).
The snap is the game's code [M 0x190d88:104-116]; the two bands were calibrated on the
deviation. I did not re-tune them inside this session's budget — that is a deliberate,
documented stop, not a green-wash: **recovery (void-recovery-2) can merge once these two
expectations are re-derived against the game's snap**, which needs the live checks below, or
an explicit decision that those two scenarios keep the physics-settle stand-in (then the snap
needs an opt-out flag for them).

## 4. Recovery fires across the walks

- use_census_07000007 (the failing walk): **2416 → 0** after the fix.
- walkreal_*_live battery with `NF_RECOVERY_LOG=1` fire counts — PARTIAL (the sandbox ran out
  of time mid-battery; these ran green with **0 recovery fires** each):
  walkreal_07000001_live 8/8, walkreal_07000004_live 10/10, walkreal_07000005_live 14/14.
  (walkreal_07000003_live needs > 100 s — not finished; 02 not built in time.)
  The remaining command (per binary):
  `NF_RECOVERY_LOG=1 ./target/release/deps/walkreal_0700000X_live-* --test-threads=1 2>&1 | grep -c '^\[recovery\]'`
  The walkreal walks do not place mid-walk, so the `place_bond` change should not move them;
  the census's 2416 fires were the only measured loop on the base branch.

## 5. Live checks for the main agent

1. **The well mouth (07000006)**: a real `Player_StandAtNewPosition` call (not a poke) at
   (31.82, 40.5, 17.26). Expected per this report: Bond ends at y ≈ 41.21 (PointOnFloor hit
   ≈ 40.11 + 1.1), standing on the barrel. If the game instead settles him at ≈ 39.9, the
   port's snap ray hit the wrong face and the test band is right.
2. **The census door (07000007)**: same call at (94.839, 48.5, −72.24957) — the under-floor
   spot. Expected: y := 48.5 + 1.2 = 49.7 (PointOnFloor miss), Bond standing on the corridor
   floor, E on door 4 works.
3. **The type-0 destroyable (0700000d)**: the fix_blast_scripts placement spot with the snap —
   where does the game stand the player, and does the smash script still fire from there?

## 6. What's left

1. Re-derive the two gate bands (§3) against the game (live checks above) and re-tune or
   opt-out; then recovery can merge.
2. The walkreal fire counts (§4).
3. Full suite before the next push (this session ran the binaries listed in §3 only).

## Usage

(not visible to me — this sandbox exposes no token, cost or context-window figures for the
session; nothing to copy)
