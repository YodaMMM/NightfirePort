# level-16-every-objective — 07000016 end to end on real input

## The inventory (what the mission data lists vs what the tests covered)

07000016's MissionData record lists TWO objectives [L record 22 @0x2a46c0, objtable 0x2a42f0 ×2;
matches `mission::c07000016::DEF`]:

1. **obj0 "Reach Drake's launch silo"** (complete ch 0x8f, enable 0x8e) — **was already fully
   covered on real input**: `walk_arrival_to_the_chasers_spawn` walks a fresh load (the carried
   07000014/15 RamSave [D], the real mechanism, no channel pokes) through the corridor, all five
   doors (the security-station E and the three panel Es through the real use path), onto the silo
   plate #253, and asserts the monitor folds it (`states[22][0] == 4`) and obj1 reveals
   (`states[22][1] == 3`). The carried 07000015 record completes at the arrival plate
   (`states[21]` all 4). Stand-ins carried by the file: the walker's `top_up_health` (the
   documented "enemies disabled for the walk" allowance) and — in `fix_finale_live`/`kiko_button_real_walk_live`
   only — the documented [S] pack-flight re-seat (a single `bond.pos` write standing in for the
   jetpack; the walkreal file itself has NO pos writes).
2. **obj1 "Prevent Kiko from launching into space"** (complete ch 0x90, enable 0x8f) — the disc's
   only writer is the KikoDeathButton #693 (kind 0x29 switch, p0 = 0x90, gate 0x8d) under Bond's E
   [D hash 020008c1; M Switch_Activate 0x1b02a8]. **Not reachable on real input yet.** The
   completion mechanics ARE pinned: `kiko_button_real_walk_live`'s gate-off sister completes the
   whole mission (ch 0x90 → mux #618 → ch 100 → MissionState 6 → 0x7300017 [D; M 0x182c60:111-112]).

## The two blockers (exact, with evidence)

1. **U-051 — the pack (jetpack) flight.** The launch chain re-enables Bond INSIDE
   HurtPoly-ShuttleLaunch #309's box at the pit floor [M `Player_Enable` 0x195498 teleports to the
   stream-8 key; D the pack cinema 0x06000631's single t747 key (3.57, −22.76, −179.55); the
   volume x −30.7..35.3, y −24.1..−21.3, z −197.3..−133.8]. The launch script raises ch 243 at its
   tick 1611 and drops it at 2170 [D stream-6], and the chaser spawner write (ch 0xf0) lands at
   tick 2180 — AFTER the pulse [D] — so even the fight cannot precede it. The pit holds no walking
   escape before the pulse: its door (13.9, −23.1, −166.4) locks on ch 240/241, no ladder/wire
   placement exists, and the lift summons after the pulse [D + the ch-80 writer census; this
   session's geometry probes agree — the gallery is walkable end to end but dead-ends at
   (21.3, −22.0, −137.9) and the fall zone at (2.4, −137)]. The game's escape is the pack's own
   flight inside the ~861-tick control window — the port enumerates no jetpack substate
   [M player.rs consts + the symtab's Player_SS*]; the flight's controls/physics are the one
   unverified piece (the report's live checks). Implementing it is the remaining port gap.
2. **The #616 gate.** The mid-corridor TouchOnce #616 (flags 0xEA, mask 8 = the player, volume
   x 0.36..4.54, z −146.65..−144.58 — the full walkable width of the only route south) latches
   ch 0x8d, and nothing on the disc clears it [D: every script write in 07000016.bin +
   07800016.bin decoded; Init_SwitchChannels runs only from the level-load path]. The switch
   refuses while its gate is on [M 0x1b02a8:22 — `ext+0x14 == 0 || switch_channels[ext+0x14] == 0`],
   so the WALKED press is inhibited — which the port pins faithfully. Whether the GAME's contact
   pass registers #616 for the walking player at all (its Trigger_Hit is contact-list-based on the
   trigger's glist bounds; the port's on the target object's collision tris) is the open
   game-side live check in docs/systems/kiko-death-button-gate.md.

## What this session added

`walkreal_07000016_live::the_walked_mission_ends_at_the_launch_blast_until_the_pack_flight_is_ported`
(un-ignored): the WHOLE mission walked from a fresh load on real input, no teleports, the walk
allowance only — the carried record completes, obj0 completes on the route (asserted), the launch
chain re-enables Bond at the pit floor (frame 6071 on the recorded run), ch 243 rises at +1543
(the [D stream-6 1611 / port 1603 / live 1591] model), and the mission FAILS at +451 after the
pulse (the death-sequence window [M 0x19a6a8 RED_FADE_TICKS 448; L 454]) with obj0 complete and
obj1 still waiting — the wall is machine-pinned, not narrated. All 8 un-ignored tests in the file
pass (37.4 s); the one ignore (`walk_the_chasers_defeat_completes_the_mission`) keeps its exact
reason (U-051 + the gate, cited above).

## What I ran

- `walkreal_07000016_live` (the binary the task touches): all 8 un-ignored tests green, direct
  run, `--test-threads=1`, NF_ISO set. No src/ changes this session — the tree differs from
  origin/master only in this test file + docs.
- The chunked full suite (this sandbox kills background jobs between tool calls; state
  `/home/z/suite16eo_state.json`): INTERIM — the deadline stopped it early. The remaining
  binaries resume with `python3 tools/test/chunk_suite.py /home/z/suite16eo_state.json --budget 480`
  (the build is warm). Environmental reds seen on this sandbox's prior trees (expected if
  reached): `mp_stress_explosives_live`/`mp_online_failures` rc −9 (4 GB OOM),
  `probe_pylon_render` (wgpu `adapter: NotFound`), the 3 `disc_golden` on the slim ISO.

## What's left

- Port the pack (jetpack) flight substate [M the substate family] with a live check of its
  controls/physics, then re-route the walked mission through the escape; settle the #616 game-side
  question with the PCSX2 watch; then the walked end-to-end can assert ch 0x90 → 100 →
  MissionState 6 → the 0x7300017 hand-off (the gate-off shape already proves the tail).
- The remaining suite binaries on the final tree (resume command above).

## Live checks for the main agent

1. U-051 closure (already drafted in docs/systems/hurt-volumes.md §U-051): fresh boot of
   07000016, play to the pack pickup; watch `switch_channels[243]` (0x26fc90+243·4) and the player
   position (obj+0x30) from the cinema's end to the launch. Expected: the re-enable pose
   (3.57, −22.76, −179.55), the player RISES out of the pit before ch 243 rises, the volume never
   deals damage. The position trace IS the flight's physics (thrust/turn/fall rates) the port
   needs.
2. The #616 gate: fresh boot, walk z −144..−147 in the mid-corridor, watch
   `switch_channels[141]` (0x26fd1d). 0→1 = the arm is real (the golden path cannot press the
   button — a shipped-game quirk to mirror); a steady 0 = the game's #616 bounds differ from the
   port's tris and the walked press should complete (then the port's `triggers::build` tris for
   #616 are the bug to fix).

## Usage

not visible to me
