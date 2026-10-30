# level-14-cascade-servers-and-door-guards — report

Branch `s/level-14-cascade-servers-and-door-guards` (from origin/master 21fbeed3). Task: Countdown
Part 1, the generator-hall blast aftermath on 07000014 — (1) the Destroy leaves the cascade smashes
must show what the game shows after the smash, (2) the guards at the locked west double door must
behave like the game's.

## Part 1: the destroyed-glist swap's draw side — DONE, pinned

**The game's mechanism** ([M] unless noted):

- `Destroy_Create` 0x12c318:25-30 stores the destroy tag's first word: `*(glist+0x40)` when the
  block-0x27 LOD-next word stands, else the parsemap fallback slot `pCelList + idx*0x50 + 0x50`.
  The fallback is computed unconditionally in `parsemap_block_map_data_dynamic` 0x1d1b68; the cel
  list is one 0x50-byte record per main-map entity and `parsemap_block_LOD` 0x1d1f10 writes
  `cel+0x40 = index*0x50 + pCelList`, so the fallback record is the NEXT main-map entity's glist.
- `Destroy_Smash` 0x12bb60 swaps it in with `Control_SetGList(obj, tag[0])` — the decomp dropped
  the argument; the disassembly settles it (0x12bbb8 `lw a1,0(s1)` → 0x12bbc4 `jal 0x134960`).
- An undrawable target shows nothing: `View_AddObjects` 0x1e62d8:33 submits a dynamic object only
  when `(obj+0xF0 & 0x8010) == 0` (the FLAGS_COPYING census, fix-striped-boxes2), and the creator
  itself zeroes the swap target when the intact glist carries 0x10 (0x12c318:57-59).

**What 07000014 carries** ([D] census, all rows hashcode −1: 48 drawn-or-placed 0xe1 leaves):

| object | placements | block-0x27 word | destroyed target | verdict |
|---|---|---|---|---|
| object_oilbarrel01 (54.290) | 25 | 289 | object_oilbarrel01_destroyed (6 batches, flags 0) | real mesh — the wreck shows |
| GENERATORS_Intact (54.291) | 4 | 304 | GENERATORS_Destroyednew (5 leaves / 48 batches, flags 0) | real mesh — the wreck shows |
| DESTROY_BoxSatellite (54.273) | 19 | −1 | fallback = 3RD_Person_Icon (flags 0x50, bit 0x10 set) | undrawable slot — vanishes |

**The fix** (`crates/nf-view/src/scene.rs`, `gpu.rs`, `lib.rs`): every drawn 0xe1 leaf now resolves
its swap target exactly as `Destroy_Create` does (LOD-next first, else the index+1 fallback record),
bakes the wreck instance at the LEAF's own matrix (the swap changes only the glist; obj+0x90
stands), and gates it: the wreck draws only while the leaf is dead (`Renderer::destroy_wreck_draw`
fed per frame against the build-time `destroy_wreck_all` set, matched by material-centre bits — the
same mechanism as the existing `destroy_dead` filter, which keeps dropping the dead leaf's intact
materials). A wreck whose object has no batches or carries 0x10 is not baked — those leaves vanish
as before. The wreck's ambient dim follows the swapped-in glist's own 0x800 flags.

**Verification**: `crates/nf-view/tests/destroy_wreck_07000014_live.rs` (both green):

- `the_destroyed_glists_resolve_per_the_game` — the disc words above ([D]), the drawable gates, no
  wreck baked for any satellite leaf;
- `the_cascade_swaps_the_smashed_leaves_to_their_wrecks` — the REAL E press on the console plate
  (the walk's real fusebox cut pins the 0x87/0x88 seeds; here they seed the REAL AND mux [D]) → the
  cascade → 0x85 [D] → the smashed drawn leaves are dead, their wreck draws are live, and the wreck
  draw set is disjoint from the dead intact draw set.

The destroy-family regressions are green (`destroy_draw_keys_0700000d_live`,
`destroy_channels_02_live`, `blast_debris_draw_live`, `fix_blast_scripts_live`): the 0d C5 leaves'
fallback slots are undrawable, so nothing is baked there and the doorway still ends open.

**For the main agent's render** (no GPU here): `--bond -55.0,1.5,42.0,-1.2,0.05` should show the
generator hall's barrel racks (east of the camera) AFTER the cascade as wrecked barrel meshes
(`object_oilbarrel01_destroyed`), not empty floor; the same spot before the cascade shows intact
racks. `--bond -48.0,2.0,44.0,2.6,0.1` should show `GENERATORS_Destroyednew` where the generators
stood (and the satellite boxes gone — they are 0x10-flagged and never drew).

## Part 2: the door guards — wait pinned, two decomp-exact fixes landed, crossing still open

**What the game does** ([M], read this session): the spawner clones (start ch 0x8C [D]) run west;
the route's door node parks them in `DSTATE_OpenDoor` 0x81 (`NDrone2_ReachedDestNode` 0x153358:144-163,
the door into +0xBA8, the saved state into +0x5A6). The wait is uninterruptible: OpenDoor's switch
handles ONLY cases 0/1/2/3/6/7/8/9/0x17/0x18/0x19/0xC/0x20/0x21 (the whole switch read at 0x1681f8)
— noise/sight/shout/scream re-route as msg 0 and case 0 drops them; case 3 leaves only on
`NDrone2_DoorIsOpen`; case 0xC re-arms the 2-tick `NDrone2_OpenDoor` retry; case 0x20 is the bark
(`Drone_SfxInfo[0x27]` = sound 1477). Its case 0x21 calls `DroneFunc_ConsiderExplosive(payload,
*(u16*)(SM+4))` — the resume is the CURRENT state, so a frightened door-waiter returns to the door.
`Door_Activate` 0x135be0 refuses a locked door; the unlock rides destroyable #729's death 0x85 [D]
and `Door_Update` 0x1363c8:88-107 self-opens on the rising edge.

