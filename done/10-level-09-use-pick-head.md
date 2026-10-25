## Your task: `level-09-use-pick-head`, the use key picks from Bond's head, not the render camera

`interact_07000009_live::the_load_point_e_route_exits_with_objective_2_incomplete` is ignored: the port's use pick runs from the RENDER CAMERA (scene.rs passes `camera` as the head), while the game's `Player_Activate` 0x1a5170 builds it from `Player_GetHeadPos` [M :101-106 — the use point = head + view]. At the 07000009 lift doors the camera hangs ~2.5 behind Bond, so the 1.0 use sphere never spans the exit box (z 10.33..12.49 [D LOAD_LEVEL #63]) and the E press never fires.

1. Make the use pick originate at the game's head position [M `Player_GetHeadPos`, `Player_Activate` 0x1a5170:101-106] for every use (doors, switches, pickups, ladders, wall-hug, lifts), not the camera. Search for the other callers that pass the camera as the head and fix those too.
2. Un-ignore the test. Every interact_* and walkreal_* test green on master must stay green (several walks press E; re-derive any pin that only held because of the camera origin, with the evidence).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-09-use-pick-head`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-10-24T22:51Z

---
## Report (done, 2026-10-25T00:08Z)


# level-09-use-pick-head — the use key picks from Bond's head, not the render camera

Branch `s/level-09-use-pick-head` (from master 7dbc9cb1). All times UTC 2026-10-24.

## What was done

1. **Every `Player_Activate` arm now takes Bond's head explicitly.** `scene.rs`'s E-press block
   passed `run()`'s `camera` parameter to `player_use_doors` as the doors/trigger/monitor origin
   (scene.rs:4836 on master) and copied it again for the SS/gunimp pick (master :4848). Both now
   pass `self.bond.head()` [M `Player_Activate` 0x1a5170:100-108 — the one use sphere is built at
   `Player_GetHeadPos` + the viewer direction; `Player_GetHeadPos` 0x195aa0]. `net_use_pass` (MP)
   already used the body's head; the SP/switch/PCQWorm arm has taken `sp_head` since
   garage-lift-calls; the creepwall arm scans by `bond.pos` [M 0x1a5170:66-97, no capsule]. The
   free-cam harness `step()` binds the real camera but hard-passes `use_key = false`, so the E
   path can never regress to a camera origin.

2. **The "render camera" story was a misreading, and is now corrected in the code.** `run()`'s
   `camera` parameter is `step_bond`'s `self.bond.head()` binding (scene.rs:3228 → :3256, no
   reassignment or shadowing inside `run()`), and that has been true since before the
   garage-lift-calls commit (verified against `fbce5c7c~1`). So at runtime the doors/trigger/
   monitor arms were ALREADY head-based; the change makes it explicit and kills the
   misnamed-arg trap. The false comments (garage-lift-calls' "the camera hangs ~2.5 behind
   Bond" in drone.rs, and the same story in the test's ignore note) are rewritten. Verified by
   probe: with Bond pressed in to z 13.85 at the 07000009 lift doors, the head-origin use
   sphere (use point z 12.85, r 1.0) spans TOUCH_ONCE #67's box (z 10.33..12.49 [D placement
   #67]) while a camera 2.5 behind does not — the mechanism the task described is real
   geometry, but it was not the code's runtime behaviour.

3. **The ignored test is un-ignored and rewritten to what the game proves.** The old premise —
   an E press at the CLOSED lift doors loads 0700000a with objective 2 incomplete — is
   contradicted by the decompile + the disc, three independent ways:
   - `Trigger_Activate`'s type-1 (TOUCH_ONCE) arm returns before any write without `mask & 4`
     [M 0x1b2318:38-40], and #67's disc mask is 8 [D placement #67 params (0,50),(1,8)]. The
     head-origin sphere DOES reach #67's box here, and the game still refuses it — so the box
     the old ignore note called "the exit box" was in reach all along and was never the blocker.
   - The lift doors are SS objects; `SS_Activate` needs `ext+0x1E & 4` [M 0x1a9ea8], and their
     disc mask (param 2) is 1 [D SS #3/#2 params]. E on the doors is refused as well.
   - The Load Point's own pick shape is its collision glist — a thin plate at z 10.232..10.283
     [D placements dump #63, object "Load Point", collision 12], ~1.6 beyond the pressed-in
     sphere (z 11.85..13.85); the game's refine is mode 0x101 → `Intersect_PointGeom` — the
     point IN the glist [M `Collide_Intersect` 0x1ebaa8], stricter than the port's AABB
     stand-in, which already misses.
   The level doc's route (b) reading ("pressing E on the Load Point loads 0700000a immediately",
   with "the exit box z 10.33..12.49 [D #63]" — that box is #67's) does not survive this
   evidence; docs/levels/07000009.md needs the same correction (left untouched per the
   shared-files rule — for the driver).
   The rewritten test `the_load_point_e_press_at_the_closed_doors_fires_nothing` pins the
   proven behaviour: after the same walk-in + 90 E presses, `level_exit` stays None and ch
   0x32/0x39/0x46 stay off, each assert carrying the [M]/[D] citations above. The exit itself
   (doors open via the panel, walk-in touch #67 → ch 0x32 → #63's gate auto-fires [M
   `Trigger_Update` 0x1b1a78:162-166]) stays pinned by walkreal_07000009, with objective 2
   complete — an obj2-incomplete exit would need the doors open without the panel, and no disc
   path writes ch 0x39 otherwise.

## How it was verified

- `interact_07000009_live`: 15 passed, 0 failed, 0 ignored (was 14 + 1 ignored).
- The E-path regression surface, run directly (this sandbox kills background processes between
  tool calls, so the fast runner cannot survive a run — see below): garage_lift_calls,
  kiko_button_real_walk, level_end, exit_aim_hint, fix_swim_vent5 (13), interact_07000001..4a
  (61, 2 pre-existing ignored), interactables_0700000a..41 (92, 6 pre-ignored), walkreal
  01/02/04/05/06/07/08/09/0a/0b/0c/0d/11/11_route/12/13/14/15/16/1b/41/4a/4a_satchel (65, 6
  pre-ignored), nf-view --lib (96).
- Crates: nf-game 886 passed / 0 failed (drone.rs delta is comment-only: zero non-comment lines
  vs master), nf-formats + nf-audio 133 passed / **3 failed — exactly the 3 `disc_golden`
  golden-manifest tests the task brief allows to fail on the slim ISO** (nf-formats is untouched
  by this branch), nf-view 341 passed / 0 failed / 14 pre-existing ignored.
- **Tally: 1360 passed, 3 failed (the allowed disc_golden trio), 20 pre-existing ignored.**
- Fast runner note: `tools/test/suite.py` collected **789** test binaries on this tree (cargo's
  `--message-format=json` stream emits duplicate artifacts; the collector does not dedup by
  name — the 2026-10-24 21:17 run in `../suite.log` logged 640 distinct). The full ordered run
  is ~74 min (per-binary times in that log), which does not fit this sandbox's killed-between-
  tool-calls background model plus the 2 h window; the crates were therefore run directly in
  the foreground with `--test-threads=1`, per-crate totals above. Suite totals are per-call
  sums, not one suite.py total line.

## What's left

- A full single-pass suite.py run on a tree where it fits (and a dedup of its 789-binary
  collection).
- docs/levels/07000009.md's route (b) bullet + the elevator-activation section still carry the
  old E-route reading (shared file; not edited here).
- The use sphere's glist refine (`Intersect_PointGeom` mode 0x101) remains the port's documented
  AABB stand-in; a mesh veto of an AABB hit is still a live-check item.

## Live checks for the main agent

1. Falsification check for the rewritten test: PCSX2, fresh boot into 07000009, walk Bond into
   the closed exit-lift doors (he should stop at z ≈ 13.85), press E 90×. Expected per [M]
   0x1b2318:38-40 + [D #67 mask 8] + [M 0x1a9ea8] + [D SS #3/#2 mask 1]: NO load, no channel
   change (watch switch_channels 0x32/0x39/0x46). If the game DOES load 0700000a there, my
   mask-gate reading is wrong and the test rewrite must be revisited.
2. Reach pin for the origin rule: same stance, single E press, breakpoint `Collide_SphereIntersect`
   (0x1ec940) from `Player_Activate` — expect the sphere centre = head + view (head = obj+0x30/34/38
   with the +0x908/−0x90c y rule) and radius 1.0 [M 0x1a5170:100-108].

## Usage

not visible to me. This environment shows no token, credit or context-window figures for the
session (no usage display is exposed to this agent), so there are no numbers to copy.
