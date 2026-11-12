# level-0a-civilian-slide-cels

Branch `s/level-0a-civilian-slide-cels` (from origin/master 08a276ce0).

## What's done

**The fix.** The sim-side drawn gate (`Drone::drawn`, the port's `obj+0xFC` stand-in) now runs the
game's own room rule instead of the bullet-segment ray stand-in:

- `crates/nf-game/src/celgraph.rs` (new): the cel graph types + the game's portal-visible walk
  (`CelGraph`/`CelNode`/`PortalEdge`, `Vision_InView` 0x1e89e0's signed-distance form, the
  `Vision_Recurse` 0x1e82e0 chain walk with the door leg, `ClipPortalWithPlane` 0x1e7fa0, the
  VisCelList `QuickSort` 0x1d30a8 comparator 3) — moved VERBATIM from `nf-view/src/scene.rs`
  (nf-game cannot depend on nf-view). `scene.rs` re-exports every public name, so the renderer's
  draw path is untouched (no draw changes).
- `crates/nf-game/src/drone.rs`: `Drones::tick_with_cel_graph` (nf-view lends the Sim's
  `cel_graph` clone at the tick call site). Once per tick it builds the room mask
  (`drawn_room_mask`): the portal walk from the viewer's linked cel (`player.obj_cel`) with the
  depth-0 planes built from the sim's own frustum (`frustum_sides` normals anchored at the eye +
  the `fw·p ≤ fw·eye + 200` near-back plane, gpu.rs `depth0_game`'s five) and this tick's
  `Doors::closed_flags` (the `Vision_Recurse` 0x1e82e0:65-70 `Door_IsClosed` leg). Per drone:
  drawn = enabled ∧ (drone's `obj_cel` ∈ mask) ∧ `in_view_of` (the object's own `Vision_InView`
  frustum leg, `View_AddObjects` 0x1e62d8:34). **No rays.** Where the walk cannot run (no graph /
  no portals / the viewer's or the drone's cel unresolved) the gate falls back to the previous
  bullet-ray stand-in per drone — the hand-built test sims (`tick_inner`, `Player.obj_cel = None`)
  and cel-less worlds keep their exact prior verdicts.
- Evidence: `View_AddCels` 0x1e6ba0 draws each VisCelList cel's object list; `View_AddObjects`
  0x1e62d8:71 writes obj+0xFC = 2 behind `Vision_InView` 0x1e89e0 — no rays in the pair [M all].
  The task's measured cause [L sim run per the task brief]: a civilian in a visible room behind a
  thin post (LaserTrackVert mat 0x06) / non-pass foliage read undrawn and slid at danim speed with
  no gait (`NDrone2_DoAnimation` 0x149f88's not-drawn arm).

**Tests run (all on the final tree, NF_ISO on):**

| binary | result |
| --- | --- |
| nf-game `--lib` | 787 passed, 0 failed, 4 ignored |
| nf-view `walkreal_0700000a_live` | 13 passed, 0 failed, 2 ignored (pre-existing ignores: the 0c corner-crossing + glide-height live checks) |
| nf-view `window_guard_0a_live` (the lounge-window patroller through the z −4.71 glass pane: drawn + gait; sister undrawn facing away) | 1 passed |
| nf-view `sight_frustum_live` (guards 158/161 drawn at the start view; the look-up rejection) | 2 passed |
| nf-game `ai_live switched_guard_patrols_like_the_game` (the 05-castle opaque-occluder pin: the guard stays undrawn behind the switch-trace sightline; the observer-at-guard sister stays drawn) | 1 passed |
| nf-game `guards_0c_live patrol_trace` | 1 passed |
| nf-view `walkreal_0700000c_barrelrow_live` | 1 passed |

**Not finished (the sandbox deadline hit):**

1. **`walkreal_0700000c_eastroute_live` `the_ground_route_crosses_to_the_east_side` FAILED** on my
   tree (walker STUCK at walker_b2.rs:720). I did NOT get the master A/B done (the stash-based
   check aborted with two minutes left), and the 0c walkreal sisters were mid-flux on master when
   this task started (the ignored `walkreal_0700000c_live` real-damage sisters). **Needs a
   master-vs-branch A/B before merge** — reproduce with
   `NF_ISO=... cargo test --release -p nf-view --test walkreal_0700000c_eastroute_live -- --test-threads=1`.
   If it is my change: the drawn gate flipped on a 0c guard whose cel the room walk now
   covers/excludes differently than the rays did — the report's room rule says the game's answer
   is the mask, so re-derive the test's guard first.
2. **The vantage walk test `civilian_slide_0a_live` is `#[ignore]`d, not green.** The real-input
   walk from the load to the vantage (4.59, 0.65, 2.43) cel 29 works (the arrival plate → out of
   the cab → lobby legs are walked; the east side sits behind the mission ring — the z 15.3 band
   walls at x 6.5 [walked], so the test uses the merged `window_guard_0a_live` stand-in pattern:
   transit to (4.59, 0.65, 5.5), then a real walk to the vantage). The watch's subject,
   **placement #695, reads `present == false` at the load** — a spawner-held template (this run's
   roster dump; the load log holds 26 character meshes back by spawners), so the test cannot yet
   pin "in-view civilian on danim 392 with drawn == false" (red on master) / drawn + gait (green).
   What the run DID establish: at the vantage Bond's cel resolves (476), the room walk runs
   (4 cels visible from it) and the frustum leg sees the lane. Follow-up: release #695's spawner
   the way the walkreal route does (the route passes his release before frame ~7485), then un-ignore.
3. The fast runner `tools/test/suite.py` was started twice with `nohup` and **died instantly both
   times in this sandbox** (0-byte `suite.out`, no process alive — background jobs do not survive
   between tool calls here, unlike the download warning). The table above is the foreground subset
   I could fit; the full suite (all `walkreal_*` binaries, the 3 allowed `disc_golden` fails on the
   slim ISO) still needs a run on the final merged tree.

## Live checks for the main agent

1. `tools/trace/fieldprobe.py` on 0700000a, the vantage (--bond 4.59,0.65,2.43,<yaw to the west
   lane>, level pitch): watch obj+0xFC of the atrium lane civilian (placement #695) while he
   crosses the lane — expect 2 whenever his cel is portal-visible from Bond's cel, even with a
   LaserTrackVert column between.
2. Same spot: confirm the danim-392 gait plays (no 0.02/tick slide) while in view.
3. Behind the lounge window (the window_guard spot), the same +0xFC read through the glass pane
   (mat 0x10) — the port already pins it; the game read confirms the room rule through glass.

## Usage

`not visible to me` — this session's environment shows no token, cost or context-window figures
to copy.
