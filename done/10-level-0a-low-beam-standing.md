# level-0a-low-beam-standing — the low beam #621 fires on a standing player

Branch from origin/master (it already carries s/player-trigger-contact).

`walkreal_0700000a_live::low_beam_621_fires_standing` is ignored as a port gap: the standing trigger toucher is the box pos−0.05..pos+1.26 r 0.55 (drone.rs player_touchers), and its reach does not meet the low beam #621 the way the game's player body does.

Do:
1. Re-run the test on master first: player-trigger-contact may already close it. If it passes, un-ignore it and stop there.
2. Otherwise derive the player's trigger contact body from the game [M Player_Collision 0x194df0 and the trigger contact pass] and fix drone.rs's toucher to match; cite every number.
3. Un-ignore the test; run walkreal_0700000a_live, interactables_0700000a_live, every interactables_*_live, then the full suite.

---
claimed-by: sandbox4 at 2026-11-02T05:18Z

---
## Report (blocked, 2026-11-02T06:09Z)


---
## Report from the worker (blocked, 2026-11-02T06:10Z)

# level-0a-low-beam-standing — the low beam #621 fires on a standing player

Branch `s/level-0a-low-beam-standing` (from origin/master 0ebfee9a). Status: **not finished — the
test stays `#[ignore]`d**; the session produced the contact-pass derivation, a verified
reconciliation proof, and a reusable ported contact function. The reason this is not `done` is at
the end.

## What the task asked

`walkreal_0700000a_live::low_beam_621_fires_standing` is ignored as a port gap: the standing
trigger toucher (the measured box pos−0.05..pos+1.26 r 0.55, `drone.rs player_touchers`) does not
reach the low beam #621 the way the game's player body does. Steps: re-run on master first; else
derive the player's trigger contact body from [M Player_Collision 0x194df0 and the trigger contact
pass] and fix `drone.rs`'s toucher; un-ignore; run the 0a suites + the full suite.

## What was done

1. **Master re-run (step ①)**: `low_beam_621_fires_standing` FAILED on master (ch 0x24 silent on
   the standing crossing) — player-trigger-contact did not close it.
2. **The contact body, derived [M]** (`work/decomp/ACTION.ELF/00194df0_Player_Collision__FP7obj_tag.c`):
   - case 0/6: radius 0.275 (`0x7cc = 0x3e8ccccd`), A = pos+up·0.55, B = pos+up·(0.275−cc)
     [M 0x194df0:204-227] — the sphere bottoms at pos−cc = the floor;
   - the default arm (= the standing walk, substate 0; the port's `body` build,
     scene.rs:3976-3981): A = pos+0.275, B = pos+0.55−cc, r 0.55 [M 0x194df0:315-333] — bottom
     also pos−cc; case 3 = the crouch [M :228-248]. cc = `*(*(obj+0xdc)+0xcc)`, the stance root
     height (standing ROOT_HEIGHT 1.0347183, crouch 0.55 [M player.rs; live hip 1.035]).
3. **The trigger contact pass, derived [M]** (`00109da0_Intersect_CylGeom__FP11HITTEST_tagP11HITDATA_tag.c`):
   per triangle ONE test point, not a capsule sweep: n = the tri's PACKED normal (an index scaled
   by 1/16384 — normalized, winding from the data) [M :283-296]; den = dot(n, b−a); if |den| >
   0.0002 the plane crossing t = clamp(−dot(n, a−v0)/den, 0, 1) and the tested point = a+t·(b−a)
   [M :310-330] — with the near-perpendicular HALF-SPAN VETO [M :321-322] `den>0.999 && t>0.5` /
   `den<−0.999 && t<0.5` → the tri is skipped; the parallel branch projects the tri centroid onto
   the segment [M :344/:371-415]; contact = point-triangle distance < r, strict [M :524 with the
   best seeded at the radius :209].
   Ported as `pub fn cylgeom_hits_tris` in `crates/nf-game/src/triggers.rs` (r == 0.0 keeps the
   shipped zero-radius bullet segment arm).
