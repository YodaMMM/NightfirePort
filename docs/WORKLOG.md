# WORKLOG

## guard-recoil-firing-anim

Branch `s/guard-recoil-firing-anim` created from `origin/master` (6f16146e).

Task: un-`#[ignore]` `crates/nf-game/tests/anim_events_live.rs::play_firing_anim_gates_the_recoil_weapon_shot`
and make it pass: a guard whose weapon has the recoil flag (+0x3E) with a shot due must fire only on the
anim event +0x21 and play the 0x5C recoil (DroneAnim_PlayFiringAnim 0x13da48). The harness never got a shot
due; find the real attack-state UPDATE path that reaches PlayFiringAnim with Bond in range, drive the guard
there with real ticks, and fix the port if it resets the due shot where the game keeps it.

Progress:
- [05:20 UTC] Branch created, WORKLOG started. Sandbox was reset; redoing setup (work-slim + ISO downloads next).
