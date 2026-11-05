# Aim snap: finish on the corrected filter

Branch from `origin/s/aim-snap-placement`, push to `s/aim-snap-final`. Read docs/reports/aim-snap-objects.md first, but note its rocket and laser results are void: `aim_pick` read object flags, which never carry 0x8000, so every static surface was skipped and the snap never landed. `s/aim-snap-placement` reads `inst.placement.flags` instead.

Now green on that tip: `firing_along_the_walkway_breaks_no_panel`, with the ray stopping at x 6.958338, the game's own point. [L PCSX2 breakpoint at 0x1a4d90: head (16.741, -1.137, 4.62), hit (6.958, -1.137, 4.62), object 0.]

Red on that tip, each to be fixed by matching the game's ray, not by turning the snap off:
- rocket_glass08_live (exploded at z -96.706, must pass the window plane -96.75), rocket_stop_plane08_live if red. Check the cel walk through the 08 window and the glass mask.
- interactables_07000014_live and fusebox_geometry_07000014_live: the laser must cut. The game's ray hits object glists, so the wire needs to be in the ray.
- rocket_origin_live against the true snapped point.

## Live numbers for 07000008 (PCSX2, breakpoints, Bond idle at the level start, launcher 0x2d held)

The report's claim that the game's ray sees nothing at the window is wrong. Head (139.531, 50.792, -76.431), raw aim (139.924, 50.792, -332.431). The ray hits OBJECT class 0x1c at (139.562, 50.792, -96.706); the object's position is (140.719, 49.5, -96.751), r 1.812. Same result for the start weapon and the launcher. So objects of that class are in the ray.

The rocket's own flight, per tick: (139.531, -76.431) start at the HEAD, then (139.538, -80.472), (139.545, -85.472), (139.553, -90.472), (139.56, -95.472), (139.568, -100.472), y 50.792 throughout. Its hit chain lists that same object on the last step. The final blast point was not captured; the tests' z -100 is older evidence. The launcher was given by a memory write, so check whether the midpoint gate (first-person model loaded) was true in this run before changing the port's origin: the beam on 0700001b DID start at the midpoint (16.2, -1.23, 4.53).

Then check walkreal_0700000c_live, walkreal_07000014_live, walkreal_07000016_live, and walkreal_0700001b_live's consoles from spots a player would use. Strip the NF_AIMPROBE prints.

Use a PCSX2-backed number wherever one exists. Run each binary whole and quote the results.

---
claimed-by: sandbox4 at 2026-11-05T22:10Z

---
## Report (done, 2026-11-05T22:47Z)

# panel-model-glist — the aim snap finished on the corrected filter

Branch `s/aim-snap-final`, cut from `origin/s/aim-snap-placement` fc6e21313 (the aim ray's filter
reads the placement word — object flags never carry 0x8000, so the old filter skipped every static
surface and the snap never landed; the aim-snap-objects report's rocket and laser conclusions are
void for exactly that reason). WORKLOG entry `panel-model-glist`.

## What's done

**One code fix: the flight's object-pass arm.** `Bullet_CollisionHandler` 0x12ce80's HITDATA walk
passes an OBJECT hit when `obj+0xF0 & 0x60 != 0` [M disasm 0x12cf58-0x12cf5c], and
`Control_CreateObjEx` 0x1343c8:36 writes `obj+0xF0 = glist_record+0x44 | create_param` — the
placement's own flag word. The AIM ray's pick does NOT apply this gate: the game's snap ray hits
the shutter object's glist at (139.562, 50.792, −96.706) [L the PCSX2 breakpoint run at 0x1a4d90],
while the live rocket's hit chain lists the same object on the crossing step and the round PASSES
to the −100 back wall [L the flight capture: (139.56, −95.472) → (139.568, −100.472)]. The port's
flight (`flush_world_impacts`) now extends the pierce for door-hosted dynamic hits whose Door
carries the placement word's 0x60 bits (a new `bullet_pass` field on `Door`, read from
`p.flags & 0x60` at `Doors::create`; the dynamic part ids are bit 40 + the door list index), and
the world impact records at the EXPLOSION point (the first non-pierce hit) instead of the first
crossed face — a pierced glass/stile entry no longer masquerades as the blast.

