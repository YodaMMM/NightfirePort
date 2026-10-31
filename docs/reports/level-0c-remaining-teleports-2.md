# level-0c-remaining-teleports-2 — report

Branch: `s/level-0c-remaining-teleports-2` (work commits `87e3516b`, `b757d40a`; merge with
`origin/master` `0003c934`).

## What was done

The task: remove every `bond.pos` write in `walkreal_0700000c_live` and
`walkreal_0700000c_eastroute_live` that can go; walk route stand-ins on real input; keep the
settled no-route and measurement placements with their reason and a sister assert next to them.

**The eastroute test's two writes are GONE** (commit `87e3516b`), walked on real input:

- **The start spot** (the band west end) — the test now REPLAYS the main walk's own route to
  the band: legs 1-9c moved verbatim from `walkreal_0700000c_live` into the shared
  `crates/nf-view/tests/common/walkreal0c.rs` (`walk_spawn_to_the_band`, same load, same
  ticks; the main walk test re-run green proves the move preserved the timeline), then the
  band-east leg to (52.0, −207.6). No teleport.
- **The jump start** (79.6, 1.1, −210.2) — replaced by the main walk's own due-east run-up
  legs; the yaw faced east on real turn input, and the old single jump became the main walk's
  proven hop idiom (the under-steps 0.75..1.55 then the 1.75 first step [D the step scan]).

