# level-0a-office-hatch-6 — the spawn settle without the 05 truck-ride regression

Branch `s/level-0a-office-hatch-6`, started from `origin/s/level-0a-office-hatch-5` (not merged),
merged `origin/master` early (tools/test/suite.py on the branch).

## What was done

### 1. The 05 truck-ride regression — REPRODUCED, ROOT-CAUSED, FIXED

**The symptom (reproduced on the hatch-5 tree):** `walkreal_07000005_live::walk_to_the_gatehouse_and_ride_the_truck`
rode the truck and parked at root **38.37661** — the exact park height of fix_truck_live's STANDING
sister — and never fired ch 0x70.

**The park-tick numbers, both trees** (the walkreal ride, ch-0x70 plate logged from the disc's own
script entity through the port's trigger load — `probe_05_park.rs`):

- the ch-0x70 plate [D script stream 6 key 8, through the port's script load]: a flat quad,
  lo (28.804108, 37.741814, 48.377644), hi (30.865875, 37.741814, 52.184772) — 2.06 × 3.81 at
  y 37.7418.
- with the settle armed at every `stand_at` (hatch-5): parked (29.875406, **38.37661**, 51.988575),
  crouched **false**. Inside the plate XZ, but the touch capsule (from pos+0.15 up, r 0.55
  [M `Toucher::player`, Player_Collision 0x194df0's 0.55]) reaches down only to 38.3766+0.15−0.55
  = **37.9766** — it misses the plate **vertically, by 0.2348** (the TOP side: the standing root
  sits the whole crouch delta too high). ch 0x70 never fires.
- without the settle (master, and this branch after the fix): parked crouched at root 37.923
  (probe run (29.857712, 37.923344, 51.79026); the trace's game park (29.885, 37.887, 51.656)
  [L tests/traces/05-truck.trace]) — the capsule bottom 37.523 is under the plate: touches.

**The root cause [M]:** the game's `Player_Enable` 0x195498 does NOT go through
`Player_StandAtNewPosition` at all — it copies the matrix translation into obj+0x30 DIRECTLY (no
floor ray, no +1.1) [M 0x195498:57-68], zeroes the velocities [M :81-86], and runs a sphere
intersect only for wire/creepwall grabs [M :90-112]. The port's `PlayerEnable` handler calls
`Bond::stand_at`, which hatch-5 had turned into "floor+1.1 AND arm the 4-tick settle hold". The
truck hand-off (SP #38's `Script_Free` → `Player_Enable` [M Script_Free 0x1da880:46-49], measured
enable `at` origin (92.25, 19.42, 103.58)) armed the hold, and the hold's early return swallowed
the FIRST post-enable tick — the tick the walkreal ride sends its crouch press on (the
disable→enable edge arms `crouch_armed`; the press goes out the next tick). Bond rode STANDING,
the standing capsule missed the plate by 0.2348. The crouch press is exactly the [D fix-truck]
trace's ride stance.

**The fix (commit 90e1eaea):** the spawn arrival settle is LEVEL-START ONLY.

- `Bond::stand_at` no longer arms anything (master behaviour: floor ray + 1.1
  [M Player_StandAtNewPosition 0x190d88:107-113]).
- New `Bond::arm_spawn_settle()` arms `settle_hold = 4`; the level-load spawn calls it —
  lib.rs's `Player_Init` 0x1910f8:156-181 port (the path the [L ee_start_0700000a] evidence
  lives on).
- The `Player_Enable` path (scene.rs:3314), the MP respawn sites (scene.rs 10415/10518/11778,
  the `MP_ReSpawn` 0x185368 → `Player_StandAtNewPosition` ports) and Scene::build's structural
  fallback (scene.rs:8414 — replaced by the real spawn on every level with 0x24 starts) keep
  master behaviour, no settle.
- Why not place the enable at the matrix origin exactly as [M 0x195498:57-68] says: the measured
  enable origin y 19.42 sits 0.87 BELOW the port's floor+1.1 (20.289) — a direct placement would
  embed Bond in the truck body and break the ride the other way. The port's enable keeps the
  floor+1.1 stand-in, now documented in `stand_at`'s doc. (A future round with live reads of the
  game's truck-body geometry can re-derive it.)

**Verified:** walkreal_07000005_live 5 passed / 0 failed (the truck ride green again, the whole
chain into 07000006 + the objective chain), walkreal_0700000a_live 4/0 (the settle rest still
pinned bit-exactly), fix_truck_live 2/0 (the ride + the standing sister).

### 2. The hold gate — not found; the observable model kept, marked [S]

Read for the gate: `Player_StandAtNewPosition` 0x190d88 (the placement ray from root+0.1, +1.1
back [M :107-113]; `obj+0xfa |= 0x20` [M :124]), `Player_Init` 0x1910f8 (writes WldGravity
(0, −9.8, 0) immediately before the placement calls [M :146-156 — so no zero-gravity hold window
explains it]), `GS_PausePlayer` 0x17d280 (a 52-byte MPGame pause-flag store, not a boot gate
[M]), `Player_Collision` 0x194df0 (the WldGravity add and the integration are in the SAME call,
so "4 adds, 1 integration" is not 4 Player_Collision calls [M :107, :189-197]),
`Player_FeetOnPoint` 0x194188 (the band catch [M :106]). Nothing in the read set yields a gate
that holds the boot's first four ticks. Per the task: the observable model stays
(`Bond::settle_hold`, the 4-held-ticks + one-integration shape that reproduces the ee_start_0a
root bit-exactly), and the gate is marked `// [S] unverified` in the field doc and in
`arm_spawn_settle`'s doc — kept out of pass/fail as to the gate; the ROOT it produces is the [L]
pin.

**Supporting snapshot measurements [L]** (`probe_05_floors.rs`, the disc's own floors through the
port's collision + the ee_start files, file offset = EE address):

- `ee_start_0700000a`: root 1.089111089706421 over floor 0.0 [L the breakpointed placement ray] —
  the settle rest, as pinned by hatch-5.
- `ee_start_07000001`: root 6.4286932945251465 over a FLAT disc floor 5.3394027 — root−floor
  1.0892906, the settle height within the band's f32 rounding (0a's floor is exactly 0.0; 01's
  is not).
- `ee_start_05`: root 24.963300704956055, ALL body velocities zero (BL+0x50 area reads 0 on the
  anim object; the obj+0x50 word is the look spherical the boot passes, not a fall velocity) —
  Bond RESTING. But the courtyard floor under the spawn xz is SLOPED (plane 23.8774147 at the
  spawn xz [D tris]), and 24.9633 matches NEITHER floor+1.1 (24.9774) NOR floor+1.0891
  (24.9665): the game's own 05 rest sits 0.0141 BELOW the port's floor sampling. This is a
  pre-existing divergence (master's spawn reads floor+1.1 = 24.9774 vs the game's 24.9633 too);
  the walkreal_05 spawn assert is ±2.0 and unaffected. Listed for the live checks below.

### 3. The hall glide and the corner — still open, unchanged

Both ignored sisters re-ran on this tree (`--ignored`):

- `the_west_hall_glide_height_matches_the_game`: the walker wedges at "the doorway base line"
  leg — bond (−9.15, 1.09, 15.92), target (−8.45, 16.29), d 0.79, grounded, segment clear
  (the walker's STUCK dump). The port's capsule still does not reproduce the game's doorway-base
  climb (+0.0256 settle to the 1.1147 glide [L fresh boot]).
- `the_corner_crosses_like_the_game`: "the wedge corner" leg arrived (+23 ticks), then "the game's
  corner line" dithered ungrounded at (−17.87, 1.17, 4.87) to the 6000-tick cap — the game walks
  it at y 1.127→1.120 [L fresh boot].

These need the game's per-tick climb/push shape through a small ledge base (the -2 report's open
item around `Player_CollisionHandler` 0x1949a8:136-140 — whose push-add the port already models
as `pos += push`; the gap is subtler, inside the below-B edge lift shape). No new decompile
evidence landed this round; both stay ignored with their notes. The main walk
(`walk_office_floor_qworm_keyboard_hatch`) still wedges at "the ai lane to the N-S corridor"
(the 0.5-wide mouth) — untouched this round, stays ignored.

## What was run

Foreground chunks of the fast runner (`tools/test/suite.py`, `--test-threads=1`, NF_SILENT=1) on
the final tree (merged with origin/master) — the sandbox kills background jobs between tool
calls, so the suite ran in five foreground chunks totalling 66 binaries:

| chunk | binaries | result |
|---|---|---|
| walkreal 01/02/04/06/07/09/0c/0d | 8 | 15 passed, 0 failed, 2 ignored |
| walkreal 11/11_route/13/14/15/16/41/4a/0a | 9 | 19 passed, 0 failed, 7 ignored |
| walk_02/05/06/09/0a/14/16/4a, level_carry, fix_world_gravity, fix_start_loadouts, mp_respawn_ammo | 12 | 40 passed, 0 failed, 0 ignored |
| nf-game/nf-view/nf-formats/nf-audio units, hatch_crouch ×2, drop_stand, fix_swim_entry/vent/vent5, fix_ladder ×3, mp_core, fix_player2/3/4, walkreal_probe0a/12b/12c/12d | 21 | 44 passed, 0 failed, 4 ignored |
| mayhew_handover, interact_05, copter_05_flight, sp_disabled_byte_05, walk_03/07/0b/0c/0d/11/13/41/04_ptp, keyboard_0a, fix_truck, walkreal_05 | 16 | 50 passed, 0 failed, 2 ignored |

**Total: 168 passed, 0 failed, 15 ignored** (the ignored are the documented sisters: the 0a
glide/corner/main-walk trio, the walkreal family's known route ignores, the lib units' platform
skips). No regressions from the settle-scope change. The remaining ~415 binaries (the render,
cutscene, menu and net families) did not fit the sandbox window after the background-kill
constraint; every binary whose input path touches `Bond::stand_at` / the spawn / the settle ran.

Crates the change touches: nf-game (player.rs), nf-view (lib.rs, scene.rs) — both compiled and
their binaries ran above.

## Live checks for the main agent

1. **The 05 boot's placement ray** (the pre-existing 0.0141 floor divergence): fresh boot 07000005,
   breakpoint `build_PointOnFloor` 0x1c95a8 (the same spot the 0a ray was breakpointed), read the
   hit y for the ray from (104.954, 25.2195, 108.825). Expected [L-model]: a hit ~23.8633
   (= snapshot root 24.9633007 − 1.1) if the 05 boot rests at the placement with NO settle — or
   ~23.8742 if the settle ran. The port's floor_below reads 23.8774147 at that xz (the sloped
   courtyard tri); which surface the game's ray actually hits decides whether the port's spawn
   ray needs the game's per-cel hit rule.
2. **The settle gate**: fresh boot 0700000a, watch `Player_Update` 0x193600 and
   `Player_Collision` 0x194df0 entry/exit for the first 8 frames after the placement (addresses
   BL+0x50 the velocity, obj+0x30 the root): confirm the tick count where the velocity first
   integrates into the root, and which caller runs Player_Collision in those frames (is it
   Player_Update at all, or the boot's own collision pass?). This settles the `[S] unverified`
   hold gate.
3. **The truck hand-off placement** [M 0x195498:57-63]: fresh boot 07000005, breakpoint
   `Player_Enable` 0x195498, read obj+0x30 right after the matrix copy and for the following 5
   ticks. Expected: Bond rides the matrix origin directly (y ≈ the truck origin 19.42 at the
   measured hand-off) and the truck's own body collision lifts him onto the bed — the port's
   floor+1.1 stand-in (20.289) may then be replaceable with the faithful direct placement.
4. **The doorway base climb** (the glide sister): fresh boot 0700000a, walk the game's doorway
   line and log obj+0x30/BL+0x750 per tick across the base touch at x −8.304 — the push vector's
   per-tick build (which contacts feed it and the band interplay) is the shape the port's climb
   is missing.

## What's left

- The 0a hall glide + the corner crossing (the climb/push shape — the live checks 3/4 above).
- Un-ignoring the sisters and the main walk once green.
- The remaining suite families (the render/cutscene/menu/net binaries) on a round with more
  window.
- Optional: the enable placement re-derived to the game's direct matrix origin once live check 3
  reads back.

## Usage

not visible to me
