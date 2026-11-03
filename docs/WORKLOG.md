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
- [06:00 UTC] Root cause of the ignored test found: the old harness forced fields on
  `state::SURRENDER_ANIM` whose ENTER drops the weapon and `ChangeWeapon(1)`s [M 0x1626a0:21-22]
  — and a raw `.state =` write bypasses the ENTER machinery entirely (`set_state` queues the
  real EXIT→ENTER; the raw write changes only which UPDATE arm runs), so `vision_on`/`firing`
  never came from the machine and the shot never stood due. Rewrote the harness to drive a
  placement guard through the real chain: 07000001's weapon-0x1C shotgun guards (recoil, [M
  DefaultInit 0x14b300:800-811]) and weapon-0x0E pistol guards, Bond 5 u away in the open with
  the guard's view cone, `set_state(IDLE)` as the stand-in for the mission layer's enable
  (WAIT_SWITCH's ENTER is `NDrone2_Enable(0)` [M 0x15b960:14-16]; Idle's ENTER opens vision [M
  0x15bf18:30]) — every tick after that is the real perception → combat chain.
- [06:10 UTC] All three rows pass on real ticks: the recoil guard holds a due shot with the
  event clear (the recoil anim's 0x10 clears +0x21 [M 0x13b750:94-95]) and fires on the 0x12
  event with the 0x5C recoil danim + +0x57B [M 0x13da48:20-29]; the non-recoil pistol guard
  fires through the +0x3E == 0 early-out without any event [M 0x13da48:10-12]. No port change
  needed — the gate in `handle_firing`/`fire_now` already matches the game (DoFiring 0x17a080
  sets the +0x3F due latch per pass and holds it for +0x3E weapons; the port's inline
  `next_shot <= frame` fold is equivalent and nothing in the state machine resets it).
  anim_events_live: 5/5 green. Test only — no source change.
