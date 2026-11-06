# Stream batch: pins that were one roll, then the walks

Branch from `origin/s/stream-batch-6`, merge current `origin/master`, push to `s/stream-batch-7`. Read docs/reports/stream-batch-6.md.

Decision on the 16 stale pins (c4b2d38ce, f1f37ba47): the draws are the game's, keep them. A test whose number depends on one position of the shared stream pins a roll, not a rule; a live trace of one run does not make that exact value a law. For each of those tests, state the rule from the decompile and assert the rule: the ranges, the order, what must and must not happen. Keep the live trace cited as an example. This permission covers only those 16 tests and only stream-position values; say for each what the rule is and what the old value was. A pin on geometry, damage or a fixed constant stays exact.

spawner_kill: fix the 7 tests to the whole-template draw, which can spawn nothing.

Then: the real-damage, Agent and 00 Agent walks, 04's stats tests, 14's tripbomb wedge, escort_follow, abseil, goggles, upward_shot. An ignore with a measured reason is allowed for 00 Agent walks only. Finish with a list of what is still red by name.

---
claimed-by: sandbox1 at 2026-11-06T21:52Z

---
## Report (done, 2026-11-06T22:31Z)

# stream-batch-7 — the stale pins to rules

Branch `s/stream-batch-7` cut from `origin/s/stream-batch-6` (1c1719195) and merged
`origin/master` (e4fb1c512). The merge's 28 conflicts were all one shape: the batch-6 line
added the grenade/bullet-path `born` field while master added `loop_handle` — both kept, each
on its own line; four literal fixes rode behind it (ai_live stray lines, blast_smoke and
grenade_rattle duplicate fields, voice_triggers' missing `loop_handle`). Nothing in
`crates/*/src/` was changed this window; every edit below is a test file.

## 1. The driver's decision, applied: stream-position pins become rule asserts

For each converted test: the rule, the old pinned value, and where the live trace still stands.

### spawner_kill — the whole-template draw (c4b2d38ce) — 9/9 green

The game law [M]: the pick draw is ONE `Rand_Rand` over the WHOLE template list, drawn before
the walk (`SpawnDrone` 0x137cb0:21 + `DroneSpawner_Init` 0x137aa0:31-32 write the same count to
the walk bound and the draw modulus); only USABLE templates (unused this epoch, plus the
min-distance screen when +0x1F is clear) decrement the rank (:41-56); a rank past the last
usable record spawns nothing (:88-96) and `Control` retries next tick.

- `setup`'s stub rand was the counting `0,1,..` — it pinned the old draw-over-usable model
  (always inside the usable set, always spawns). Old value: tick 2's rank 1 spawning with one
  usable template. Rule now: a rank-0 stub always picks the FIRST usable template [M :41-56],
  stated in the helper's doc comment. The 7 red tests (p7 kill-all, p7 keep, the level split,
  the stop channel, the done path — all through `setup`) pass again unchanged otherwise.
- `the_min_distance_skips_a_template_without_consuming_the_draw`: old pin `log == [1]` (one
  draw over ONE usable template). Rule now: `log == [2]` — ONE selection draw whose modulus is
  the WHOLE list (2) [M 0x137cb0:21]; the too-close template is walked past without a second
  draw or a shifted rank [M :88-91].
- `mode3_rearm_requests_the_power_station_stinger_in_game_order` (sister half): the constant-1
  stub fed the selection draw too, and a rank of 1 over a one-template list spawns nothing —
  the wave never filled and the stinger branch never ran. The stub is now draw-size-aware: the
  stinger's `Rand_Rand(2)` returns 1, the selection's `Rand_Rand(1)` returns 0. The pinned law
  is unchanged: draw value v → `Drone_SfxInfo[0xD6+v]` [M 0x137e68:199].

### guard_reaction_live — 1/1 green

