# door-scan-skip-bit — report

**Status: done** (the flag is carried on the port's player and drones and feeds the door
auto-close scan; the runtime player-side writers are out of scope and listed under "what's left").

## What the bit is, and who writes it

`Door_Update` 0x1363c8's auto-close scan walks the object list and skips every candidate whose
`obj+0xF0` (u32 draw/flags word) carries bit 0x10 [M 0x136678-0x136684]. `Door_Activate` 0x135be0
has no such filter — its only +0x10 read is the +0x70 busy marker [M 0x135be0:64] — so open/crush
paths see every body; only the auto-close scan filters.

Full census of the bit's writers in ACTION.ELF (decompile at `work/decomp/ACTION.ELF/`, every
claim checked in the file; ~300 `+ 0xf0` write sites triaged, non-obj_tag structs excluded):

1. **The seed: the level's celglist word.** `Control_SetGList` 0x134960:12-13 copies
   `*(glist+0x44)` into `obj+0xF0` (masked against 0x30008000); 19 creators copy it inline
   (Trigger_Create 0x1b1720:95, FuseBox_Create 0x1b0448:27, Sensor_Create 0x1af010:127, …), and
   `Control_CreateObjEx` 0x1343c8:30-36 copies it plus its flag argument. The player reaches
   SetGList through `Player_Init` 0x1910f8:115 → `AnimObjectNew` 0x1be910:114-115 →
   `hashtable_set_object_to_entity_gfx` 0x1ca6d8:38-39; `Player_Init` 0x1910f8:149 then ORs 0x800.
   The drones: `Drone_Create` 0x136be0:62 → `NDrone2_CreateObj` 0x14d7c8:17-19 →
   `Control_CreateObjEx(..., 0x44)` → `obj+0xF0 = *(glist+0x44) | 0x44`.
2. **The drone enable lifecycle.** `NDrone2_Enable(0,…)` 0x149dd0:12-13 sets 0x10|0x20 on the
   waiting/disabled drone's object (the disable arm; callers: WaitSwitch msg 1 0x15b960:15, every
   death state — Death_Anim 0x1637f8:89, Fade 0x172cd8:43, Dead 0x163fe8:40, DeleteMe 0x173f00:12
   —, Drone_FellOutMap 0x13a878:13, Drone_EnableAll(0) from script event 5 0x1d94c0:123,
   DroneSpawner_Init 0x137aa0:58, BOT_respawn 0x124e38:68). `NDrone2_DefaultInit` 0x14b300:606
   clears 0x10|0x20 on every spawned drone; `NDrone2_Enable(1,…)` 0x149dd0:24-26 clears them when
   `(ext+0x4F8 & 0x200) == 0` (bit 0x200 = dead/corpse — a corpse can never re-enable).
