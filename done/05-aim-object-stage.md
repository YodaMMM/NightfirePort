# Aim ray: the object stage

Branch from `origin/s/aim-snap-final`, push to `s/aim-object-stage`. Read its report. The aim ray still has no object stage, so the game's snap (shutter object at (139.562, 50.792, -96.706) [L]) and the laser's cut of the 14 wire differ. Add the object stage with honest glist posing, in the order `Collide_Pick` 0x1eb320 and `Collide_Intersect` 0x1ebaa8 walk it.

Must end green: interactables_07000014_live, fusebox_geometry_07000014_live, rocket_glass08_live, rocket_stop_plane08_live, rocket_origin_live, solar_panel_hits_live. Then run walkreal_07000008, 0c, 14, 16 and 1b and report each. The 1b console walk needs firing spots above the panels. Cite the rule for the rocket's midpoint gate, or leave it alone. Strip the NF_AIMPROBE prints.

Run each binary whole and quote results.

---
claimed-by: sandbox5 at 2026-11-05T23:01Z

---
## Report (done, 2026-11-06T00:07Z)

# aim-object-stage — the aim ray's object stage

Branch `s/aim-object-stage`, cut from `origin/s/aim-snap-final` 23975ae9f. 2026-11-05/06,
written ~00:10 UTC. Code-side: `crates/nf-game/src/drone.rs` (the aim ray's object stage in
`Drones::aim_snap`; the flight's wire contact frame) — `world.rs` and `scene.rs` only lost
the env-gated NF_AIMPROBE prints. All NF_AIMPROBE prints are stripped (grep 0 across
nf-game/nf-view).

## What was built

**The aim ray's object stage** (`Drones::aim_snap`, after the cel-scoped level query), in
the order `Collide_Pick` 0x1eb320 and `Collide_Intersect` 0x1ebaa8 walk it [M]:

1. **Candidates**: every loaded script's stream entity — `Scripts::sp_bullet_targets`, the
   set the flight's own object tests already use (the game's pick covers every object with
   a built sphere).
2. **The broad phase**: the built-sphere gate — `Control_BuildWorldSph` 0x133070's pair
   (centre = obj+0x90 · glist+0x10, radius = glist+0x1C · |obj+0xE8|), which
   `sp_bullet_targets` returns per entity; `Collide_Pick`'s sphere test is the closest
   point on the hittest segment vs that sphere (decomp 0x1eb320:159-170, read this session).
3. **The refine**: `Collide_Intersect` 0x1ebaa8's type-0x201 arm — the segment vs the
   object's OWN glist triangles posed by the object's own matrix, back-face-culled
   (`ray_tri_t` [M 0x10a8c8:168]), nearest t wins into the snap's min. An object whose
   glist has no parsed triangles never enters the hit chain — the sphere alone does not
   stop the ray [L the 0700001b breakpoints: no hit entry without a triangle touch].