**The port's two deviations — fixed** (`crates/nf-game/src/drone.rs`, both [M]-cited in code):

1. The central SCREAM/SHOUT route transitioned OpenDoor guards (`enemy_alerts`); the game's
   OpenDoor has no 0x11/0x12 case → dropped. Added OPEN_DOOR to the pre-dispatch drop.
2. The central msg-0x21 route resumed ATTACK for OpenDoor guards (the `else` fall-through); the
   game's case 0x21 resumes the state itself. Added `s == OPEN_DOOR → OPEN_DOOR`.

**Pinned** (`crates/nf-view/tests/door_wait_07000014_live.rs`, green): the released clones park at
the locked west door in state 0x81 east of the door plane (≈ −51.8, 1.0, 40.0) for the whole
sustained window (>> 1200 OpenDoor ticks), and the door never unlocks nor moves during it — the
game's locked-door wait.

**Still open — the post-unlock crossing.** The probe (`crates/nf-view/examples/probe14_guards.rs`,
committed here) measures the port end-to-end (seeds 0x87/0x88, real E press, cascade): the door
unlocks and fully opens, but the three door-waiters restore into their saved combat state (+0x5A6 =
0x5a), engage Bond 8.7 m west, throw frag grenades at the threshold (the leaf still blocks the
line), and every thrower dies to the resting grenade's blast at the doorway — nobody crosses x <
−52.4. The two landed fixes do not change that outcome (measured: identical before/after), because
the pull-out here is the OpenDoor UPDATE restore into combat, which the game's own case 3 also
does — the remaining defect is somewhere in the restore→combat→grenade chain at the threshold. This
session's earlier suspect (the dmg-0 screen-swap blasts pulling them out) is CLEARED: the port's
`explode_propagate` already has the game's `< 0.0002` skip [M 0x17c518 — `fVar10 < 0.0002` drops
the hit before the switch], and dmg-0 blasts produce no msg 9 at all.

**Live checks for the main agent** (PCSX2, fresh boot; the state word is Drone_tag+0x10C):

1. One spawned door-waiting guard around the 0x85 write: does its state stay 0x81 until the door
   reaches open, then flip to the saved state — and what IS the saved state byte (+0x5A6)? Expected
   [M]: 0x81 until open, then the saved state; if the game's saved state is NOT a combat wrapper
   (0x59/0x5a/0x5c), the port's `ReachedDestNode` save is wrong and the restore target is the bug.
2. Does a door-waiting guard ever throw a grenade between the E press and the door reaching open?
   Expected [M]: no (OpenDoor has no throw path). If the game shows threshold grenades, the
   restore→throw chain is game behaviour and the defect is elsewhere (the crossing itself).
3. After the door is fully open: does the game's guard fire THROUGH the doorway or wait at it?
   gdbwatch the guard's state 0x10C for ~10 s after pos01 reaches 1.0 — expected: combat-move arms
   (0x5a family) walking west through the doorway within ~2 s.

## What ran

- New/changed-crate tests: `destroy_wreck_07000014_live` (2/2), `door_wait_07000014_live` (1/1),
  `destroy_draw_keys_0700000d_live` (1/1), `destroy_channels_02_live` (1/1),
  `blast_debris_draw_live` (3/3), `fix_blast_scripts_live` (3/3) — all green on this branch's tree.
- FULL suite: started in the background twice. The branch-start-tree run (fast runner,
  `tools/test/suite.py`, started 01:58 UTC) was still executing binaries at cutoff; its partial
  output shows the slim-ISO disc failures only (`identity_disc`,
  `disc_golden::bigf_header_fields_behave_as_documented`,
  `disc_golden::bigf_tables_match_the_golden_manifest`,
  `disc_golden::directory_tree_matches_the_golden_manifest`) plus one runner TIMEOUT-CRASH
  (`own_sleeve_live`, exit −9 at the 600 s limit) — all pre-existing on the start tree, none from
  this task. The final-tree run (started 03:14 UTC) did not reach its total line before the sandbox
  cutoff; no total line this session — the next message should run it on this branch.

## What's left

Status at the sandbox cutoff (03:33 UTC, expiry ~03:54): part 1 is complete and pinned; part 2
needs one more session (the crossing bisect via the live checks; the FULL-suite total line on this
tree). This dispatch marks the queue entry blocked for that reason only — the branch carries all
two commits of fixes, both pin tests, the probe harness and this report.

1. The door-guard crossing (the live checks above; the probe harness is committed for the bisect).
2. The FULL suite total line on this final tree.
3. Optional: the satellite boxes' smash plays its explosion + channel write (the sim already does;
   only the draw side is 0x10-undrawable) — a visual check on PCSX2 would confirm the intact box
   keeps drawing after its smash (the creator's 0x10 zeroing leaves the intact glist attached).

## Usage

- GitHub token: `x-access-token:` PAT from the driver message step 1 (used as the HTTPS password for
  clone/push only; not stored in any committed file).
- Context window: not visible to me. Token/cost figures: not visible to me.