3. **The player hide/show paths.** `Player_Disable(…,0)` 0x195f70:17 sets it on the player object
   and the held-weapon obj `*(ext+0x7E8)`:21 (callers: script event 4 0x1d94c0:119 — the level
   script's intro-disable —, the NIS/FMV handler 0x205f10:105, Monitor_Activate 0x1b0058:46);
   `Player_Enable` 0x195498:53,77 and `Player_ChangeSubState` 0x190630:35-40 clear it.
4. **Effect/view hiders** (never classes 2/3/0x12, out of the door scan's scope): Script_HideObj
   0x1db808:26-32, Drone_PauseNoDraw 0x13a710:23-31, pickup take/respawn 0x18e650:344-347 /
   0x18e148:185-188, sensor trip/release 0x1af618:57-90, bullet embed 0x12ce80:568-569 / clear
   0x12e7b0:141-142, gas 0x1ab2d0/0x1aacc8, muzzle flashes 0x180498/0x17d828/0x17dbb8, view gates
   (sky 0x1e79a0:37-38, one-sided walls 0x1e7d58:36-40, BodyGlow 0x1ac2c8, redeemer cams
   0x19dec0/0x19e178, MP GoldenEye 0x187568, MP_CreateObject 0x185de8:183-184, grapple rope
   0x192728:32-44, dropped pickups 0x17a2a0:104-111 / 0x190810:49-54), plus Trigger_Update's
   channel copy 0x1b1a78:39-55 and AnimObjectUpdate's part propagation 0x1befd8:309.

**The condition per level/class** [M+L]: the boot value is the level's celglist draw-flags word
for the object's model — on 07000001 the player's glist word is 0x90 (bit 0x10 SET: obj+0xF0 =
0x90 | 0x800 = **0x890** [L ee_start_07000001.bin]) and the boot drones read **0x8B0**
(0x10|0x20 — the disabled-until-spawned state [L]); on 07000005 the player's glist word is 0x80
(obj+0xF0 = **0x880**, bit CLEAR [L ee_start_05.bin]).

## What the port does now

- `crates/nf-game/src/door.rs`: `player_door_skip(level)` — the player's boot value per level:
  SET on 07000001, the 07000005 model (CLEAR) everywhere else. The port does not resolve the
  player model's celglist word (the game finds it through the entity-gfx hashtable,
  `hashtable_celglist_to_hashcode` on entity+4 — not the port's loader path), so the pinned
  per-level [L] values stand in; // [S] unverified for every board except 01 and 05.
- `crates/nf-game/src/drone.rs` (`Drones::tick`, the `player_actors`/`actors` lists): the third
  `DoorActor` field now carries the flag — the player per `player_door_skip(d.level)`, each drone
  per the enable lifecycle (`!started` — the port's `started` latch IS the spawn event
  `NDrone2_DefaultInit` 0x14b300:606 clears the bit at; corpses already sit out via `!corpse`).
  The old modelled-CLEAR-for-everyone is gone. The MP `net_bodies` arm carries the same per-level
  player value (each body's own obj gets the same glist word).
- The `DoorActor`/`visible_actors` plumbing in door.rs already filtered on the flag; the two
  remaining `(pos, 0.4, false)` sites (drone.rs 19149/19396) feed `Door_Activate`, which does not
  filter — correct as-is.

## Verification

- New `crates/nf-game/tests/door_skip_flag_live.rs` (3 tests, all pass):
  - `player_door_skip_matches_the_live_boards` — SET on 01 / CLEAR on 05 [L values above].
  - `on_07000001_the_auto_close_ignores_bond_in_the_doorway` — the real 07000001 doors through
    `Doors::update`'s auto-close timer arm [M 0x13660c-0x136700]: with the player flag SET the
    door closes through Bond standing in the open leaf's sphere; the 07000005 model (CLEAR) keeps
    it open. This is the behaviour change on 07000001.
  - `unspawned_drones_do_not_hold_doors_open_started_ones_do` — the 0x8B0 boot state [L] does not
    hold the door; a spawned drone (bit cleared [M 0x14b300:606]) does.
- Runs (all with `NF_ISO` on the slim ISO, `--release`): nf-game lib **648 passed / 0 failed**;
  the nf-game door tests `doors_live` + `doors_01_live` + `door_guard_live` +
  `door_glist_census_live` + `misc3_door_wipe` **11 passed / 0 failed**; the new tests **3/0**;
  the walkreal binaries that cross doors: `walk_07000001_live` **3/3**, `walk_07000005_live`
  **2/2** (nf-view).
- **Full suite: could not run.** `tools/test/suite.py` was started in the first minutes, but this
  sandbox kills background jobs between tool calls — three detached starts (nohup and setsid) all
  died within minutes without completing a build. The affected-crate runs above are the reported
  coverage; the next session should run the fast runner once on this tree (the branch merges with
  origin/master cleanly — the change touches only door.rs's new helper + doc, drone.rs's actor
  list, and the new test file).

## What's left

- The runtime player-side writers are not ported: `Player_Disable(…,0)` (script event 4 / NIS /
  Monitor_Activate) setting the bit and `Player_Enable`/`Player_ChangeSubState` clearing it. The
  port carries the boot value only — a 01 intro script that disables the player, or any
  substate change, would flip the bit in the game but not in the port. No live evidence pins a
  gameplay moment where this matters (the [L] reads are boot values); a live check could.
- The player model's celglist word could be resolved through the loader instead of the pinned
  per-level value, which would generalize the flag to unevidenced boards.
- The drones' 07000005 (and other boards') values are not in the snapshots; the lifecycle model
  [M] is level-independent and applied uniformly.

## Live checks for the main agent

1. `gdbwatch` on `obj+0xF0` (the player object) during 07000001's intro and for ~5 s of play
   after: expect 0x890 at boot, and a clear (→ 0x880) on the first `Player_ChangeSubState`
   [M 0x190630:35-40] — settles whether the port's boot-value-only model diverges during play.
2. `gdbwatch` on a 07000001 drone's `obj+0xF0` across its spawn tick: expect 0x8B0 before
   [L ee_start_07000001.bin], 0x880 after `NDrone2_DefaultInit` [M 0x14b300:606].
3. 07000005: read a live drone's `obj+0xF0` pre- and post-spawn (not in the snapshots; the model
   predicts 0x8B0-shaped pre-spawn values per its glist word).

## Usage

not visible to me (no token, context-window or cost figures are exposed in this environment).
