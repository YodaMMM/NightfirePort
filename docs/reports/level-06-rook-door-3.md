# level-06-rook-door-3 — the 01 contact kill attributed: the harness's own volley, and the walk taught the player's line discipline

Branch `s/level-06-rook-door-3` (from `origin/s/level-06-rook-door-2` 4c3edcc3, merged `origin/master`
e6e3c899). No port-code change this round — the kill was not a sim deviation. One harness change
(`crates/nf-view/tests/common/walker.rs`) and the two 01 walkreal test files carry the fix.

## The kill, attributed (the round's question)

`walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact` went red on door-2's tree: the
contact (drone **#1**, skin 0x0500004f, team 2, the AllyLead machine, mode 12, dtype 12) died during round 0
of the escort loop, and door-2's report left the killer unattributed ("the sim keeps no killer record").

**Method.** A new env-gated harness hook, `NF_HEALTH_TRACE` (walker.rs, additive — the NF_DRONE_WATCH
pattern): every drone's health is diffed each tick, and a change prints the drone's identity (skin/team/
state/mode/alt_state/dtype/position), its `last_damage`, its `hit` HITDATA (part, ray, point, damage,
weapon), the `hit_by_player` stamp (+0x2B4 [M 0x145bd0]), its `blast`, its drone-opponent (+0x170), and the
live in-flight bullet owners. The trace turns a death into a reading instead of a guess.

**The reading** (the red run on door-2's tree, reproduced bit-exact):

- `t137198 d#1 skin 0x500004f team 2 10.0→−2.4 st 0x44 mode 12 dtype 12 at (−5.14, 6.34, 14.25)
  hit Some(Hit { part 53, ray [6.3875, −0.6105, 0.0094], point (−5.27, 5.76, 14.33), damage 3.0,
  weapon 28 }) by_player TRUE opp Some(14)`
- Same tick, same volley: `d#5 skin 0x5000039 10.0→−3.5 ... ray [6.3866, −0.6083, 0.1229] ... weapon 28
  by_player TRUE` — **the killer is BOND.** The two rays differ by the spread; weapon 28 = 0x1c is the
  8-pellet spread-32 gun (pellets +0x18 = 8, spread +0x24 = 32, damage +0x0C = 3.0
  [L ee_start_05.bin weapon_data 0x2bf150; the record layout + per-pellet spread
  M Bullet_init 0x12c5a0:148-180 — max deviation 32·0.0014 = 0.0448 rad]). One pellet of the volley aimed
  at guard #5 flew 0.16 off the aim line at the contact's distance and hit the walking contact's head part
  (the hit point y 5.76 is feet−0.58: the cowering hide boxes hang BELOW the feet origin).
- **No guard shot him.** The port's drone-fired bullets test world + Bond's cylinder only
  (crates/nf-game/src/drone.rs 17292-17472 — the drone list is never in the pass), so a guard could not
  have. The health trace over the whole red run shows every other hit on every drone is `by_player true`.
- The game-faithfulness check the task asked for: the game's own ballistics would do the same —
  `Drone_CollisionHandler` 0x140968:85-95 hands **every** HITDATA contact on the object to
  `NDrone2_DealWithObjHit` with **no team gate** (the only upstream drop is `Collide_PickObj` 0x1eae80:26-30
  removing the disabled `+0xF0 & 0x20` objects). A player who fires an 8-pellet gun down a corridor with
  the escort in the cone hits the escort in the game too (and the mission layer — not ported — would fail
  him for it).

**The mode-2 guard side question** [M]: the creation-mode-2 guard #2 at (−29.73, 10.22, 13.66) — does he
leave 0x29/0x2a and fire into the hall in the game? Yes, on the game's own chain: SniperAim 0x2a's UPDATE
(case 3) calls `SM_SetState(0x2b)` when the seen bit (+0x228 & 4) stands [M 0x15fc18:28-40], and SniperFire
0x2b fires via `DroneWeap_Fire` on the anim's fire event (msg 0xc) [M 0x15fd88:76-79], returning to 0x2a
when +0x274 ≥ 0x10 [:50-58]. The port's #2 walks exactly that on the merged tree (NF_DRONE_WATCH=2):
0x0 → 0x1 (+48) → 0x29 (+2064) → 0x2a (+2242, the aim micro-steps −30.06→−29.73) → **0x2b (+140057) → 0x2a
(+140193)**. His behaviour is not a deviation; he was never the killer (see above).

## The fix — the WALK plays it the way a player does

The kill is game-faithful ballistics, so per the task the WALK changes, with real input only
(`crates/nf-view/tests/common/walker.rs` + `walkreal_07000001_live.rs`; no sim code touched):

- `Walker::shoot` gains an opt-in **friendly-fire line gate**: `pub shield: Option<usize>` (default `None` —
  every other caller unchanged). When set and the shield drone is alive, a tap is REFUSED when the aim
  line, extended past the target to the shield's distance, would cross the shield's cylinder — the xz miss
  under `0.9 + 0.05·ds` (the fixed 0.9 plus the pellet spread's reach 0.05/dt·ds ≈ 0.045 rad) and the
  extended y inside feet−1.2 .. feet+2.2 (the cower boxes hang below the feet origin — the first, tighter
  gate missed the kill line for exactly that reason, run 2 in the worklog). The refused tap falls into the
  loop's existing `blocked` arm: the strafe (the real player's step off the line) re-aims, and the flank
  nav still applies. `walkreal_07000001_live` sets `w.shield = Some(ci)` once after the contact is found;
  every clearing loop in the walk inherits it through `shoot`.
- `NF_HEALTH_TRACE` stays as the evidence hook (env-gated, additive).

**Result**: `walkreal_07000001_live` is green — all 3 tests (the contact walk reaches the plate, the AND
fires, level_exit to 07000002 at tick 275304; the plate-refusal and the hall-guard sisters pass), and the
contact's health trace over the whole walk is **empty** — he is never touched again.

## Verification (what ran, on the merged tree e6e3c899)

- `cargo test --offline --release -p nf-view --test walkreal_07000001_live -- --test-threads=1`:
  **3 passed / 0 failed** (the contact walk + both sisters), with the flush still removed.
- `rook06_realpath_live` **un-ignored and green** (1 passed): the ch-16 guard keeps his placement mode 0
  through the whole engagement, the doorway watch (1800 ticks) ends clean — no drone parked in the door
  #47 jamb, the door ends at the shut idle 0x2. Its `#[ignore]` note is replaced by the landing note.
- The neighbour families, one invocation each (`--test-threads=1`): escort_cover 3, escort_follow_07000001 1,
  escort_hide_findrun 3, fix_06_alarm 2, fix_06_driver 2, fix_06_fire 3, fix_mission 4 (incl.
  `the_07000001_exit_works_end_to_end`), rook_absent_06 1, rook_door_06 1, rook_door_06_walk 1,
  rook_sp_06 1 — **22 passed / 0 failed**.
- `cargo test --offline --release -p nf-game --lib -- --test-threads=1`: **547 passed / 0 failed / 3 ignored**.
- nf-formats + nf-audio: 28 + 105 tests, the 3 `disc_golden` failures are the allowed slim-ISO set.
- The full four-crate suite on the merged final tree: see the fast-runner total below.

## What's left

Nothing from this task's list. Candidates for a later round (not started, no code):

- The mission-fail layer for the escort's death (the fail_label that should fire when the contact dies) is
  not ported; the walk now avoids the kill by play, but a player who shoots Mayhew in the port sees no
  fail. Needs its own evidence pass.
- Guard-fired bullets still never test against drones in the port (the game's Collide pass can route a
  guard's round into any drone, the contact included). The 01 live pressure question (door-2's report
  "Live checks" item 1) stands for the main agent.

## Live checks for the main agent

1. Fresh boot of 07000001, Bond at the hall exit, corridor guards unshot for ~60 s: does any guard's round
   hit the contact (skin 0x4F) — i.e. does the GAME route guard bullets into drones (the port does not)?
   A gdbwatch on the contact's +0x144 (health) during the window decides.
2. The mode-2 guard at (−29.73, 10.22, 13.66): a gdbwatch on his state word during the same window — the
   port shows 0x2a → 0x2b → 0x2a at +140057/+140193; confirm the game's sniper fires down into the hall on
   sight the same way [M 0x15fc18:38-40 + 0x15fd88:78].

## Usage

not visible to me (no token, credit or context-window readouts are exposed in this environment).
Session wall-clock: branch created 03:36 UTC from origin/s/level-06-rook-door-2; the sandbox window is
~03:31–05:31 UTC.
