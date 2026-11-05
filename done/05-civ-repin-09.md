# Civilian alarm walks: 09

Branch from `origin/s/civ-alarm-repin-2`, push to `s/civ-repin-09`. Read docs/reports/civ-alarm-repin-2.md first: it has the measured cause for each walk and the two fix attempts that failed. Test-side only. One task only: touch only the walk file named here. Do not loosen a pin tagged [L].

walkreal_07000009_live: 5 red at the "must not raise the alarm" assert. A guard who acks reaches the panel in about 670 ticks [L PCSX2]. Play it as stealth: dart each lobby and stair guard before the ack (35 ticks of held LOS at reaction 80) or keep out of sight.

Real input, no teleports. Run the binary whole and quote the result. If the window closes, push what is green.

---
claimed-by: sandbox3 at 2026-11-05T21:17Z

---
## Report (done, 2026-11-05T22:21Z)

# civ-repin-09

Scope kept: `walkreal_07000009_live.rs` only (test-side, the one named walk file), no [L] pin
loosened, no shared helper touched. The five reds at `:133` ("the walk to the panel must not
raise the alarm") stand — the session rebuilt the walk's whole takedown machinery on a measured
mechanism and closed most of the press-race gap, but the last ~100 ticks were still missing when
the window closed. The branch ships the rebuilt helpers plus this report; the next session lands
the finish with the recipe in "Left".

## The mechanism, measured on the real path (all [L] this branch's runs unless tagged)

The alarm race is not lost to a sight latch at all — it is a **civilian relay**, and the trace
(NF_ALARM_TRACE, run 6) pins it tick by tick:

1. **t1223 — g4 challenges**: `0x14 → 0x3e`, `seen true los true`, d 6.8 — g4's own patrol
   descends the stairwell into the car park and meets Bond's stair-base approach head-on on the
   same floor. The meeting is unavoidable on every route: the stair base is on the path to
   everything (the spawn → stair base → the stairwell's lobby door is the graph's only way up).
2. **t1224 — g3 relays, ONE tick later**: `0x14 → 0x63` with `alarm_run = 1, ever_seen = 0`
   — g3 never sees Bond the whole run (`seen false los false` on every traced tick). The
   challenged guard's challenge instantly arms the teammates' RunToAlarm. A racer in 0x63
   (GoToGoalPosition — also the patrol's own nav state) is INVISIBLE to a `live_state`-only
   filter; the old beat clears ignored him for his whole run.
3. **g3's target**: `FindAlarmPoint`'s assigned switch `alarm_point = (−0.82, −369.42, 13.92)`
   — read off the live Drone [M `Drone+0x738` pair, 0x153008:26-30]. He ENTERS PressAlarm
   (0x8a) at **t1972** at (0.2, −368.43, 15.6) — his run took 748 ticks — and the channel write
   lands at **≈ t2182** (the 210-tick PressAlarm fuse [M 0x168ea8:30-31, 56-63]). Latch→press
   ≈ **958 ticks**.
4. **The door tax**: the nav graph's only stairwell→lobby connection runs through the door at
   (18.8, −368.4, 39.0); a chase arriving at it CLOSED pays **~375-444 ticks** of opening
   animation mid-race (runs 5-7, the chase's door node `+375`/`+444`/`+390`). A chase from the
   stair base = stairwell climb ~435 + lobby east ~255 + door ~375 ≈ 1065 ticks against the
   ~700 available after the g4 takedown. That margin is the whole failure.

Also measured: the old dart volley was its own budget burn — `shoot_gated` early-returns only
on corpse/health, never on the dart SLEEP, so a 320-tick volley fired half a clip into an
already-sleeping guard while the next guard's race ran; and the darts land point-blank only
(the run-1 baseline: 15 darts, zero hits at 6.5 u; the in-flight dart lands ~25-40 ticks after
the tap and the caller must re-read to see the sleep).

## What shipped (the rebuilt helpers, walkreal_07000009_live.rs only)

- **`racer(d)`** = `ever_seen || alarm_run || live_state(state)` — the alarm-run flag (+0x16
  [M 0x1736f8:19]) is the only tell a 0x63 relay-racer carries.
- **`chase_to`** runs to the racer's **`alarm_point`** (his switch — static, a straight nav
  plan) through whole nav legs at the walker's own pace, re-checked per node; the earlier
  direct-steer chase crawled at ~1.5 u/s into stairwell geometry between one-node hops and lost
  from 12.5 u (run 2-4).
- **`shadow_to`** closes on a quiet guard from BEHIND his ±1.57 rad half-fov cone [D props] so
  he never latches at all; bails the tick he latches (the chase takes over).
- **`takedown`**: dart volleys cut to **80 ticks** with a re-read after each (the in-flight
  sleep is caught on the next loop); an **emergent racer elsewhere bails** the current
  takedown (`other_racer`); volleys only at ≤3.4 u point-blank; darts dry → the caller
  resupplies, and only a real drought falls back to the loud P2K.
- **`clear_live_guards`**: the candidate set adds `racer()` and the latching class
  (`seen_ticks > 0` — inside the 12 u + FOV gate this tick, before the ack), racers first.
- **The walk body**: the loadout assert still proves the P2K is owned, then cycles BACK to the
  K5 darts — the stealth legs run silent (the old P2K execute's gunfire was the original
  noise-pull; the dart sound ids 0x11C/0x11D are ignored on 07000009/0a [M the sound loop] and
  a sleeping guard runs no chain). A climb-first leg order was tried and kept: it does NOT move
  the g4 meeting (the stair base is on every route) — recorded honestly below.

## Verification, quoted

- `walk_tower_a_start_to_finish --exact`: **FAILED** (runs 1-9; the final run 9 finished in
  1.27 s of sim after the rebuild — g3's relay still wins the press race by ~100 ticks; panic
  at `walkreal_07000009_live.rs:133`, the unloosened [L] assert).
- Baseline (the branch point, this session's run 1): 3 passed / 5 failed — the five at the
  same assert.
- The rebuilt takedown's own wins, visible in the traces: g5 and g4 both SLEEP silently now
  (`t1426 g4 0x3e → 0x50` — a dart landed mid-challenge, no corpse, no gunshot); the racer
  flag catches g3 at 0x63 within one beat (round 1, 12.4 u) instead of at his 0x8a 31.8 u
  away; the nav chase crosses the whole stairwell (~500 ticks) instead of crawling.

## Left (the exact resume)

The press race needs ~100-300 ticks. The measured levers, in the order to try:

1. **Pre-position Bond INSIDE the stairwell, at the top, before g4's descent** (~t1000): his
   descent reaches the car park at t~1200; a Bond already at the stairwell top meets him
   coming down INSIDE the corridor at 2-4 u — the dart beats everything there, and a relay
   (if it still fires) starts the chase at the TOP with the door ahead already open (~480
   ticks to the west switches vs the 958-tick fuse). Shape: after the loadout cycle, a fast
   `walk_nav_clearing` leg to (18.8, −368.4, 39.0) BEFORE the stair-base leg — the current
   climb-first leg does this but still walks the base approach first; the climb must not pass
   the base meeting zone, so time it against g4's descent or accept the in-corridor meeting.
2. **Intercept g4's emergence at the car-park door** (21.5, −377.1, 40.9) from his blind side:
   he exits heading south-west; a Bond waiting ~2.6 u WEST of his line darts his back as he
   passes — no ack, no relay, the whole race vanishes. Needs a wait-and-dart beat at the
   stair-base nav 4 timed to the descent (deterministic: fixed 60 Hz, no draws).
3. The chase itself is near-optimal now; the door pre-open only helps if it happens before the
   meeting, which lever 1 subsumes.

## Live checks for the main agent

- **07000009** (PCSX2, gdbwatch): confirm the civilian relay — when one team-3 guard enters
  CivilianChallenge (0x3E), do the OTHERS get an immediate RunToAlarm with `+0x16` set within
  1-2 ticks, and through which message (the challenge bark's SHOUT/sound, or a direct
  EnemyAlerts)? The port shows a 1-tick relay (g3 t1224 after g4 t1223); the game's exact
  channel would tell whether a stealth walk can suppress it at the source.
- **07000009** (fieldprobe): the g4 descent timing — his patrol leaves the lobby stair top for
  the car park at a fixed tick on a cold load; a screenshot/trace of the car-park door
  (21.5, −377.1, 40.9) approach line would pin the blind-side intercept spot for lever 2.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session, so there are no numbers to copy.