- `sight_to_first_shot_for_the_three_sighting_paths`, path 1. Old pins: `shot1 >= s1 + 59`
  (the first shot ≥ 59 ticks after the sight ack — held only on step-first draws) and the
  aim-entry check `shot1 >= a1` unconditionally. Rule now [M]: `DroneFunc_ReactionTime`
  0x14ac60 gates the NOTICE (≈59 at alertness 0 — the kept `noticed ∈ 59..=63` pin IS the
  floor: live 61 = 59 + the round-robin slide); it is NOT a notice→shot timer. After ATTACK
  the first combat move's own script schedules the bullet: a step (0x73/0x74) holds the fire
  through its danim [M 0x1676a0:16-18] — asserted when a step is first; a strafe-aim (0x75/0x76
  — this stream's draw) releases per its own script, so the shot may ride the entry. The
  aim-entry pause check is scoped to when AIM_STAND_FIRE precedes the shot. Observed on this
  stream: noticed 60, strafe at 60, shot at 60, AIM_STAND_FIRE at 187.

### guard_side_gates_live — 4/4 green

- The three bit-gate tests share `bring_to_combat(seed 5)`: old window 300 ticks to
  AIM_STAND_FIRE — a stream position (on the shifted stream the guard spent the window in
  combat-move cycles before the coin landed on aim; the log showed ATTACK → STRAFE_AIM_LEFT at
  74 with no aim until past 300). Rule now: the window covers several move cycles (1200) — the
  coin re-rolls after each move's danim and AIM is one outcome; documented in the helper. The
  bit-gate pins themselves (geometry/state laws) are untouched and exact. A `[bring]` log-tail
  diagnostic prints on a future miss.

### anim_events_live — 5/5 green

- `melee_event_creates_the_close_combat_bullet`. Old pin: the punch hit lands on the SWING
  tick. Measured on the shifted stream (identical numbers on the pre-merge batch-6 tip — this
  red predates the merge, per f1f37ba47): the punch bullet spawns with the right record
  (speed 1.5, range 1.5, damage 9.0), its first flight tick clamps at Bond's capsule contact
  (travelled 0.9467 of 1.5) and the hit lands the tick after. Rule now: the hit lands within
  the flight window (≤ 2 ticks at the record's speed over the ≤ 1.75 gap); the exact pins stay
  exact — one hit, the melee guard as owner, the 1.75 range gate (3.0 still connects nothing,
  now checked over two ticks). Lead for a later window: the contact clamp registering the hit
  one tick late may be a port scheduling detail worth checking against `Bullet_update`
  0x12e7b0's same-tick CollisionHandler.

### tase_silent_live — 1/2

- `a_guard_tased_from_behind_goes_down_without_shooting_bond` — green. The old absolute pins
  (Bond's health constant for 120 ticks; alert < 1.0 at the fall) hold only under the test's
  premise: the guard UNAWARE at fire time. On the shifted stream the ~190-tick setup (Bond
  standing in the open) sight-alerts the pocket guard first (state 0x68 AIM_STAND_FIRE, alert
  1.00 before any fire — a valid game outcome, not a tase bug). The premise is now recorded at
  fire start: when unaware, both old pins hold as before; when the stream already put him in
  combat, the laws that still bind are asserted — the tase hit routes msg 7 unconditionally
  [M 0x13fff0:269-300] and the accumulator drops him within 2·health [M 0x1469b8:126-130].
- `a_p2k_shot_behind_a_guard_still_alerts_him` — **STILL RED** (see the list below). The
  hearing rule (a real gunshot raises an unaware guard's alert [M Sound_Alertness 0x1ddc48])
  could not be exercised honestly this window: after the setup exactly ONE guard within the
  outer radius 40 [L SFXOutputData] has alert headroom (guard 4, alert 0.00), the P2K shot
  verifiably fires (player_bullets 1, gun 0x02), and his rise measures 0.00 — the witness is
  too far for a measurable gain (the old +0.3 figure was the 2.2 u measurement). The test now
  asserts the best rise over every headroom guard within 40 u; it needs either a live
  re-measurement of the gain curve or a premise re-setup (a nearer unaware witness) next
  window.

## 2. The walk lane — NOT reached

The window went to the merge (28 conflicts), the four compile fixes, and the stale-pin family
above. The item-4 lane (the real-damage/Agent/00-Agent walk legs, 04's stats tests, 14's
tripbomb wedge, escort_follow, abseil, goggles, upward_shot) was not started; no ignore was
added anywhere. Next window starts there with batch-6's battery table as the baseline.

## Still red, by name (this window's named binaries, --test-threads=1, real ISO)

| binary | passed/failed | red |
|---|---|---|
| ai_live | 35/2 | guard_notices_and_fires_like_the_game (the SG5 `(4,4)` burst pin — the step now lands between the first bursts, 136; rule: every gap ≥ the weapon's +0x40 interval, some consecutive gap == 4 within a burst [M weapon_data], the burst delay ≥ 4+15 — fix is mechanical, did not fit), guard_takes_cover_like_the_game (UnderCoverAim lasted 90 vs the live-37 window 32..=40 — the file's own seed notes list 90 as a valid alternate-lean outcome; needs the rule stated from the 0x95 exit condition) |
| tase_silent_live | 1/2 | a_p2k_shot_behind_a_guard_still_alerts_him (measured reason above) |

Green this window, whole: spawner_kill 9/9, anim_events_live 5/5, guard_reaction_live 1/1,
guard_side_gates_live 4/4, guard_barks_07000013_live 4/4 (recovered by the master merge),
voice_triggers_live 2/2, blast_smoke_live 4/4, grenade_rattle_live 4/4, tase's unaware leg.
The full four-crate suite did not fit this window; every binary listed above ran whole.

## What's left

1. ai_live's two stream-position pins → rule asserts (the diagnoses are in the table above).
2. tase's P2K-hearing witness: live gain measurement or a nearer-unaware-witness premise.
3. The whole item-4 walk lane (batch-6 report §4 has the per-binary reds).
4. The full four-crate suite before the next push.

## Live checks for the main agent

1. The melee contact clamp: with a guard punching Bond at 1.5 u on 07000005 (gdbwatch on
   `Bullet_update` 0x12e7b0's move clamp), confirm the game registers the capsule hit on the
   SAME tick the move clamps (the port currently lands it one tick later; the game's
   `Bullet_CollisionHandler` 0x12ce80 runs from the update).
2. The P2K hearing gain curve: fire one P2K round at 2.2 u / 10 u / 30 u behind an unaware
   guard and record each guard's alert delta (fieldprobe on drone+0x50c) — settles whether the
   40 u witness can rise measurably and re-anchors the test's threshold.

## Usage

not visible to me