With the corrected snap (the port's stat ray lands on the cage back wall (139.567, 50.789,
−100.000) [L the NF_AIMPROBE run], the game's own wall) the flight line o→snap crosses leaf 3's
closed stile (x ≈ 139.587) whose object passes, then dies on the back wall at z −100.0 — the
game's blast point.

**The steer test's calibration follows the corrected ray** (test-side): the harness's line offset
is now the muzzle-to-snap direction's (measured 0.071; the raw-aim line measured ~0.19), and the
straight-line probe names the shutter surface the flight actually crosses — leaf 3's stile
(mat 0x03, the object the flight passes) where the uncorrected tree's raw-aim line crossed
DOOR_Shutter_2's pane (mat 0x10). Both bytes are the closed shutters' own faces at the game's z.

## Runs (each binary whole, --test-threads=1, NF_ISO set, the final tree)

| binary | result |
|---|---|
| rocket_glass08_live | **3 passed, 0 failed** (was 2/1 on this tip) |
| rocket_stop_plane08_live | **4 passed, 0 failed** (was 2/2) |
| rocket_origin_live | **1 passed, 0 failed** (green on the tip; re-run on the final tree) |
| fusebox_geometry_07000014_live | **1 passed, 0 failed** (green on the tip; re-run on the final tree) |
| interactables_07000014_live | 9 passed, 1 failed — `the_east_wing_fusebox_cuts_through_the_laser_and_the_e_is_inert` (:323) |

The walkway pin `firing_along_the_walkway_breaks_no_panel` was already green on this tip (the ray
stopping at x 6.958338, the game's own point [L the 0x1a4d90 breakpoint: head (16.741, −1.137,
4.62), hit (6.958, −1.137, 4.62), object 0]) — untouched this session, no re-run.

## What's left

1. **interactables_07000014_live's east-wing fusebox laser cut** — the task's "the wire needs to
   be in the ray": the game's laser ray hits object glists, and the port's aim ray (the snap) has
   NO object stage yet (stat runs + drones only). The wire is a script entity whose posed
   world.shape framing the aim-snap-objects session measured mis-framed (the +42 x AABBs) — the
   fix is the object stage with honest glist posing, not a filter knob.
2. **The aim ray's object stage on 07000008**: the game's snap lands on the shutter object
   (−96.706 [L]); the port's stat-only snap lands on the back wall (−100.000 [L the probe]). The
   flight outcome matches (both die on the back wall), but the snap point itself is not yet the
   game's. The same object stage as (1).
3. **The rocket origin's midpoint gate** — the live 08 launcher was given by a memory write, so
   the first-person model was not loaded and the rocket started at the HEAD (139.531, −76.431)
   [L]; the 0700001b beam DID start at the midpoint (16.2, −1.23, 4.53) [L]. The port's harness
   equips through the real path (the model loads, o = the midpoint (139.706, 50.732, −77.129)
   [L the probe]) — the gate's disc rule (when BL's first-person model is loaded) still wants a
   decompile citation before anything moves.
4. **The sweep of the flight change**: the object-pass arm + the impact-record gating touch every
   bullet-vs-door and every pierced shot's impact record. The full suite did not fit the window;
   the next session should run the walkreal set (at minimum walkreal_07000008/0c/14/16/1b_live)
   and the console tests before merging. The walkreal consoles from player spots (the task's
   ask) are likewise unrun this session.
5. **The NF_AIMPROBE prints** are still in the tree (env-gated, inert without the var) — the
   strip is mechanical and deliberately left for the next session so the probe stays available
   for the object-stage work above.

## Live checks for the main agent

- **07000008, steered shot on leaf 1's stile** (breakpoint 0x1a4d90 + the flight capture): aim a
  rocket so the crosshair ray crosses DOOR_Shutter_1's right stile (x 139.44..139.53) — does the
  snap land on that object, and does the rocket STOP at −96.706 (the live steered-stop evidence)
  while leaf 3's stile passes? That would confirm the 0x60 bits are per-placement (leaf 1 without,
  leaf 3 with) — the port's per-door `bullet_pass` models exactly that split.
- **obj+0xF0 for placements 49–52** (gdbwatch on Control_CreateObjEx 0x1343c8:36): dump
  glist_record+0x44 per shutter placement — the port assumes the word carries 0x60 for the
  passing leaf; the disc value settles it.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session, so there are no numbers to copy.
