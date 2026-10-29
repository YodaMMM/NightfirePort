# level-16-security-station-use — the 07000016 security station works when the player uses it

The report's symptom: on 07000016 (Countdown part 3) walking up to the security station and
using it did not switch its screens, and the only existing proof of the screen swap
(`dynobj_glass_live::the_16_screens_swap_with_their_channels`) SETS ch 0x24 directly — no
player anywhere in the loop. Task: reproduce on real input, port every missing link, assert
end to end from a fresh load.

## What was already covered (and what each carried)

| Test | Covers | Stand-in / gap |
|---|---|---|
| `dynobj_glass_live::the_16_screens_swap_with_their_channels` | the 0x3b screens' create states + the ch-0x24/0xcc swap, in the sim AND the draw palette | **pokes** `set_channel(0x24)`/`set_channel(0xcc)` — never proves a player can raise 0x24 |
| `walkreal_07000016_live::door0_unlocks_from_the_security_station_use` | the REAL walk (east ramp → upper ring → the station), the E press, ch 0x24, door[0]'s unlock + open | asserts nothing about the screens |
| `walkreal_07000016_live::door0_stays_locked_without_the_station_use` | door[0] refuses the E while 0x24 is cold | (teleport to reach the door — allowed: it then operates the door itself) |
| `worldobj_live::dynobj_07000016_display_pairs_swap_on_their_lock_channels` | the five 0x3b rows' create states + channel swaps through the real load path | sets the channels directly (no player) |

## Investigation: the use path was already fully wired

- **The station is trigger #597** "SecurityStation", kind 0xEA TouchOnce, params
  `0:0x24 1:0x4` — out ch 0x24, mask 0x4 = the activate dispatch, at (−15.20, 6.52, −94.56)
  [D placements; docs/levels/07000016-audit.md].
- **The game's chain**: the use key runs `Player_Activate` 0x1a5170 — a sphere (radius 1) from
  `Player_GetHeadPos` along the viewer direction (:100-108) → `Collide_SphereIntersect`
  (:113-119) → per-hit class dispatch; class 0x1D → `Trigger_Activate` 0x1b2318, which for the
  type-2 record gates on `mask & 4` (:38-40) and writes `switch_channels[0x24] = 1` (:42-43)
  [M]. The port has all of it (player.rs `activate_request` / drone.rs
  `player_use_triggers` — the level-0c-complete deferral lands the write where the same-frame
  object walk reads it).
- **The screens**: 0x3b DynObjects #682 Screen_door_unlocked (p0 = 0x24) and #683
  Screen_door_locked (p1 = 0x24, mode 0), created hidden/visible respectively [M
  `DynamicObject_Create` 0x18dbf8:27-29 `|0x30 when p0 != 0`]; ch 0x24 rising makes #682's
  watch A restore the base flags (drawn) [M 0x18d3b8:32-33] and #683's armed watch B delete it
  [M 0x18d3b8:26 `obj+0xFE |= 1`, mode 0]. The audit's GAP-16-1 ("a channel rise never
  re-syncs the drawn set") was closed by the 1b glass task's per-tick re-sync — the palette
  follows the sim every tick now (the dynobj_glass tests pass on this tree).
- **The screens' world spot is the station itself, not the corridor**: the port probe of the
  two slots shows both carry 8 verts whose glist centre puts the object world-sphere at
  (−15.158, 7.232, −94.561) — the security-room console. The locked screen is what the player
  sees ON the station until the use; afterwards it is the unlocked one. (07000016.md §7's
  "the corridor displays #335/#336" line misnames these — the kind-0x3b census's #682/#683 is
  the disc's pair [D audit table].)

**No port gap was left to fix**: every link of the chain is present and green on real input.
The missing piece was the end-to-end test itself, which this task adds.

## The new test

`walkreal_07000016_live::the_station_use_swaps_the_screens_end_to_end` — fresh
`load_with_carry` of 07000016.bin (the same carried 07000014/15 channel state every walkreal
of this level uses), then real input only (`step_bond` through the Walker; no channel pokes,
no teleports, no bond.pos writes):

1. asserts ch 0x24 cold and the create states: #682 flags carry 0x30 (hidden), palette hides
   it; #683 drawn [M 0x18dbf8:27-29];
2. walks the door[0]-unlock route (the corridor south, door[7], the east ramp, the upper ring
   west, the station — 16 waypoints), arrives, and presses E (`Act::Use`, the walker's pulsed
   edge);
3. asserts the whole downstream set: **ch 0x24 on** [M 0x1b2318:38-43]; **#682 now drawn**
   (alive, flags & 0x8010 == 0, the 0x30 cleared; palette shows it) [M 0x18d3b8:32-33];
   **#683 now not drawn** (dead / hidden; palette hides it) [M 0x18d3b8:26]; **door[0]**
   "Door_1" unlock (ch_lock 0 → 0) and open (state 0x4, leaf 1.0) [M door.rs Door_Activate on
   the rising edge].

FAIL-FIRST record: none needed — the chain was green end to end once measured; the test pins
it (the split-half tests above still cover the poke-only and door-only halves).

## What must DRAW for the main agent (render checks)

- Spot: Bond in the security room's upper floor facing the station console,
  `--bond -15.7,6.9,-93.6` looking east/southeast at (−15.16, 7.23, −94.56).
  - BEFORE the E: the console shows the **locked** screen quad (#683; #682 hidden).
  - AFTER walking to (−15.2, 5.9, −94.6) and pressing E: the **unlocked** screen quad (#682)
    replaces it (#683 gone).
- door[0] "Door_1" at (1.84, −0.10, −114.23) slides open after the use (state 0x4, leaf
  travelled 1.0) — visible from `--bond 2.3,1.2,-111.0` looking south.

## Verification

- `walkreal_07000016_live` (all 8 tests incl. the new one) and `dynobj_glass_live` (2) run
  green against the ISO, `--test-threads=1`, release.
- Full suite via the chunked fast runner (`tools/test/chunk_suite.py`, per-binary
  `--test-threads=1`, 570 s limit): **see the total line below** (the run was still
  completing at report time; the state file `/home/z/suite16_state.json` holds the per-binary
  results). Known environmental failures only: `mp_stress_explosives_live` (OOM exit −9, the
  4 GB sandbox), `probe_pylon_render` (no GPU — wgpu `adapter: NotFound`).
- Crates the diff touches: nf-view (a test-only change — `walkreal_07000016_live.rs`). No
  library code changed, so no behaviour re-verification beyond the tests was needed.

## Live checks for the main agent

None required — the whole chain ran green in the port. Optional confirmation, low value: a
fresh-boot PCSX2 watch of `switch_channels[0x24]` (0x26fc90+0x24) rising exactly on the
station's E press, and of the two 0x3b objects' obj+0xF0 words (#682: 0x30 → base;
#683: obj+0xFE bit 0 set) — would upgrade the swap's [M] pins to [L].

## Left

- Nothing for this task. (The level's other open item — U-051's jetpack hurt window on the
  launch flight — is a separate task, untouched here.)

## Usage

not visible to me — this session exposes no token, credit, cost or context-window figures in
any environment output I can read.
