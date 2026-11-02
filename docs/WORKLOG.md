## door-scan-skip-bit (started 2026-11-01 20:52 UTC)

Task: the door auto-close scan (Door_Update 0x1363c8) skips any object whose obj+0xF0 has bit
0x10 [M 0x136678-0x136684]. Live reads: 07000001 player (0x890) and drones (0x8B0) have it SET,
the 07000005 player (0x880) has it clear [L]. The bit's writer is unported, so drone.rs
(~line 17476, the player_actors/actors lists) passes false for every actor and door.rs (~line 305,
DoorActor) documents it. Find every writer of obj+0xF0 bit 0x10 in ACTION.ELF [M: cite each] and
the condition that sets it per level/object class; carry that flag on the port's player and drones
and feed it into the actor lists, so a door's auto-close ignores exactly the bodies the game's
does. Pin the flag per level with tests ([M] for the writer, [L] values above as the check), plus
one door behaviour that changes on 07000001. Branch cut from origin/master 6042ee53.

## door-scan-skip-bit — progress (2026-11-01 22:22 UTC)

Writers of obj+0xF0 bit 0x10 found (full census): the seed is the level celglist word —
`Control_SetGList` 0x134960:12-13 and the creators' inline `+0x44` copies; the player reaches it
via `Player_Init` 0x1910f8:115 → `AnimObjectNew` 0x1be910:114-115 → `hashtable_set_object_to_entity_gfx`
0x1ca6d8:38-39 (0x890 = glist 0x90 + `Player_Init`'s 0x800 at :149; 0x880 = glist 0x80), the drones
via `Control_CreateObjEx` 0x1343c8:30-36. Runtime: `NDrone2_Enable(0)` 0x149dd0:12-13 sets
0x10|0x20 on waiting/dead drones, `NDrone2_DefaultInit` 0x14b300:606 clears them at spawn;
`Player_Disable(…,0)` 0x195f70:17 sets (script event 4 0x1d94c0:119, NIS 0x205f10:105),
`Player_Enable` 0x195498:53,77 / `Player_ChangeSubState` 0x190630:35-40 clear; plus the effect
hiders (Script_HideObj 0x1db808, Drone_PauseNoDraw 0x13a710, pickup respawn 0x18e148/0x18e650,
sensor 0x1af618, bullet embed 0x12ce80, muzzle/flash objs, sky/one-sided/BodyGlow view gates,
MP GoldenEye/redeemer cams). Ported: the actor lists in `Drones::tick` now carry the flag —
the player's boot value per level (`door::player_door_skip`: SET on 07000001, CLEAR on
07000005 [L ee_start_07000001.bin/ee_start_05.bin]; // [S] unverified elsewhere — the player
model's celglist word is not resolvable through the port's loader yet), the drones' per the
lifecycle (`!started`; corpses already sit out via `!corpse`). The runtime player writers and
the effect hiders are NOT ported (out of scope: they never touch classes 2/3/0x12 bodies).
Tests: crates/nf-game/tests/door_skip_flag_live.rs (3 — per-level pin, the 01 auto-close
ignores Bond in the doorway, unspawned drones don't hold doors). Runs: nf-game lib 648/0;
nf-game door tests doors_live+doors_01_live+door_guard_live+door_glist_census_live+misc3_door_wipe
11/0; walkreal walk_07000001_live 3/3, walk_07000005_live 2/2. Full suite could not run: this
sandbox kills background jobs between tool calls (three starts, all dead within minutes); the
affected-crate runs above are the reported coverage.

## door-scan-skip-bit-2 (started 2026-11-02 13:46 UTC)

Branch cut from origin/s/door-scan-skip-bit c5753f76; origin/master a1cfe068 merged. Master dropped
docs/WORKLOG.md + docs/reports/ in batch-1101j e0ec89de; restored both here from c5753f76 so this
task's entries and report live in the repo per the driver contract. Task: on master the skip-bit
work turns fix_npc_doors_live::the_escort_opens_the_interior_doors_on_his_route and
walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage red — the
escort's interior shoji doors no longer open for him. Find what the game's door scan skips and what
it still lets an ally open [M Door_Update 0x1363c8 / the ally door-open path], fix the port so both
the skip bit and the escort's doors behave as the game does, keep both tests green without
weakening an assertion.

## door-scan-skip-bit-2 — the fix (2026-11-02 15:16 UTC)

Reproduced both driver-named reds on the reapplied skip-bit tree (revert-of-revert 2d30c128, clean):
the escort parked in ALLY_LEAD_WAIT 0x20 at (3.6,6.3,6.6) for 42k ticks, never targeting door #2;
green/red differential trace showed the green run's contact reaching ALLY_LEAD 0x1d with
hit_door Some(2) and leading past at +24725. Root cause [M]: the first cut fed the player's BOOT
bit value (0x890, SET [L ee_start_07000001.bin]) into the Door_Update 0x1363c8 auto-close scan for
the whole level, but the game's runtime writers flip it before the escort ever walks —
Player_Disable(…,0) 0x195f70:17 sets it (the intro NIS), Player_Enable 0x195498:53,77 clears it at
the hand-back, Player_ChangeSubState 0x190630:35-40 clears it — so in play the game's Bond is
bit-CLEAR and the scan sees him. The scan filter itself [M 0x136678-84], Door_Activate's lack of
one [M 0x135be0:64] and the ally's own open chain (ReachedDestNode 0x153358 → DSTATE_OpenDoor
0x1681f8 → NDrone2_OpenDoor 0x14aa38, no bit read) were re-verified from the decompile and stand.
Fix: Drones::bond_door_skip carries the runtime life — boot value latched at the Mission_Init
site, set by the PlayerDisable request, cleared by PlayerEnable (drain_script_requests) and by the
Cam2Mode(else) hand-back in scene.rs (the modelled ChangeSubState site [M 0x195fc8:11-16]); the
door actor list feeds it. No assertion weakened — door_skip_flag_live's three pins pass unchanged.
New pin door_skip_lifecycle_live (real 07000001 boot → SET, intro hand-back → CLEAR, stays clear).
Runs: escort_* + every *07000001* binary 38/0; all 26 interactables_* 233/0; nf-game 1037/0
(door_skip_flag_live 3/0 unchanged). Full suite on the final tree: see the report.