4. **Attempted fix (wired, tested, reverted)**: standing toucher = `player.body` (the default arm
   with the real cc) + `cylgeom_hits_tris` in `Trigger::touched_by`. Result:
   `low_beam_621_fires_standing` **PASSED**, but `fix_truck_live` FAILED **both** tests — the
   truck chain stalled at the very start (0x22/0x72/0x70/0x65 never fired; Bond never left
   (100, 25.88, 108); the sister's "miss −12.264" is that stall, not plate math). The same class
   of stall the first reverted attempt saw. Reverted in commit 3de7801e; `fix_truck_live` 2/2
   green again; `walkreal_0700000a_live` 10 passed / 3 ignored (master parity).
5. **The reconciliation proof (why a toucher swap cannot work)**: the game's own live facts
   require two different bodies.
   - 07000005's ch-0x70 park plate (a flat quad at y 37.7418 [D script stream 6 key 8]) stays
     silent under the standing ride and fires crouched [D fix-truck's trace; the sister test pins
     the standing miss at 0.23 = the reach model pos+0.15−0.55, i.e. reach bottom ≈ pos−0.4].
   - 0700000a's #621 band y 0.13-0.39 [D] fires on the standing crossing [L 2026-11-01, fresh
     boot] — that band tops 0.74 below the standing pos (1.13): reach ≥ pos−1.0.
   - One body cannot satisfy both (at the 05 park the standing and crouched capsule bottoms differ
     by 0.025 yet the game separates the two stances). ⇒ the laser beams ride a DIFFERENT contact
     pass than the floor plates — the pick capsule that "reaches the floor" (the case 0/6 arm, as
     the test doc itself cites) — and the port folds both into one trigger toucher. The vetoed
     opening plate in 05 is consistent with the veto being real [M :321-322] but applied to a pass
     the port feeds the wrong body.

## What is left (the next session's shape)

1. [D] Classify the 0a laser placements (#607/#614/#621) — trigger class 0x1D or the sensor
   family? (`docs/fields/sensor.md`; the port's `sim.drones.triggers` build in nf-formats/level.rs
   / placement.rs shows the class byte.)
2. Split the standing contact: floor plates keep the measured reach toucher; the beam/sensor
   volumes get the pick capsule (case 0/6: A pos+0.55, B pos+0.275−cc, r 0.275) with
   `cylgeom_hits_tris` as the refine. Everything needed is already on the branch.
3. Un-ignore `low_beam_621_fires_standing`; run walkreal_0700000a_live,
   interactables_0700000a_live, every interactables_*_live, fix_truck_live and the full suite.

## Live checks for the main agent

- `fieldprobe.py` (or a trace) on 0700000a: the class byte (obj+0xFF) of the placement-621 object
  and of placement 607/614 — expect 0x1D if they are plain triggers; anything else names the pass
  that fires them.
- On 07000005 fresh boot: `switch_channels[0x45]` (=1 [L ee_start_05.bin]) while watching which
  object the ch-0x45 contact credits — if the boot-tick fire is the player's, the standing contact
  body at the spawn (BLData 24.9633, plate 24.4065) needs to reach 0.53 above the feet, which the
  measured-reach model (pos−0.4) does and the case 0/6 capsule also does (bottom = feet).

## Verification

- `NF_ISO=$NF_ISO cargo test --offline --release -j2 -p nf-view --test fix_truck_live` → 2 passed
  (after the revert).
- `... --test walkreal_0700000a_live` → 10 passed / 3 ignored (master parity; the target test is
  one of the ignored).
- The attempted fix run: `low_beam_621_fires_standing` ok (with the capsule+veto wired), then
  `fix_truck_live` 2 failed → reverted (the evidence trail is in docs/WORKLOG.md).
- The full suite was not run: the tree's net code change vs master is one new unused-by-default
  pub fn + comments; the two suites above cover the touched behaviour.

## Usage

not visible to me — this session exposes no token, credit or context-window figures (no usage
readout is shown in the environment; nothing to copy).
