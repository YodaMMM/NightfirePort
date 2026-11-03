# stun-release-no-stun-anims — report

Branch `s/stun-release-no-stun-anims` (from `origin/master` 6f16146e; master did not move during
the task, so the branch is the final tree).

## What the task asked and what I found

The task's premise (from 107a's ignore note) was that the stun-grenade release chain stalls in
`StunGrenadeImpact` 0x4D for guards "whose anim set has no 0x2d/0x2e stun anims", and asked me to
port `DroneAnim_CallAnim`'s no-datum fallback. Reading the decompile and probing the tables showed:

1. **The no-datum fallback is already ported, complete.** `DroneAnim_CallAnim` 0x13ab58: when the
   requested state is 0 or `DroneAnim_CanDoAnimState` 0x13dfc8 says the set cannot do it, the state
   is substituted (the switch's arms: {0xb,0xc,0xd,0x29,0x2a,0x3d,0x4a,0x4f,0x52..0x56,0x5d,0x5e,
   0x61} → 10, {0xf,0x36} → 0x35, {0x1f,0x62} → 0x1e, {0x22,0x60} → 0x21, {0x25..0x27} → 0x24,
   0x4c → the current anim, 0x57 → 0x11/0x19/0x21, default → 0x21; then 0x24 when even the
   substitute is missing), and a substitute equal to the anim playing now completes the call at
   once (`DroneAnim_CallFullyComplete` 0x13dcb8 → `DroneAnim_SetEndAIState` 0x13dd00). The port's
   `call_anim_variant` (crates/nf-game/src/drone.rs) implements every arm of this, arm-for-arm.
2. **The transition lookup already has the game's retry.** `DroneAnim_GetDAnimForCall` 0x13db30
   retries `DroneAnim_GetDAnimForAnimState` 0x13de40 from anim state 0 when the current anim's
   transition list has no entry (-1); `DroneAnimData::transition` (crates/nf-formats/src/drone.rs)
   does the same `find(cur).or_else(find(0))`.
3. **A call whose transition exists nowhere pends in the game too.** `DroneAnim_CallHandler`
   0x13add0 returns without acting when `GetDAnimForCall` yields nothing; there is no timeout — the
   queued-record bytes CallAnim writes on the way (+0x55A, +0x55E, +0x55F) have no readers in the
   decompile. The port pends identically. (A table probe over every anim state shows the states
   with no transition into 0x2d/0x2e are the transient/death anims — 0x3d, 0x40-0x4a, 0x4c-0x59,
   0x5c, 0x60-0x64, 0x68-0x76 — none of which an alive, flash-reachable guard on 07000005 sits in
   on a fresh load; the level's active guards idle in 0x25/0x26/0x37/0x0e, all connected.)
4. **"Sets lacking 0x2d/0x2e" does not exist at the CanDo level.** A probe of all 24 anim sets
   against `states`+`script_for` (the ported CanDoAnimState law) shows every set can do 0x2d, 0x2e,
   0x21 and 0x76 (three body-only sets also lack 0x24).

So the stall 107a saw was never the missing-anim fallback — the REAL deviations from the decompile
in the stun-grenade family were three state-machine bugs, all fixed here:

## The fixes (each cites its address)

1. **The Rand polarity was swapped** [M 0x172bb0 case 1, dis 0x172c34-0x172c5c]. The game plays the
   fall 0x2E when `Rand_Rand(2)` draws 0 and 0x2D on 1; the port played 0x2D on 0. One draw either
   way, so the shared stream stays aligned — but the anim (and thus the clip length that gates the
   follow-on) differed from the game on half the stuns. Fixed in `handle_stun`'s IMPACT ENTER.
2. **The Loop's wake timer now re-arms with the STORED span** [M 0x1631c0 case 1:
   `+0xCC4 = GameState._52_4_ + (+0xCE0)`]. `DSTATE_StunGrenadeImpact`'s ENTER stores
   `DroneFunc_RecoverTime` into +0xCE0 [M 0x172bb0 case 1]; the Loop ENTER arms a FRESH span from
   the Loop's first tick. The port carried the Impact-ENTER deadline (`stun_until`), ending the
   hold one fall-clip early. New field `stun_recover` (the +0xCE0 stand-in, on its own lines in the
   struct and both literals, per the merge rules), stored before the weapon drop in the case-1
   order; the dart family's `dart_recover` and the taser family's `tase_recover` already followed
   this stored-span shape — the grenade family was the odd one out.
3. **A flash in the Loop re-arms and stays** [M 0x1631c0 case 0x18/0x19]. The port's msg-0x18 route
   re-entered Impact 0x4D for a guard already in the Loop 0x4E; the game's grenade-loop case 0x18
   only re-arms `+0xCC4` (the dart loop's case 0x18 DOES forward to HandleImpact → 0x4D
   [M 0x163570 case 0x18] — `on_flash`'s set_state is right for 0x51 and every other carrying
   state, so the special case is scoped to 0x4E only). The re-arm value is `RecoverTime(msg[6])` —
   the only 0x18 broadcaster is the 0x35's flash arm [M 0x12e0c0], whose RecoverTime arm is 30
   [M 0x147188 case 0x35].
