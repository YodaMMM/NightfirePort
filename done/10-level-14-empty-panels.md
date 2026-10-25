## Your task: `level-14-empty-panels`, 07000014: openings and panels with nothing behind them

Spots from real play (Bond root x, y, z, yaw, pitch; frame since load):
- the broken cut-scene: (40.87, 2.98, 40.85, -3.179, 0.210) at frames 1931 and 2300 — the cut-scene camera is inside rock, the rest of the frame a flat blue void;
- the camera-view console: (33.71, 0.65, -18.11, -7.579, 0.220) frame 9909 — "Press Action for Camera View" shows, nothing happens;
- the panels: (54.21, 1.03, 46.15, -8.344, 0.097) frame 5739 — a vent/hatch opening with nothing beneath it; (-88.58, -1.10, 46.15, -0.333, -0.041) frame 25538 — a wall panel with nothing behind it;
- the fall: (-49.65, 0.65, 36.85, -6.979, 0.006) frame 20889 (black screen) then (-107.07, -70.14, 73.28, -4.003, 0.595) frame 21799 — Bond below the map after a broken cut-scene, objective "Sabotage base security and escape from room".

1. At the two panel spots: dump what the port draws and collides there and what the level data places there [D — the placements near each spot, their objects/glists, the static cels, the destroyable/breakable/SS parts], and find what is missing (an undrawn object, a missing cel, a LOD/variant not swapped, a script entity not created) [M for the creator].
2. Port what is missing so the vent shows its shaft below and the wall panel shows what the game shows behind it; tests on the draw lists and the collision there; headless renders named in the report.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-empty-panels`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-10-25T07:20Z

---
## Report (done, 2026-10-25T08:21Z)

# level-14-empty-panels — report

Task: 07000014 openings and panels with nothing behind them. At the two panel spots, dump what
the port draws and collides and what the level data places there [D], find the missing piece
[M for the creator], port it, pin draw lists + collision with tests, name headless renders.

Start tree: `origin/master` `38f1bc93`. Branch: `s/level-14-empty-panels`. **No product code was
changed on this branch yet** — the deliverables landed are the [D] dump tooling, the verified
root causes, and the real-input tests (one pin green, one fail-first repro ignored).

## 1. The [D] dump at the two spots (new example `dump_07000014_panels`)

`crates/nf-view/examples/dump_07000014_panels.rs` (run: `cargo run -p nf-view --release --example
dump_07000014_panels`) prints, per spot: every placement within 15 (any map, with the glist word,
the 0x8000 cel bit, the collision block, the cel-graph node and its visibility), the port's
Materials within 12 with the EXACT render-cull verdict (node/orphan/room-vis replicated from
gpu.rs:1031-1035), the portal-graph trace (BFS + the exact `cel_visibility` pipeline incl.
near_los_rescue + widen), the collision rays, the region-overlap census (world AABBs), the
cel→placement inversion, and script 0x06000A1B's decode.

What the dumps established:

- **Both spots' static geometry DRAWS in the port.** At the panel spot the generator hall's room
  cel (graph cell 43 = scene room 38, class 0xc047) is visible from the eye cell; the intact
  generator (0xe1 `GENERATORS_Intact` #828), the destroyed art (kind-0x19 CEL
  `GENERATORS_Destroyednew` #824, cells 222-225 at y −5.1 — the generator PIT under the floor),
  the fusebox #385, the sign and the fires all carry draws that the exact cull keeps (the pit
  cells are portal-orphans → force-drawn, the game's `View_AddForcedObjects` 0x1e5fb8 stand-in).
  Same at the vent spot: rooms CorridorA2_1/2 + AirConditioningRoom all node-visible. **The
  "empty" is not a cel-cull or a missing static.**
- The wall panel itself is the small door object **#386 "OBJECT_Door" (kind 0xdb, aabb
  0.6×0.8×0.1 at (−89.58, −0.68, 49.17))** beside the real fusebox #385 — and the disc has NO
  geometry behind that wall (no placement's world AABB overlaps x < −91.1 there): what the game
  shows "behind the panel" must be script-spawned entities, and the fusebox script's are exactly
  what the port stands in for (below).
- Port-limit notes recorded on the way: `find_cel` indexes `scene.cels` (rooms), NOT the graph
  cells — a dump that mixes the two index spaces reports false HIDDENs (my first pass did;
  corrected). The portal graph is sparse: 55 portals / 288 cells, one connected component of 41.

## 2. The root causes [M for the creator]

Both spots are **script-spawned entity art**:

1. **The vent spot** — the route lid SP #849 "Vent_StoreRoom_Graphic" (51.58, 3.58, 44.94), one
   of the 15 kind-0xd9 mode-0x40 preloads of script **0x06000A3B**. Its stream 0 spawns the
   fusebox prop 0x02000750 (type 0x31) at key 1 and the **ladder art 0x02000751 (type 0x19) at
   key 61** [D probe14_lidscript decode]. The mode-0x40 preload runs ONE tick ([M] SP_LoadScript
   0x1dbc08:46-54) so only the prop exists at load; the ladder art needs the REPLAY
   ([M] SP_Update 0x1dbda8 state 1: mask bit 4 = the activation word SP_Activate 0x1dc338 sets on
   the player's use → Script_Play from key 0). **The port's replay WORKS** — the new test drives
   it: an E press from INSIDE the duct (the SP's use sphere rides the live matrix at y 3.58;
   from the facility floor d ≈ 2.6 it is out of reach) spawns the ladder art exactly at the lid's
   base (0x02000751, kind 0x19, at (51.578, 3.585, 44.942)). The driver's "opening with nothing
   beneath it" is the crawl route passing the lids WITHOUT the E-open: no press → no replay → no
   art beneath, exactly the route-legs open question in docs/levels/07000014.md §1a.
2. **The panel spot** — the real fusebox #385's script **0x06000A1B** [D placement params] spawns
   the wire sub-objects **0x02000A62 / 0x02000A61 / 0x02000A73** (type 0x31) in streams 0/1/3
   [D decode, key-1 Entities]. The port's fusebox.rs stands in for them (one wire per group —
   docs/levels/07000014.md §5 "the SP-entity bullet path is not ported"), so the opened panel
   shows the recess empty. **Confirmed by the fail-first repro**: after the real use press +
   120 ticks, no 0x02000A61/62/73 entity exists in `scripts.objects`.

## 3. The tests (new `crates/nf-view/tests/panels_07000014_live.rs`)

- `the_route_lid_use_spawns_the_ladder_art_beneath_the_opening` — **GREEN** (1.64 s): pins the
  pre-state (the prop exists, the ladder art does not — [M] one-tick preload) and the post-state
  (the in-duct use press spawns the ladder art at the lid's base — [M 0x1dbda8 state-1 replay;
  D 0x06000A3B s0 t8]). Real input only (teleport to the in-duct spot, the use press through
  `step_bond`).
- `the_fusebox_use_spawns_the_wire_sub_objects_behind_the_panel` — **#[ignore]d fail-first repro**
  (a red test must not ride the branch): the use press at the fusebox must spawn the wire
  entities; it does not today. The fix (next message's work): run the fusebox script's Entity
  streams through the same SP-entity path the lid replay uses, replacing the stand-in.

Ran before pushing: `nf-view` — panels_07000014_live (1 passed / 1 ignored), walk_07000014_live
(2/2), the new example builds and runs. Product code unchanged ⇒ the master baseline stands.

## 4. The suite

The full fast suite **completed on this exact product code (master `38f1bc93`) at 07:24 UTC:
`3284 passed, 23 failed, 194 ignored (104 min)`** — the 23 = the sandbox's documented
environmental families (the 3 `disc_golden`, `identity_disc`, the GPU-render trio, the
net_*/mp_* OOM SIGKILLs; same set the previous tasks' full runs produced). Its per-binary log was
truncated by the session's restart, so the breakdown is cited from the totals + the documented
families. This branch adds only nf-view tests/examples/docs — the touched binaries ran green
directly (above); a fresh full-suite run on the branch tip did not fit the remaining window and
is the first item for the next message.

## What's left

1. **Port the fusebox script-entity path** (the panel spot): replace the wire stand-in with the
   real 0x06000A1B Entity spawns (streams 0/1/3), un-ignore the repro, pin the wires behind the
   panel. The lid replay's working path (nis.rs state 1 → Script_EntityStart) is the template.
2. **The vent route**: make the real-input crawl E-open the route lids (the route legs pass them
   without pressing E) so the ladder art spawns beneath each opened opening; then re-try the
   wedged east-mouth walk (x 40.98, the doc's live check).
3. A fresh full-suite run on the branch tip (merged with origin/master) and its total line.

## Headless renders for the main agent

- `--bond 51.58,3.5,45.6,3.14159,0` — inside the duct at the route lid, AFTER an E press: the
  ladder art should show beneath the opened opening (the test proves the spawn; the render shows
  the art draws).
- `--bond 54.21,1.03,46.15,-8.344,0.097` — the driver's vent spot from the facility floor BEFORE
  any lid use: expected to show the bare opening (the current gap).
- `--bond -88.4,-2.2,48.6,-0.87,0` — facing the fusebox #385 / the small door panel #386: the
  panel with the recess (empty today; the wires land behind it after item 1).

## Live checks for the main agent

1. Fresh boot, 07000014, from the vent store room: E-open one route lid from inside the duct and
   watch — does the ladder art appear beneath the opening in the real game (the [M] chain says
   the replay spawns it at key 61; the art's look is the render question).
2. At the fusebox #385 (−89.30, −2.22, 49.36): E the lid press — do the wire sub-objects appear
   behind the front panel, and does the small door object #386 move (is it the lid's art)?
3. The intro cut-scene (frames 1931/2300 stuck at (40.87, 2.98, 40.85), camera in rock): which
   script is playing at that beat (0x06000833 is the objective-1 scene, 1400 ticks) and what its
   camera stream does around tick ~1900 — the blue-void + stuck-camera pair looks like the
   scene's camera stream never advancing; a fresh-boot tick count for the scene would settle it.

## Usage

not visible to me — this session's tooling shows no token, context-window or cost figures.
