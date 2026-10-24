# WORKLOG

Entries appended at the end, one per task. Do not rewrite earlier entries.

---
Task ID: level-06-castle-fire
Agent: sandbox1
Task: 07000006 held/repeated fire: first press spawns the round but turns the view -1.38 rad from the aimed yaw; every later press spawns nothing (12 presses -> 1 bullet). Reproduce with real input, find the cause in the fire path [M Player_SetFiringAnim 0x1a3c70, Player_WeaponFiring 0x1a5460, Player_SetWeaponAnimObj 0x1a2e38] vs the port's Gun::update_with and the fire-script/state handling in crates/nf-view/src/scene.rs, fix, pin with tests that fail before the fix, un-ignore the two 06 walks.

Work Log:
- (started) Cloned repo, created branch s/level-06-castle-fire from origin/master at 26543436 (batch-1023j).

Stage Summary:
- Branch created; setup and investigation in progress.

Work Log (continued):
- Reproduced the fire path through real input on 07000006 at two fight sites (guardhouse courtyard, searchlight-courtyard mouth past the boot NIS): every press fires (12 presses → 12 rounds incl. a mid-battery real reload), the press-tick yaw is unchanged, held fire with the semi-auto is exactly 1 round. The walkreal doc's measured wedge (−1.38 rad press-tick turn, later-press starvation) does NOT reproduce on current master — the pins are kept as regression tests (fix_06_fire_live.rs) with the [M]/[L] citations.
- Added the held-automatic rate pin to gun.rs (weapon 0x12: spp 999, interval 6 [L ee_start_05.bin weapon_data]).
- The two 06 walks: route improved with the guardhouse fight and the through-door #59 waypoint (the auto-closing doorway is a nav blind spot) — the walk now runs the truck ride → the well lane → the guardroom → the whole searchlight courtyard → door #71 → the tunnel to y 46.6, and wedges at the tunnel ramp's upper half (measured fallbacks: dense jump pulses, crouch-walk, re-center — all pinned at (5.45, ~46, −59.04)). Re-ignored with honest notes; the tunnel geometry is the next question, not the fire path.

Stage Summary:
- fix_06_fire_live.rs: 3 real-input pins, all green. gun.rs held-automatic pin green; nf-game suite green.
- walkreal_07000006_live.rs: route legs added, tests stay [ignore] with updated evidence (fire path proven whole; tunnel ramp open).