4. **The whole-class audit of the other stun/dart-anim callers** found them already correct:
   `DSTATE_StunDartImpact` 0x163430 (drop bands {5,6,9}, fail 0xE for modes 0x11/0x0c, Rand
   0 → 0x2c else 0x2b, `dart_recover` = +0xCE0), `StunDartLoop` 0x163570 (fresh re-arm, case 7/0x19
   re-arms with the new hit's RecoverTime, case 0x18 → HandleImpact), `StunDartRecover` 0x163700
   (get-up 0x76, follow-on 0x56 for team 1), `DSTATE_Taser` 0x162b98 (Rand 0 → 0x32 else 0x31, the
   30-tick fall wait) and `DSTATE_Stunned` 0x162eb0 (stored-span re-arm, the twitch by the stored
   fall anim). No changes needed there — the port already matched.

## The test

`crates/nf-view/tests/fix_grenade_types_live.rs::the_stun_stuns_the_guard_for_3600_and_drops_his_weapon`
is un-ignored and rewritten. The old staging's guard selection ("a placed guard within 20 u of
Bond's start") predates the flash's delivery law and could no longer be stunned at all on this
master: the flash reaches drones within 20.0 of the BLAST with the head→blast sight check
[M 0x12e0c0 → `Drone_MessageObjVicinity` 0x13a438 + `NDrone2_CanSeeObject` 0x176f18 = the AI-net
ray + `Collide_LineOfSight`], and the only level-start within-20 guard (list[6]) stands 7.6 u below
the blast floor — the geometric LOS drops him, and the game's AI-net ray would too (the floor
separates the cells). Per the rules this is a TEST bug (its selection contradicts the decompile's
delivery rule), fixed by staging Bond 3 u to the right of the placed guard list[2] facing him
(measured: the one offset near a placed guard where the floor holds Bond, the grenade pops and the
flash reaches him), throwing, and asserting the game's chain:

- the guard enters 0x4D with weapon 1 (or the unarmed 4) and no explosion [M 0x1469b8 case 0x18,
  0x172bb0 case 1, 0x12e118];
- `stun_recover` == 3599 — the game's own float path `fptoui(f32(1800)·0.01·(200−0xB9))` truncates
  (f32 0.01 = 0.00999999977648) [M 0x147188:44-46 + the fptoui tail];
- the release chain FINISHES: held > 3600 (the fresh re-arm cannot finish sooner [M 0x1631c0 case
  1]), ≤ 4300 (3600 + the fall and get-up clips, not a stall), and the exit is Attack 0x56 (the
  get-up's follow-on [M 0x163350 case 1]) or 0x66 — Attack's own draw, since the guard holds the
  fists ChangeWeapon(1) gave him and pulls his sidearm on waking.

Measured on this tree: held 4077 ticks, exit 0x66.

`oddjob_hat_live`'s fixture wrote the old `stun_until` deadline to hold its victim; it now writes
`stun_recover` (+0xCE0) — the field the Loop's ENTER consumes since the fresh-re-arm fix.

## What I ran

- `cargo test --offline --release -p nf-game --lib` — 655 passed, 0 failed.
- `cargo test --offline --release -p nf-view --test fix_grenade_types_live` — 4 passed (the
  un-ignored stun test included).
- The sibling stun-family binaries the task named or that share the family:
  `anim_events_live` 2 passed, `fix_stunner_press_live` 3, `stun_ai_recover_live` 2,
  `tranq_dart_live` 3, `tase_scream_live` 1, `tase_silent_live` 2, `oddjob_hat_live` 2.
- The full fast suite (`python3 tools/test/suite.py`): NOT completed in this sandbox — background
  jobs (nohup and setsid alike) are killed between tool calls, and the foreground run exceeded the
  10-minute tool-call window twice (the all-binaries build alone eats most of it). The binaries the
  task touches were run individually instead, listed above; the per-crate sweep the driver's
  command names (nf-game, nf-formats, nf-audio, nf-view with --test-threads=1) ran green for the
  crates the change reaches (nf-game's lib tests and the nf-view stun-family binaries).

## What's left

- Nothing known for the grenade/dart/taser stun families. The ignored-test premise ("sets without
  0x2d/0x2e") is refuted at the table level; if a level ever ships a guard whose CURRENT anim sits
  in a transition-gap state at the flash tick, the game's own law (verified above) is to pend the
  call until that clip's follow-on lands in a connected anim — the port now does the same, and this
  report documents it rather than inventing a timeout.

## Live checks for the main agent

1. The flash vicinity's sight gate: gdbwatch `NDrone2_CanSeeObject` 0x176f18 during a stun-grenade
   throw at the 07000005 start (Bond on the upper floor, the weapon-36 guard list[6] ~9 u away on
   the lower floor at [99.7, 16.8, 101.6]) — EXPECT the guard NOT to enter 0x4D (the AI-net ray
   crosses the floor cell boundary), confirming the port's geometric-LOS stand-in agrees here.
2. The EE rounding: fieldprobe `DroneFunc_RecoverTime`'s return (or drone+0xCE0) on a 07000005
   guard stunned by a 0x35 — EXPECT 3599 (not 3600), the fptoui truncation [M 0x147188:44-46].
3. The Loop's fresh re-arm: gdbwatch drone+0xCC4 when a stunned guard's fall clip ends — EXPECT
   +0xCC4 = (the frame at the Loop ENTER) + 3599, i.e. it moves forward by the fall-clip length
   relative to a deadline armed at the Impact.

## Usage

not visible to me

## ntfy failed

All three notification fallbacks were unreachable from this sandbox:
- `curl -sS -m 20 ... https://ntfy.sh/nightfire-yoda-8k3x` → `curl: (28) Connection timed out after 20002 milliseconds`
- `python3 urllib.request` (the same URL, timeout=20) → no output, killed at the shell's 60 s limit
- `wget -q -O- --post-data ...` → no output, killed at the shell's 60 s limit
