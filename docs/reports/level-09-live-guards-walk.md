# level-09-live-guards-walk — the 09 walk with the guards live (no hold_npcs)

Branch `s/level-09-live-guards-walk` (from origin/master 814c8976, merged with origin/master
0c90eba8 before the final push). Sandbox 04:45–06:45 UTC.

## What was done

**The walk passes WITHOUT `hold_npcs` on real input.** `walkreal_07000009_live::
walk_tower_a_start_to_finish` now runs the whole 07000009 mission with the guards live:
car park → the stairwell (the pinch guard patrolling mid-stair) → the lobby → the real panel
E-press (objective 2 completes) → the cab → the exit fires to 0700000a — **the exit fired
28 ticks into the cab walk, 216,228 ticks start-to-finish, 0 stuck, 0 falls, and no alarm
channel (0x61/0x7C/0x7D) ever rose** — the walk's own no-alarm asserts are the proof the
patrol beats were cleared in time. All 4 tests in the file green on the final merged tree
(179 s): the walk, the elevator-activation pin, the alarm-switch fail arm, and the new dart pin.

### The root cause of the open item (the challenge-state guard's zero damage)

The level-09-every-objective report left: the stair-head pinch guard, holding
CIVILIAN_CHALLENGE, took zero damage from 15+ point-blank rounds across three runs. Three
layers, all verified this session:

1. **The 15 rounds were the K5 sleep darts.** 09's loadout opens on weapon 0x43 — clip 5 +
   spare 10 = exactly 15 (the level's stealth start; the load dump prints `weapon 0x43 clip 5
   spare 10`). Darts damage nothing BY DESIGN: `NDrone2_DealWithObjHit` 0x13fff0:269-306 routes
   0x43/0x44 to msg 0x19, whose handler (`DroneFunc_HandleImpact` case 0x19, 0x1469b8:315-361)
   never reaches `NDrone2_HitDamage`. Not a pick gap.
2. **Real port gap (fixed):** `Drone::on_dart_impact`'s whitelist lacked CIVILIAN_CHALLENGE —
   `NDrone2_DSTATE_CivilianChallenge` 0x162558:40-46 forwards its {7, 9, 0x17, 0x18, 0x19}
   group to `DroneFunc_HandleImpact(…, 0x89, 1)` → case 0x19 → STUN_DART_IMPACT 0x50. The old
   whitelist returned unhandled → GLOBAL → dropped: no sleep, no flinch (the observed
   symptom). After the one-arm fix the challenge-state guard **falls asleep to a dart with
   health untouched** — pinned by the new file test
   `the_challenge_state_guard_sleeps_to_a_dart_and_loses_no_health` (challenge at +2447 via
   the real sight chain, K5 fire through the real path, state 0x51, hp unchanged, no alarm).
3. **The "mid-stair geometry shadow" hypothesis was real but situational.** From the stairs
   BESIDE the guard the P2K kills through the real pick (verified point blank); from BELOW
   (the car-park floor, the guard 4u up the stairwell) the stair geometry itself eats the
   ray — that is why the first approach designs missed.

### The port fix

`crates/nf-game/src/drone.rs` — `Drone::on_dart_impact`: added `CIVILIAN_CHALLENGE` to the
case-0x19 whitelist, cited to 0x162558:40-46 (the decompile group) — one additive arm, no
other behaviour touched. With the fix, a dart on a challenging guard sleeps him instead of
dropping; the sleeping guard then dies to ANY bullet regardless of health (the dart-family
DSTATEs' own msg-8 arm 0x163430/0x163570/0x163700 case 8/9 → 0x44 Death_Anim) — the stealth
takedown chain the level stages.

### The live-guard walk machinery

- `crates/nf-view/tests/common/walker_b2.rs` (append-only): `shoot_bounded(di, label, cap)`
  (shoot with a caller tick cap; `shoot` keeps its 10800-tick behaviour) and `shoot_loose`
  — the aim gate widened to ±0.07 yaw / ±0.10 pitch, because a PATROLLING guard at ~2u sweeps
  more than the tight ±0.02 gate between ticks (the first run's 15-dart miss at 1.8u).
- `crates/nf-view/tests/walkreal_07000009_live.rs`: `cycle_to_lethal` / `cycle_to_darts`
  (the real weapon wheel), `approach` (nav-graph closure to ~2.2u — the direct XZ steer
  stalls at the stair base), `takedown` (the silent dart sleep; the loud direct fire only
  when darts run dry), `clear_live_guards` (the patrol-beat clear: ever_seen latched, the
  alarm-bound states, the heard-noise investigators 0xa3-0xa5 prioritized over the 15u-bubble
  pre-sight picks; sleepers excluded; a drought breaks the clear), `walk_nav_clearing`
  (walk_nav_to with a clear after every node). The K5 is the takedown tool all walk long;
  the P2K stays the dry fallback — both through the real fire path.

## How it was verified

- The walk file: 4/4 green on the final merged tree (`test result: ok. 4 passed; 0 failed`,
  179 s, the binary run directly).
- **The touched binaries' slice (suite.py `--only`, final merged tree): `625 passed, 0 failed,
  7 ignored  (4 min)`** — binaries: nf_game (the crate units incl. the damage/dart machinery),
  walkreal_07000009_live, interact_07000009_live, security_alarm_09_live,
  elevator_buttons_09_live, lift_indicators_09_live, census_07000009, hold_npcs_live,
  tranq_dart_live, stun_ai_recover_live, fix_stunner_press_live, mission_outros, progress.
  Zero FAILED/TIMEOUT/CRASH lines — the dart whitelist fix broke none of the pinned dart/stun
  behaviour, and the whole 09 family stays green with the live-guard walk in.
- The full suite (the fast runner) restarted on the final merged tree after the merge
  (build ok: 915 test binaries in 1109 s); it needs ~2–3 h here and ran on past the report —
  its `suite.out` total line is the next session's first read.
- Evidence discipline: every game-behaviour assertion in the new test code carries its [M]
  decompile address; the two [L] facts (the loadout dump, the scratch kills) are marked as
  live runs in comments; the walk's stealth tuning choices (the dart-first order, the bubble
  radius) are port choices, not pinned game behaviour.

## What's left

1. The full-suite tail on the final tree (the fast runner needs ~3 h here; the slice above
   covers the touched families).
2. A `SeenDeadBody` discovery walk (a corpse found by a patroller — investigate, no alarm)
   is untested; the dart-only walk leaves no corpses on the route by design.
3. The other DSTATEs whose decompiles carry case 0x19 but sit outside the dart whitelist
   (INTEROGATE 0x161a40, INTEROGATE_ASSIST_WAIT 0x161860, CIVILIAN_MISSION_WAIT 0x15dbb0,
   HOSTAGE_KILLER 0x15c728/0x15c940 and the smoked/bot-cover families per a case-0x19 scan)
   — same one-arm shape as this fix, each needs its own failing test first.

## Live checks for the main agent

1. Whether the game's own PCSX2 guard, challenged (state 0x3E) and darted, falls asleep with
   no damage — the port now says yes (0x162558:40-46 + 0x1469b8:315-361); a fresh-boot
   `gdbwatch` on the guard's DSTATE word while firing the K5 would settle it end to end.
2. The wake timer: `STUN_DART_LOOP`'s RecoverTime on a 09 guard (watch `+0xCC4`/`+0xCE0`)
   — the walk assumes the sleep outruns the walker by minutes.

## Usage

My own session's token/context/cost figures are not visible to me (no such readout in this
environment). The only numbers I could read were per-command shell readouts, not session
totals — none are quoted rather than estimated.
