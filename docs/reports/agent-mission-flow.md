# Mission flow audit (s/agent-mission-flow)

Scope: `Mission_MonitorObjectives` 0x1827d8, `Mission_Init` 0x182270, `Mission_Update` 0x182c60,
the fail/death menu `P_ENDMISSION_Handler` 0x216618, the elapsed clock, the warm-load carry
(`Player_InitWeapon` 0x1a7ef8 / `Player_RamLoad` 0x1957a8), on every on-foot level. Ally-death
fail reasons are out of scope (another branch).

## Fixed

1. **No pause once the mission is over.** `Mission_Update` writes `GameState+0x24 = 1` in case 2
   (fail), case 3 (death hand-off) and case 6 (success) [M 0x182c60:46,60,88]; only
   `ResetMap_GameInit` clears it [M 0x1d5080:9]. `Player_Update` opens the pause page 0x4000004b
   only while it is 0 [M 0x193600:72]. The port opened the pause over the "Mission Complete" and
   "Mission Fail" banners and after the death hand-off. Now `Mission::ended` mirrors +0x24 and
   `nf_view::pause_allowed` gates the Esc arm.
2. **The results' Time row stops at the end banner.** `PlarStat_LogUpdateElapsedTime` is called
   from `Player_Update` 0x193600:132, behind the `GS_IsPaused(player)` return [M :88-91];
   `Mission_Update` case 7's `GS_PausePlayer(1, 0)` [M 0x182c60:124] raises it from the hold's
   first tick, and the hold's `GS_PauseGame(1)` [M :127] keeps it to the next load
   [M 0x1d5080:16]. The port kept counting through the 240-tick "Mission Complete" banner, so
   every completed mission's Time row read 4.00 s long (and its time score was lower).
   `Mission::player_paused` gates both port call sites (drone.rs and scene.rs).

Tests (07000005, real load): `pause_results_flows_live::the_pause_refuses_once_the_mission_has_ended`
(red with the old gate: the pause was allowed under the Mission Complete banner) and
`::the_mission_clock_stops_under_the_mission_complete_banner` (red on the old code: 220 → 621
hundredths across the hold).

## Checked, no divergence found

- The monitor's five cases, the standing 0x60/0x61/0x7b/199 hooks and the all-AND, line by line.
- `Mission_Init`'s part classes (previous kept, current/future reset), the per-level
  `LevelToEndTo` table, the fail/death menu rows, the restart's entry checkpoint and health floor.
- Per-difficulty objective sets: `MissionData` has no difficulty field; the difficulty readers
  (`Drone_Create`, `Lock_Create`, `Hint_Create`, `Copter_Create`, `SoundTrigger_Create`) are all
  ported with their gates.

## Structural difference with no observed effect

`Mission_Init` clears the current/future parts' condition channels, but on the warm levels whose
`Player_InitWeapon` case calls `Player_RamLoad` (02/03/04/06/07/08/0a/0b/0d/12/13/4a/15/16/17
[M 0x1a7ef8]) the RamLoad runs later in the same load [M ResetMap_Load 0x1d6360:133 then :154] and
puts the exit channels back. The port's init runs after its channel restore, so such a channel
would stay cleared. A probe over `campaign_flow_live`'s real chains found no channel that this
clears on any warm arrival, so nothing was changed.

## Unproven

The `GameState+0x24` reader in `MenuManager_Monitor` 0x21d508:104 (a controller-message branch)
is not ported; not player-visible on PC.
