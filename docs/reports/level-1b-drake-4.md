# level-1b-drake-4 — 0700001b: the scripted carry into the shafts, then Drake

Branch `s/level-1b-drake-4`, continued from the stub (`origin/s/level-1b-drake-4`), merged
`origin/master` early.

## What was done

### 1. The carry hunt — swept the whole 1b script lib; NO scripted player carry exists

`probe1b_carry.rs` (committed, assertion-free) decodes every script in the level's parsed lib and
sweeps for the three candidates the task named:

- **The player-carry events** [M Script_EventHandler 0x1d94c0]: event 4 = `Player_Disable`
  (:118-121), event 6 = the enable-matrix capture (:125-134) whose `Script_Free` flag-0x100 tail
  re-enables Bond at the captured matrix [M Script_Free 0x1da880:46-49 → Player_Enable 0x195498].
  The entity an event 6 captures is the STREAM's own created entity [M 0x1d94c0:88-96 — the
  SCRIPTINFO entity slot selected by the stream's index byte; an entity-less stream captures
  `glb_players+0x90`, the PLAYER's own matrix]. **The whole lib has exactly one disable/capture
  pair**: `0x0600085c` s7 (disable t0 → capture t1994, substate param 2; SP#147) and
  `0x0600085f` s6 (disable t0 → capture t826, substate param 0) — the two cutscenes, and NEITHER
  stream creates an entity, so both capture Bond where he stood. No stream on the level carries
  an entity through an event 6 — there is no position carry on the disc data the port reaches.
- **The ch-74 pulse** [D 0x0600096e s3/s4: t2 `[74,1]`, t4 `[74,0]`]: no reader exists among the
  level's SP gates (the full live-player census is in the probe dump: no gate 0x4a), the AND
  lists (0x0c←[6,8], 0x09←[0x15,0x16,0x1b], 0x5a, 0x59, 0x5b, 0x5c, 0x13 [D PL#109]), the
  missiles (p4 = the console channels, p7 = 19 [D §7]), the laser (p6 = 14, p7 = 19) or the
  drone wait-switches. The ch-75 pulse's own answerer [M C_CHCHZEROG_Handler 0x216388 —
  `param_4 == 0x4b` → substate 9] has no 0x4a arm; the FLY sibling [M C_CHCHFLY 0x215c70]
  neither. The repeated pulsers (0x06000a1d via SP#273 gated on ch 10 — the Drake fight gate —
  and 0x06000a1f via SP#274) pulse a channel nothing on the disc reads.
- **The iris doors**: the "iris doors" plc #30/#32 are kind **0x31** = `FuseBox_Create`
  [M parsemap_create_dynamic_objects 0x1d04f0 case 0x31] — shootable panels, not movers (the
  placement NAMES say IrisDoor; the runtime kind is the fusebox family, object class byte 0x3c
  [M FuseBox_Create 0x1b0448]). The booth loopers SP#141/#142 (out 0x1c/0x18, script
  0x06000818) create 8 door-model entities (0x02000834-0x3b) — again GFX-attach visuals
  [M Script_CreateEntity 0x1d8968: obj_type 0x31 falls through every creation arm to the plain
  `hashtable_set_object_to_entity_gfx` attach] — and write their own out channels OFF at t550.
  The fly-by chain (SP#211: 0x060007fd creates entity 0x02000823 type 49; 0x060008f0 creates
  8 more, 0x02000919-0x20, then SUBs the ch-75 pulser, the ch-74 pulser, and writes ch 10 at
  t260 [D]) is a cutscene of moving door MODELS, not an opening mechanism.
- **The over-the-top route** (`probe1b_overthetop.rs`, real input): the float rises in the pit
  column between the boundary wall and the slab's scanned west edge and caps at **y 15.91** —
  the island's east face (x 4.983) runs full-height to that cap. The east cell is a closed box
  in the port's collision of the disc's own data (floor −2.4, west wall 4.433 + the inner
  4.983 face, top 15.91/16.74).

Conclusion: the port's sealed-box reading stands on the disc data; **the game's own crossing
past x 4.43 needs a live read** — the checks are in walkreal's ignored chain test (below).

### 2. The kill-channel gap — ROOT-CAUSED: the test's own assert was wrong; the defeat test is UN-IGNORED and green

The observed gap ("the walker-driven kill leaves ch 0x0e off while a raw step_bond kill raises
it") does not reproduce as a kill-path difference:

- `probe1b_killgap.rs` runs the SAME fight twice in one binary — (A) the walker path
  (`Walker::new` + the real `w.shoot`) and (B) the raw `step_bond` path — and BOTH die into
  AstronautDeath 0xbf with **ch 0x0e raised**. The kill path is identical and healthy.
- The real failure is the test's own assert: **the game's own SP out write clears ch 0x0e one
  tick after the defeat NIS ends** [M SP_Update 0x1dbda8: case 3 — the first script finished,
  the trigger starts the next — sets `cVar10 = '\0'`, and the tail (:222-226) writes
  `switch_channels[out] = cVar10 != 0` while the still-on ch 0x0e keeps the mask-1 trigger
  armed]. The NF_CHWATCH=0x0e trace of the test run names it exactly: raised at frame 3160
  (`Drones::tick_inner` — Drake's death-channel write [D param 12 = 14]), re-written per-frame
  by the playing NIS's out passthrough, **cleared at frame 3881** (`nis::Player::update`) one
  tick after the NIS's t720 ch 0x64 = 100. The port matches the game; the TEST contradicted the
  decompile and was fixed (the driver's rule).

**`drake_defeat_by_real_fire_ends_the_mission` is un-ignored and green** — the full chain
through real input (the gate drive + the walker's fire; the arena TELEPORT stays the documented
stand-in for the sealed crossing): Drake spawns via spawner #187 (the gate driven while Bond is
far — the min_dist screen [M 0x137cb0:30]), dies to the walker's fire, the defeat NIS writes
ch 0x0b + ch 100, MissionState 6, to_end **0x0730001b** [M 0x182c60:114-116]. The asserts now
pin the raise via the game's `switch_channels_time` stamp [M 0x1b1a78:87,107,188-189 — every
nonzero write stamps] AND the game-true clear. The same assert fix landed in walkreal's chain
test. **interactables_0700001b_live: 7 passed / 0 failed.**

### 3. What remains

- **The crossing** (walkreal's `walk_arrival_to_drakes_defeat` stays ignored): the disc data
  seals the east cell and carries no scripted carry — the game's own route needs a live read.
- The defeat test's arena teleport: replaceable once the crossing reads back.

## Live checks for the main agent

1. **The crossing itself**: fresh boot 0700001b, watch the player obj's `obj+0x30` (the root)
   through the slit/zero-G segment — does the game's Bond pass x 4.43 westward in substate 9,
   or is he stopped like the port? (gdbwatch on obj+0x30, the address via `glb_players`
   0x2d88e0 → obj+0x30.)
2. **A code-side mover while disabled**: the same window — watch for ANY write to obj+0x30
   while the player is event-4 disabled (`Player_Disable` 0x1d94c0 case 4 breakpoints cleanly at
   0x1d95xx's case-4 arm). If Bond moves while disabled, the writing instruction is the carry
   the port needs.
3. **ch 74's reader**: gdbwatch on `switch_channels[74]` (0x2d8890-family array — the byte at
   the channels base + 74) set to READ during 0x060008f0's play (t392-399 of the SP#211 chain,
   behind the missile choreography — reaching it live may need the console shots first). The
   placement sweep found no reader; if the game reads it from CODE (not placements), the read
   address names the mechanism.
4. **The SP#211 chain end-to-end**: fresh boot, drive the real console path (shoot console 1's
   screen 20 rounds) and confirm ch 30 (0x060007e2 s1 t180 [D]) → SP#211 → 0x060007fd (4780
   ticks) → 0x060008f0 (the float entry + ch 10) fires without any stand-in writes — the disc
   chain above the missiles may be alive once the consoles' gates open another way.

## What was run

Foreground chunks of the fast runner (`tools/test/suite.py`, `--test-threads=1`, NF_SILENT=1) on
the final tree (merged with `origin/master`) — the sandbox kills background jobs between tool
calls, so the suite ran in foreground chunks:

| chunk | binaries | result |
|---|---|---|
| walk_0700001b, zerog_0700001b, fix_finale, interactables_0700001b, walkreal_0700001b, fix_world_gravity | 6 | **26 passed, 0 failed, 1 ignored** |
| probe binaries (probe1b_drake/route/wall/scripts/blockers/float/volumes/rover/spawner/c2writers/nis211, probe1b_carry, probe1b_killgap, probe1b_overthetop) | assertion-free dumps | run individually with --ignored --nocapture; outputs quoted above |

The 1b family is fully green with the defeat test un-ignored (the ignored one is
walkreal's sealed-crossing chain test). The change set touches only
`crates/nf-view/tests/` (two test files' asserts/notes, three new probes) and
`docs/levels/0700001b.md` (§8) — no engine code changed, so the wider families carry no
behaviour risk from this round; nf-view compiled clean and its 1b binaries ran above.

## Usage

not visible to me