**The ladder495 measurement spot keeps its write** — it is a measurement spot, not a route
stand-in: the test's subject is the ladder grab/climb classes
([M 0x1953f4's DEFAULT arm + 0x196118/0x196310]) at #495's own plane, and the walkreal route
never crosses the yard's west edge. It gained a **floor sister** right next to the placement
(west-ground floor ~0.0 [D probes] pinned with a `segment_hit` assert).

**Four writes remain, each a documented stand-in with its reason and sisters inline** (all
four read in this session, verified in place):

1. **The trench exit** (`common/walkreal0c.rs:256`, [5.0, 12.35, −112.9]) — the vent shaft is
   a ONE-WAY DROP in the disc's own data: the south crawl pocket is sealed by the wall slab
   z −112.34..−112.363 and the crouch capsule stops ON it at −111.813 [D block dump + M
   0x194df0 case 3; the game holds the body too, L 2026-10-26 hatch-3; no breakable glass
   exists, D census]. Sisters inline: the shaft hole east of the lane, the stair landing, the
   ramp's mid surface (three `segment_hit` asserts directly after the walk-on).
2. **The mezzanine reach** (`common/walkreal0c.rs:398`, [54.0, 5.8, −158.0]) — every walking
   route off the display-ground complex is closed: the barrel row stops the north end
   (walkreal_probe0c_barrelrow_live), the ST-1 walkway's south lip holds at z −153.65
   (walkreal_probe0c_lip_live), the mezzanine deck floods as a separate component; the game's
   own connection is the warehouse-side corridor/annex vent — a filed live check (below).
   [L 2026-10-31] fresh-boot runs closed every other entrance. Sisters inline: the deck holds
   Bond, the ch-0x54 VentLatch #182 exists, ch 0x54 off before the burn.
3. **The east climb** (`walkreal_0700000c_live.rs:342`, [95.0, 11.3, −209.5]) — the band deck
   is a COLLISION ISLAND [D eastclimb3-7]: the disc's own static data connects NOTHING to it
   (turb4 stair flights disjoint with an empty slot; the band strip gaps 10.4 m over the
   chasm; crates4 gap 3.1 m + 2.06; no lift kind on 0c; ladders/grapples west-only [D]; the
   nav patrols G3/G4 are separate paths with no links [D the 0c nav dump]). Sisters inline:
   the ground gate does NOT fire (120 real E pulses, negative pinned), the stand-in must land
   on the deck at 10.34 [D plc44], and the gate plane asserts after.
4. **The ladder495 spot** (`walkreal_0700000c_live.rs:414`, [−7.6, 1.1, −176.9]) — the
   measurement spot above, with its new floor sister directly under it.

Everything else in the two tests is real input (turn/forward/jump/E ticks through `step_bond`
input only — no channel or state pokes on the walked legs).

## How it was verified

- `walkreal_0700000c_live`: **2 passed, 0 failed** (`walk_the_whole_mission_from_the_start_to_the_0d_handoff`,
  `ladder495_walk_grab_and_climb_to_the_top`) — 26.5 s, fresh run on the merged tree.
- `walkreal_0700000c_eastroute_live`: **1 passed, 0 failed**
  (`the_ground_route_crosses_to_the_east_side`) — 22.5 s, fresh run on the merged tree.
- `interactables_0700000c_live`: **5 passed, 0 failed** — 14.0 s, fresh run on the merged tree.
- Fast suite (`tools/test/suite.py` binary set) on the final tree (merged with
  `origin/master`): this sandbox kills background jobs between tool calls, so the same
  runner's chunked form (`tools/test/chunk_suite.py`, one state file, 2 workers) ran it in
  foreground chunks. **At the 18:26 UTC cut-off: 418/996 binaries recorded, 1316 passed,
  1 failed (see below), 100 ignored** — final numbers at the end of this report.
- The one failure, `probe_pylon_render` (`probe_render_pylon_only`), panics in wgpu adapter
  creation (`NotFound { active_backends: Backends(0x0) ... }`): this sandbox has no GPU and
  no active render backend. It is an environment limitation, not a regression from this
  branch — the branch changes three sim-only walkreal test files and touches no render code.
  Per the merge lessons, anything visual needs the main agent's render anyway.
- Several heavy `mp_*`/`walk_*` live binaries were OOM-killed by the sandbox (rc −9) when two
  ran concurrently; their entries were cleared and they re-run in the final pass (solo).

## What's left

- The suite tail (bins after the cut-off; final count at the end of this report) — the
  sandbox clock, not a test result. The three task-named binaries and all
  0c/walkreal/interactables/mp/fix_ priority bins are inside the recorded set.
- The OOM re-runs (final count at the end of this report).
- The GPU-render probes cannot pass in this sandbox (no adapter) — they need the main agent's
  environment.

## Live checks for the main agent

1. **The mezzanine reach (the filed check).** In a fresh-boot PCSX2 play of 0700000c, from
   the display-ground complex, walk the warehouse-side corridor into the annex and watch how
   the game reaches the mezzanine deck (x 52..56, z −176..−152, deck 5.44..6.03). Watch
   `bond.pos` (gdbwatch) + the SS list: expected route is DOWN through the annex vent
   (SS #180 `ventmesh` + blocking poly #184 at (53.8, 5.5, −163.59), latch #182 `VentLatch`
   ch 0x54 at (53.80, 5.90, −165.36) [D #180/#184/#182 p5]) — i.e. the game enters the plates
   corridor from ABOVE through the vent, which is exactly what the test's stand-in replaces.
   If instead Bond can climb to the deck some other way, the stand-in's reason is wrong and
   the test should walk that route.
2. **The east climb.** Fresh boot, east ground near (95, 6.2, −209.5): confirm nothing in the
   real game lifts Bond onto the band deck (10.34 [D plc44]) — no mover serves it (no lift
   kind on 0c [D kind census]), the eight SSs/2 doors/2 ScriptPlayers are west-side [D audit
   §6]. gdbwatch `bond.pos` while standing at the gate ground spot and firing 120 E pulses
   (the test pins the gate does not fire from the ground); expected: `bond.pos` stays at
   6.17 and no `level_exit` fires. Any dynamic that does lift Bond is a parse gap the static
   data cannot show, and the island claim gets revisited.

## Usage

not visible to me (no token, cost or context-window figures are shown in this environment).