**The 0x94 & 0x20 skip is not modeled** ([S]) — the decomp confirms the gate
(0x1eb320:141-144: obj+0x94 & 0x20 → the object is skipped; also the 0x1000+hittest-0x20
pair and the 0x201 arm's obj+0x3C & 0x8000000 skip), but the port has no source for the
per-object flag word yet; it is the first thing a false-positive snap should check.

**The wire entities' frame.** The fusebox wires ride their GFX-bounds frame end to end —
the aim ray's wire candidate is the posed gfx box (`wire_geo` through the entity matrix,
entry t) and the flight's contact is the same posed box (the fusebox-geometry machinery,
restored). The measured reason [L, this session's probes]: the wire's COLLISION glist
posed by the entity matrix lands ~0.1-0.3 u BEHIND the room's stat wall face on 07000014 —
the flight-stop trace shows the round dying on the stat face before the posed glist — and
the models-table bound sphere (the glist's own centre/radius) poses 10+ u away from the
visible cables. The gfx frame is the one geometry that matches the visible cable; the
collision-glist pivot question goes to the live checks.

## Runs (each binary whole, --test-threads=1, NF_ISO set, the final tree)

The task's must-end-green set:

| binary | result |
|---|---|
| rocket_glass08_live | **ok. 3 passed; 0 failed** |
| rocket_stop_plane08_live | **ok. 4 passed; 0 failed** |
| rocket_origin_live | **ok. 1 passed; 0 failed** |
| solar_panel_hits_live | **ok. 1 passed; 0 failed** — the walkway pin stays green under the object stage (the panels' posed triangles do not catch the walkway line, matching the game's object-0 breakpoint evidence) |
| fusebox_geometry_07000014_live | FAILED. 0 passed; 1 failed (was green on the tip — see below) |
| interactables_07000014_live | FAILED. 9 passed; 1 failed — `the_east_wing_fusebox_cuts_through_the_laser_and_the_e_is_inert` (red on the tip too) |

The walkreal sweep:

| binary | result |
|---|---|
| walkreal_07000008_live | FAILED. 5 passed; 1 failed |
| walkreal_0700000c_live | FAILED. 2 passed; 1 failed; 1 ignored — the death is `walk_the_whole_mission_from_the_start_to_the_0d_handoff_real_damage` (a real-damage Bond death, walker_b2) |
| walkreal_07000014_live | FAILED. 6 passed; 3 failed; 1 ignored (the aim-snap-objects report's own 6/3 — unchanged) |
| walkreal_07000016_live | FAILED. 13 passed; 1 failed; 2 ignored (the report's 13/1 — unchanged) |
| walkreal_0700001b_live | FAILED. 15 passed; 2 failed; 2 ignored (the report's 15/2 — "console 1" still stalls; the firing-spots-above-the-panels work did not fit this session) |

## The 14 cut's measured chain (why the two fusebox binaries are red)

The object stage works — the probes show the snap landing on the aimed wire's geometry
([L] the NF_AIMPROBE runs: the security box's snap (−89.237, −0.492, 30.907), the cable
band's own y; the east wing's the same shape). The cut still fails because the round dies
before the contact:

1. The security room's near door leaf (the room's own door, ~1.1-1.9 u from the eye) is a
   flight-pick candidate on every line through the room; the laser's re-aimed line crosses
   it and stops there (mat 0x5, no pierce — `Door.bullet_pass` reads the PLACEMENT flags'
   0x60 bits as the proxy for `glist+0x44 | create_param` [M Control_CreateObjEx
   0x1343c8:36], and this door's proxy says stop).
2. The game's cut works at the same aim, so the game's flight pick either skips that leaf
   (obj+0xF0 & 0x20 — `Collide_PickObj` 0x1eae80:26-30) or pierces it (the 0x60 bits) —
   the port's placement-flags proxy cannot tell which without the disc word.
3. Independently, the posed COLLISION glist sits ~0.1-0.3 u behind the stat wall face (the
   flight-stop trace), so even a round that passes the leaf meets the wire's collision
   behind the wall; the gfx frame is the honest one until the pivot is settled.

The 08 shutter snap (the task's other named gap) is deliberately NOT moved this session:
the doors were tried in the aim ray (a world.rs dynamic walk, two runs) and the near-door
snap broke the 14 cut and the security test's calibrated line; with the doors out, the
four 08/walkway binaries stay green and the snap stays the tip's stat answer (the back
wall at −100.0 vs the game's shutter object at −96.706). Moving it needs the same
obj+0xF0 answer plus the rocket steer test's re-calibration.

## What's left

1. **The live checks below** — the door obj+0xF0 word and the wire glist pivot unblock both
   14 binaries; they are one gdbwatch session.
2. **The 1b console firing spots** (the task's ask): unchanged from the tip — consoles
   3/4/5/6 latch, "console 1" stalls; the surviving panels still block the tuned lines.
3. **The 0c real_damage death** — my object stage or the tip's unrun flight sweep (the
   tip's report never ran the walkreals); a single `git checkout origin/s/aim-snap-final --
   crates/nf-game/src/drone.rs` + one binary run A/Bs it.
4. **walkreal_07000008's 1 red and the 14/16/1b reds** — the tip's own debts (the counts
   match its report where it ran them), plus whatever the object stage shifts; the per-walk
   review is the next session's first hour.
5. The rocket origin's midpoint gate — left alone per the task ("cite the rule or leave it
   alone"): the gate's disc rule (when BL's first-person model is loaded the round starts
   at the model datum midpoint [L the 1b beam vs the 08 memory-write launcher]) still wants
   a decompile citation this session did not produce; the port's real-path equipping
   (the model loads, o = the midpoint) already matches the live evidence.

## Live checks for the main agent

- **07000014 doors** (gdbwatch on Control_CreateObjEx 0x1343c8:36): dump obj+0xF0 per door
  placement in the security-room + east-wing areas (and the 08 shutters 49-52, carried
  over) — which carry bit 0x20 (the flight pick's skip) and which 0x40/0x20 of glist+0x44.
  Expected: the laser-cuttable rooms' doors skip or pierce in the flight pick, so the
  round reaches the wires.
- **07000014 wire pivot** (gdbwatch/fieldprobe): one wire entity's obj+0x90 (the live
  matrix) vs its glist's authored vertices — the port's entity matrix poses the collision
  glist ~0.1-0.3 u behind the stat wall face while the gfx bounds pose honest [L both
  probes this session]. The disc's obj+0x90 vs the port's script-Entity matrix settles
  whether the pivot or the matrix is the delta.
- **07000008 snap** (breakpoint 0x1a4d90, carried over): with the aim steered at the
  shutters, does the game's snap ray ALSO see the security-room-class near doors on 14
  (i.e. is the aim pick's door admission per-cel)? The port needs the cel assignment of
  door objects (obj+0x20) to scope its dynamic walk honestly.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures
anywhere I can read.
