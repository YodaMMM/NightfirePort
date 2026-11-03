# level-0c-yard-climb-real-health-3 — both -2 walls solved, the wire crossing passes; the west-ledge fight remains

Branch `s/level-0c-yard-climb-real-health-3`, cut from origin/s/level-0c-yard-climb-real-health-2
(9cf3d8ca), merged origin/master (35dca502) clean at the start of the session.

## What is done

**Wall 1 (the "unkillable" deck sentry) is retired with measurements, not a fix.** The -2 report's
twin/corpse-shield theory does not hold on this tree: a probe dump shows **#36 present false,
enabled false** at the emplacement (state 0x0, never spawned) in every variant (top-up, real
damage, with/without the door sweep). The -2 census's PAIR reading (#18 + #36 at one spot) was
[L]-sourced from an older run and does not describe this tree. The real mechanism, measured this
session: the deck sentry #18 **wiggle-steps ±0.1 u** (state 0x2b), and the zero-spread aimed act
[M Bullet_init 0x12c5a0:139-170] has **no lead** — the round flies ([M] Bullet_update 0x12e7b0
steps `speed` per tick) and a converged round (err 0.000, perr 0.000, blocked false) dies in
flight while the sentry steps; the same stand from the same spot LANDED the run the sentry
paused (probe A round 2: hp 10.0 → -50.0; the master upward-shot proof's round-1 kill is the
same luck). A player holds the shot until the guard pauses.

**The fix: the STILL-GATE aimed stand** (0c-local in `crates/nf-view/tests/common/walkreal0c.rs`,
flag-gated by `with_yard_aimed_stands` exactly as before — flag off ⇒ byte-identical master
behaviour): converge on the exposed box, and tap **only after the target has held still 18
ticks**, with the blocker-retarget idiom and the strict gun-ray aim. The gun rays matter: the
plain eye ray read Some on #15 while every round died on the deck structure (5 rounds, hp 10.0
never moved — the predictor-vs-muzzle-flight gap), while the strict `exposed_part_gun_rays`
demands the flight's own lane clear.

**Wall 2 (the health budget) is solved by kill order, all player-like:**
- the door spot is **not exposed at all** (aim None from d ~37.3, elevation 15.6° — the -2
  at-the-door sweep never fired a round); the emplacement dies **MID-CROSSING** from the yard
  legs (d ~24.6-32, 19-23°) — measured: `[] still alive` after the mid-crossing stand;
- the east-ledge snipers #20/#37 (2 × -16 from d ~59 during the door approach — outside every
  fight window) get an at-the-door sweep (64 u);
- the pre-climb sweep suppresses #15/#1 (the #15 futile-clip case is what the gun rays fix);
- the ladder-top sweep does **not** wait for a recent hit (#1's first beat lands ~110 ticks
  after the mantle — before any hit window could pick him) — measured: #1 killed at the top;
- the west-edge sweep clears the far side before the wire legs.

**Measured result (the real-damage walk, real input from the level start):** the walk survives
the whole yard — the climb mantles, the ledger at tick 12300 is 82.4 for 100 with Bond **ALIVE
at 17.6 hp** (the -2 runs died at the ladder base at 1.6), and the **#284 wire ride + drop land
on the west ledge** ("the west gap crossed on wire #284 — landed (-8.21, 12.82, -215.66)"). The
walk's furthest point is the west-ledge fight at tick 14308.

## What is left (why the sister stays ignored)

The west sniper **#13 patrols to (-3.0, 12.2, -221.1), d ~6 LEVEL across the gap**, and its
2 × -16 + the fatal -1.6 graze kill the 51.4-hp Bond at tick 14308 [L the death log]. The walk's
own fights land killed false against him, and the still-gate sweeps find **no gun-ray-exposed
candidate** from the west edge or the ledge landing — the plain eye ray reads Some while the
rounds die on the ledge lip (the same predictor gap), and the strict rays read None behind #13's
cover. Next session: kill #13 from a spot whose gun rays clear (or walk the lip corner toward
him), then the ledge diagonals + wire #282 + the 0d legs under the fights; then un-ignore.

**A razor edge the main agent should know:** with the west-edge sweep at the LADDER TOP, the
ride **froze byte-exact** at d_B 1.52-1.61 (glide 8.12 → 1.61 then wire_anim_time stuck 22.6,
anim 0x60000b5, auto_drop frozen 20.8, substate 6, sim.disabled false — wire_tick stops
executing mid-ride). Moving the sweep to the west edge shifted the ride's phase and the crossing
passes — the freeze is **path-phase dependent**, not deterministic, and not a master regression
(the earlier WORKLOG entry claiming 6d1ef8e9 broke the top-up walk was WRONG — the "control"
run's name filter had matched both tests; the failure was the sister's; corrected in the
WORKLOG). See the live checks below.

## How it was verified

- `walkreal_0700000c_live`: **2 passed, 1 ignored** (the top-up walk + ladder495 green; the
  sister re-ignored with the measured state above — it must not ship red).
- `upward_shot_0700000c_live`: **2 passed** (the hip dry-clip pin + the aimed kill — the
  flag-off paths keep the pinned behaviour on the final tree).
- `nf-game`: **655 passed, 0 failed, 5 ignored**.
- The session's probe runs (scratch binary, never committed): the -2-variant vs proof-variant
  isolation, the #36 present-false dump, the still-gate kill, the ride-state traces. The fast
  runner (`tools/test/suite.py`) did not run this session: the window went into the two walls
  and the ride; the touched binaries were run explicitly as listed above. Run the full sweep
  (`walkreal_*_live`) before merging — the shared-file changes are confined to the 0c-local
  module + the 0c binary, but rule 7's sweep is mandatory for any shared-module touch.

## Live checks for the main agent

1. In PCSX2, ride wire #284 (the west gap) on a fresh boot: does the game's ride ever freeze
   mid-wire (anim stuck, the auto-drop timer paused)? The port shows a path-phase-dependent
   freeze at d_B ~1.5-1.6 with the wire body not executing — is there an NIS/script trigger at
   the west ledge that can catch a rider mid-wire?
2. From the west ledge landing (-8.2, 12.8, -215.7), can a player shoot the west sniper #13 at
   (-3.0, 12.2, -221.1) — or does the ledge lip eat the rounds the way the port's predictor
   mismatch shows? A screenshot of the sight line would settle the cover.

## Usage

not visible to me (no token, credit or context-window figures are exposed to this environment;
the only counters visible are process wall-clock times in the tool outputs above).
