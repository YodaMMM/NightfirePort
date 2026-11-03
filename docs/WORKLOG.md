# WORKLOG

## stun-release-no-stun-anims

Branch `s/stun-release-no-stun-anims` created from `origin/master` (6f16146e).

Task: un-`#[ignore]` `crates/nf-view/tests/fix_grenade_types_live.rs` (~line 204, 107a) and make the
stun-grenade release chain finish for guards whose anim set lacks the 0x2d/0x2e stun anims. The stun arm
works (state 0x4d, weapon 1, stun_until = frame + 3600, TIMER_C fires); the release stalls in 0x4d —
guard 0 reaches the Loop, the chosen guard never leaves Impact. Read DroneAnim_CallAnim's no-datum
fallback in the decompile (what the game does when the requested anim is missing from the set: skip,
substitute, or end the clip at once) [M], port it, and fix the whole class (stun dart, stunner states
that call stun/dart anims).

Progress:
- [05:59 UTC] Branch created, WORKLOG started. Setup reused from sandbox (work/, ISO, nf-queue present).
