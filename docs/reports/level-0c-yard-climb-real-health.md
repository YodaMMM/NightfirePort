# level-0c-yard-climb-real-health — report

Branch `s/level-0c-yard-climb-real-health` (cut from origin/master edd5748b).

## What's done

**1. The mantle/pain question is answered: the port's climb has NO pain gate — and neither
does the game. The port matches.** The real-health run's Bond climbed NORMALLY while hurt:
19.8 → 3.8 hp with the climb ticks advancing every tick (y 2.03 → 7.95 at the climb's 1.2 u/s
[L the NF_0C_CLIMBTRACE run]); the ascent froze at exactly y 7.95 only when health hit 0.0 —
the death sequence [M `Player_CheckForDeath` 0x190810 → the dead branch of `Player_Update`
0x193600:316-319 — no substate arm, no collision pass] correctly ended it. The game's law
agrees term for term: `Player_Climb` 0x196310 advances the climb on the stick alone (the only
up-block is the AnimObj 0x20 head-hit flag [M 0x196310:69-71]); `Player_HandlePain` 0x1902c8
writes NO substate (health/armour/flash/dir-bits/grunt only); `Player_CollLadder` 0x196118:17's
sole health read is the GRAB gate (`0.0 < *(float *)(BL+0x894)` — a dead Bond cannot grab — the
port mirrors it). **No fix to the port's climb is warranted; nothing differs.**

**2. The kill chain is named** [L the real-health run's damage log + the yard census]:
- **drone #18** (skin 0x5000090, weapon 0x1e, (36.4, 12.1, −208.8), hp 10, a STATIONARY sentry)
  dealt **51.8 of the run's 100 damage in 4 hits** (−16 each) from d ~34 u — outside the plain
  20 u shooters window the whole run — and killed Bond ON the ladder at y 7.95 (two −16s on the
  climb);
- **drone #1** (skin 0x5000093, weapon 0x1e, (13.0, 12.1, −196.0)) — d 9.6 from the ladder base,
  too steep to shoot from below (its strict gun rays read false there), LEVEL at d 7.6 from the
  ladder top — killed Bond within ~110 ticks of the mantle on the runs that survived the climb;
- the small fry (#27/#33/#30/#31, 8-11 each over 7-18 hits) bled the rest through the answered
  fights.

**3. The walk-side answer was built and measured.** `walkreal0c.rs` gained
`yard_sniper_stands` (real damage only — the top-up walk is untouched by construction): the
direct number-key switch to the sniper 0x1e through the real input path [M
`Player_WeaponChange` mode 0] — the AIMED sniper's spread is zero [M `player_spread`'s
0x4080d0 record aims exact] — then the shared fight machinery against whoever is hitting Bond.
Measured outcomes: the stands SUPPRESS #18's beat (the climbs that followed a stand survived to
the mantle — "the yard ladder climbed for real to y 12.12"; without one, death on the ladder),
but the UPWARD rounds never LAND: the aimed 0x1e at a stationary box, d 33, perfect aim (err
0.000, `blocked false`, strict gun rays true), three clips dry, tgt hp 10.0 — while the
LEVEL-geometry fights (the deck legs, d 7-10) land every time. **That mismatch — the player's
upward shot dies in flight where the flat eye-ray/gun-ray predictor says clear — is a port gap
on its own** (the predictor traces the flat world; the player-bullet flight runs the cel-scoped
pick) and is the next session's first job. `lane_fights` was tried on this route and reverted
(the 48 u fights stall and trade worse — a death at the staircase, tick 6412). The sister test
stays ignored with the measured reason (an un-ignored red test would break the suite).

## Tests

- `walkreal_0700000c_live`: **2 passed, 1 ignored** (the top-up master walk + the ladder495
  climb test green; the real-health sister re-ignored with the measured reason).
- The climb/mantle group: `fix_ladder2_live` 4, `fix_ladder_live` 4 (+1 ignored),
  `fix_ladder_water_live` 4, `ladder_02_live` 3, `ladder_underwater_02_live` 6 — **all green**.
- `nf-formats` + `nf-audio`: 136 passed, **3 failed = exactly the 3 allowed slim-ISO
  `disc_golden`**.
- The branch touches ONLY `crates/nf-view/tests/common/walkreal0c.rs`,
  `crates/nf-view/tests/walkreal_0700000c_live.rs` and `docs/WORKLOG.md` — no src changes —
  so nf-game/nf-formats/nf-audio/nf-view-src carry master's own state; the full 4-crate
  nf-view build (~950 binaries) cannot fit this window (the standing sandbox limit).

## What's left

1. **The upward-shot port gap**: why the player's bullets die on an upward shot whose flat
   eye-ray is clear (the cel-scoped flight vs the flat predictor). Fix it and the yard stands
   start KILLING #18/#15 from the base — the climb then runs on a clear field.
2. The 0b real-health sister's follow-ups (the [L] corridor-cost check from
   `docs/reports/level-0b-corridor-world-damage.md`) — same open item.
3. Un-ignore the 0c real-health sister once the climb phase is survivable end-to-end (the
   suppression stands + the landed upward shots, or a route that clears the deck snipers at
   level geometry before the descent).

## Live checks for the main agent

1. **The real machine's yard climb at low health**: PCSX2, 0700000c, walk Bond to the yard
   ladder at ≤ 20 hp with the deck snipers alive. Does the game's Bond mantle the yard ladder
   under their fire (the port's law says the climb advances on the stick regardless of pain
   [M 0x196310])? Watch Bond's health float (`gdbwatch.py` on BL+0x894) during the climb.
2. **The snipers' real beat**: how often does #18-class (weapon 0x1e) actually fire at a
   runner in the yard (the port's log shows 4 hits in ~800 ticks from LOS-open), and does the
   game's AI dive when fired at from below (the suppression the port's stands rely on)?
3. **The upward player shot**: can the player hit a deck-level sentry from the yard floor in
   the real game (the aimed 0x1e at d ~33 upward)? If YES, the port's upward-flight death is
   confirmed as a port bug; if NO (the game's own lip geometry eats it), the walk needs the
   level-geometry route instead.

## Usage

not visible to me — this environment exposes no token, credit, context-window or cost figures
for the session (no usage panel or API counters are reachable from the sandbox shell).
