# door-scan-skip-bit-2 — report

**Status: done.** The door-scan skip bit survives AND the escort's interior doors work again: the
defect was not the scan filter but feeding the player's **boot** flag value into it for the whole
level, when the game's own runtime writers clear the bit at the intro hand-back.

## The bug, in game terms

`Door_Update` 0x1363c8's two proximity scans walk the object list and skip every candidate whose
`obj+0xF0` carries bit 0x10 [M 0x136678-0x136684 and the timer-arm copy 0x136724-0x136730]; their
only action is `Door_Activate(door, glb_players, 0)` when no unflagged class-2/3/0x12 body is
within `2·(r_door+r_body)²` — the close/toggle arms. The open paths are elsewhere and unfiltered:
`Door_Activate` 0x135be0 reads only the door's own +0x70 busy marker [M 0x135be0:64], and the ally
door-open path is the NPC's own chain — route-node door flags (`NDrone2_ReachedDestNode` 0x153358
stores the door obj in `DCVars+0xba8`) → `DSTATE_OpenDoor` 0x1681f8 → `NDrone2_OpenDoor` 0x14aa38
→ `Door_Activate`, which never reads the ally's `obj+0xF0` [M 0x14aa38:9-11]. So "what the scan
skips" = bit-0x10 bodies, for the close decision only; "what it still lets an ally open" = his own
targeted doors, regardless of the bit.

The first cut (door-scan-skip-bit, merged then reverted in 8ade7e7c) seeded the port's player flag
from the **boot snapshot** — 0x890 on 07000001 (bit SET) [L ee_start_07000001.bin] — and kept it
for the whole level. But that snapshot value is pre-gameplay: the first cut's own census lists the
runtime writers it did not port, and they flip the bit before the escort ever walks: the intro NIS
disables Bond (`Player_Disable(…,0)` 0x195f70:17 sets the bit — script event 4 [M 0x1d94c0:119],
NIS handler [M 0x205f10:105]) and the hand-back re-enables him — `Player_Enable` 0x195498:53,77
**clears** it [M] — as does every `Player_ChangeSubState` 0x190630:35-40 [M]. From the intro's
`Script_Free` to level end the game's Bond therefore reads bit-CLEAR — the auto-close scan sees
him, doors hold open under him, and the escort's own open chain works the shoji doors. With the
boot-only model the port kept Bond invisible to the scan for the entire level: the start-room
shoji doors #0/#1 cycled closed behind him (the 360-tick auto-close [M 0x13660c-0x136700] now
fired while he loitered), and the escort — whose lead/wait bands key off Bond's position and
progress — parked in `ALLY_LEAD_WAIT` (state 0x20) at (3.6, 6.3, 6.6) and never reached the
route's door-#2 node, so `the_escort_opens_the_interior_doors_on_his_route` (settled x = 3.7,
needs < −20) and `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage` went red. A
green/red differential trace of the escort test confirmed the divergence (red: no `hit_door`
target ever set, no door-#2 open, contact in 0x20 for 42k ticks; green: contact targets door #2
in `ALLY_LEAD` 0x1d and leads past at +24725).

## The fix

- `crates/nf-game/src/drone.rs`: new `Drones::bond_door_skip` field — the player bit's runtime
  life. Boot value per level latched at the `Mission_Init` site (`player_door_skip(level)`: SET on
  07000001, CLEAR on 07000005 [L ee_start_*]; the game's boot write is the `Player_Init` glist
  chain [M 0x1910f8:115 → 0x1be910:114-115 → 0x1ca6d8:38-39 → 0x134960:12-13]); `Player_Disable`
  request sets it, `Player_Enable` clears it (`drain_script_requests` — the game's calls are
  synchronous inside `Script_EventHandler` 0x1d94c0, before the control walk). The door actor
  list now feeds `bond_door_skip` (both the offline and net-bodies arms; MP bodies keep the shared
  value — // [S] unverified per-slot lifecycle).
- `crates/nf-view/src/scene.rs`: the `Cam2Mode(else)` hand-back — the port's one modelled
  `Player_ChangeSubState` call site [M 0x195fc8:11-16] — clears the bit with its `SS_WALK` reset
  [M 0x190630:35-40 clears it].
- The scan itself is untouched: the filter stays exactly where the game has it [M 0x136678-84],
  `Door_Activate` stays unfiltered, and the drones' enable-lifecycle flag (`!started`, the
  `NDrone2_DefaultInit` 0x14b300:606 / `NDrone2_Enable(0)` 0x149dd0:12-13 pair) is unchanged.
- No assertion was weakened. The prior task's `door_skip_flag_live` pins still pass unchanged —
  they pin the scan's filter behaviour given a flag, and the boot values, both still true.

## Verification

- New `crates/nf-view/tests/door_skip_lifecycle_live.rs` — the real 07000001 boot through the sim:
  bit SET on the first tick [L 0x890], CLEARED after the intro hand-back [M 0x195498:53,77], and
  still clear 600 ticks into play. 1/0.
- The two driver-named tests, both red before the fix, both green after:
  `fix_npc_doors_live::the_escort_opens_the_interior_doors_on_his_route` ✓ (7.8 s),
  `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage` ✓.
- Group totals (all `--release`, `NF_ISO` on the slim image, `--test-threads=1`):
  - `fix_npc_doors_live` 1/0; `escort_cover_live` 3/0; `escort_follow_07000001_live` 1/0;
    `escort_hide_findrun_live` 3/0; `glass_dive_07000001_live` 1/0; `interact_07000001_live`
    10/0; `interactables_07000001_live` 10/0; `walk_07000001_live` 3/0;
    `walkreal_07000001_live` 6/0 — the `escort_*` + every `*07000001*` binary group: **38/0**.
  - Every `interactables_*` binary (all 26): **233/0** (07000001's 10 above + 108 + 115).
  - `nf-game` crate: **1037/0** (includes `door_skip_flag_live` 3/0 unchanged).
  - Full suite (`tools/test/suite.py`) on the final tree (merged with origin/master at a1cfe068):
    see the total line below.

## What's left / notes

- The full-suite total line is appended below when the run completes.
- Live checks for the main agent: with a **FRESH boot** (a save-state resume skips the boot path),
  watch the player object's `obj+0xF0` (u32) on 07000001 through the intro: expect 0x890 at boot
  [L], still 0x890 while the intro NIS holds Bond, then **0x880** after the hand-back (bit 0x10
  cleared by `Player_Enable` 0x195498:53,77), staying 0x880 during normal play. If a mid-play
  `Player_Disable` fires (a later cut-scene), it should read 0x890 again for the NIS's duration.
  That single watch (`gdbwatch.py` on the player obj's +0xF0, write+read) would upgrade the
  runtime half of the lifecycle from [M]-only to [L].
- // [S] unverified: the other levels' player glist boot words (only 01 and 05 are evidenced), and
  the MP bodies' per-slot bit lifecycle.

## Usage

not visible to me — this session's environment shows no token, credit or context-window figures
(input/output tokens, context used, cost are all absent from the sandbox tooling I can see).
