# abseil-damage-repro — the widened search found the two unkillable-roper cases

## What the task asked

`abseil_damage_live` (real rounds at the 0x06000892 rope guard, aimed by writing Bond's
yaw/pitch) passes on master and on the older code that showed the symptom, so it does not
reproduce it. The task: widen the search with real input only — every abseiling placement on
07000011 at difficulties 1/2/3, every phase (Init 0x8e, Slide 0x8f, Hang 0x90), from the
stairwell landings a player actually shoots from, aim from the mouse-look path (`Input` look
deltas) rather than yaw writes, auto-aim on and off — log where each round ends and whether the
guard's health drops, find the case where rounds reach the guard without damage or never reach
him, fix it from the decompile, and pin it with a test that fails before the fix.

## The two cases the search found (both fixed, both pinned pre-fix-red)

The widened harness (`crates/nf-view/tests/abseil_damage_matrix.rs`, the 90-config
`probe_all`, kept `#[ignore]`) ran 3 difficulties × {p258 ch 0x13, p280 ch 0x14} × 6 stairwell
landings × auto-aim on/off with semi-auto trigger rounds and mouse-look aim. Two distinct
failure modes reproduce; neither is in the impact path — the abseil DSTATEs' case 8/9 arms
damage the guard exactly as decompiled ([M 0x169610:48-50 / 0x169730:68-70 / 0x169940:50-52]).

### 1. The guard never descends: the Slide's opponent gate read the wrong port fold

From most stairwell landings (west 14.8,31.4,−2.6; east-mid 22.4,23.2,−3.0; the NE landing
22.3,43.6,−3.2) the port's guard ran Init → **Hang at the rope top (y ≈ 46.5)** and stayed
there: the probe shows `seen[i1 s0 h1]` (the Slide never entered) and every round ending on the
stairwell flight face above the player (`world 16` hits, 0 guard hits — the sample world hits
sit 1.7–6.7 units above Bond's head on the flight above). The player cannot kill him because
**rounds never reach him — he never comes down**.

Cause: the port's `ABSEIL_SLIDE` UPDATE modelled the game's `+0x170 == 0 → Hang` arm [M
0x169730:46-51] with the old `ever_seen` stand-in (sight + reaction gate). But the game's SP
opponent pointer is **unconditional**: `NDrone2_FindOpponent` 0x1424e0:100-120 — run every tick
by ControlSTANDARD 0x1490f0:167 — calls `SetOpponent(glb_players)` for every non-team-2 guard
with **no sight test**, zeroing it only when `PlrStat_OkToUpdate` fails or the player is
dead. The port already folds that correctly as `bond_alive` ("stamped per tick in the vision
pass from Bond's health… the old `ever_seen` fold sent never-seen guards to NoOpponent",
standins-drone-922) — the abseil states were still on the old fold.

Fix (`crates/nf-game/src/drone.rs`, the ABSEIL_SLIDE/ABSEIL_HANG UPDATE arms): `ever_seen` →
`bond_alive`, with the 0x1424e0/0x1490f0 citation. The Hang's rope-tracking + re-slide gate
[M 0x169940:42-47] reads the same pointer and got the same fold.

### 2. The auto-aim lock never engaged through the stairwell's 0x40 well pane

With auto-aim **on** (the shipped SP default [M lib.rs:6115 + menu.rs:1332]) the lock
(`aim_target == Some(guard)`) never engaged from any landing — `lock 0` across the whole matrix
even where the guard hung 4 units in front of Bond and the bullets visibly hit him. The lock's
LOS was `line_of_sight` → the global `LOS_MASKS_ON=false` fold → a plain segment test, and the
east connector's **0x40-material well pane** (measured on the head→guard ray at t 0.16, material
byte 64) blocked it. A player with auto-aim on fires into the pane; the magnetized round the
game would produce never reaches the guard.

The game's Check_AutoAim LOS passes that pane: [M 0x1a73f8:164-165] calls
`Collide_LineOfSight` with **mask 0xa**, and `Collide_Intersect` [M 0x1ebaa8:305-309] sets
HITDATA+0x51 bit 1 on every hit whose surface byte carries 0x40/0x80 (and the node's 0x40 cel
flag sets it too, :143-145), which the mask's **0x8 bit drops outright** (:317-319). The port's
masked body had the node-flag drop under 0x8 but missed the surface-0x40/0x80 half of the rule.

Fix (`crates/nf-game/src/world.rs` + the `auto_aim` call): the masked body moved to a public
`line_of_sight_masked(a, b, mask)` with the corrected 0x8 rule (node flag **or** surface 0x40
**or** surface 0x80), applied unconditionally; `auto_aim` routes its LOS through it with the
established `LOS_SENSOR` mask. The shared `line_of_sight` keeps the global `LOS_MASKS_ON` gate —
no other caller's behaviour changes (the parked per-caller flip stays parked).

## How it was verified

- `the_roping_guard_descends_to_his_opponent_without_needing_sight` — Bond on the west landing,
  the guard woken by his channel: he runs the Slide without ever needing sight and hangs just
  above Bond (the 3.0 gate [M 0x169730:57-58]). **FAILED before the fix** (the guard sat at the
  rope top), passes after.
- `auto_aim_locks_the_hanging_guard_through_the_well_pane` — auto-aim on, Bond on the east
  connector: the lock engages through the pane, the locked rounds kill through the real
  pipeline, the death routes to AbseilDeath 0x92 [M 0x1461c8:44-52] and the rope releases [M
  0x169b28:28]. **FAILED before the fix** (`locked ticks 0`), passes after.
- Both pre-fix reds were verified by stashing the two src fixes and re-running the binary
  (panics at the matrix test's line 145 / line 200 — the descent assert and the lock assert).
- `mouse_look_rounds_kill_p258_from_the_east_connector` — the widening of `abseil_damage_live`:
  the same kill with the aim riding `Input.turn`/`Input.look` deltas (never a yaw write),
  auto-aim off, per-volley phase/impact logging. Held before and after (coverage, not a fix pin).
- `the_difficulty_2_abseiler_takes_the_rounds_the_same_way` — p280's first-ever damage test: he
  does not exist at difficulty 1 (the create gate [M 0x136be0:49]) and dies to the same real
  rounds on channel 0x14 at difficulty 2.
- The full suite ran on the final tree (see the total line below).

## What was covered (the matrix summary, all with real trigger rounds + mouse-look aim)

- **Placements**: p258 (18.895, 46.516, 1.434, ch 0x13) at diffs 1/2/3; p280 (19.745, 46.580,
  0.363, ch 0x14) at diffs 2/3 (absent at 1, pinned). No 07000011 placement uses mode 0x1b [D
  `nf-extract drones`; abseil_death_live's census].
- **Landings**: the NE landing (22.30, 43.60, −3.20), the east connector (22.34, 29.37, 3.52),
  east-mid (22.40, 23.20, −3.00), the old repro spot (22.40, 24.01, −1.39), the west landing
  (14.80, 31.40, −2.60), under-rope (18.90, 26.00, 1.40) — the walkreal landing waypoints.
- **Phases**: Init 0x8e / Slide 0x8f / Hang 0x90 observed and logged per volley; damage
  confirmed on the rope (Slide/Hang) by the per-volley impact log; the death + rope release +
  corpse route asserted in the pins.
- **Aim paths**: mouse-look deltas everywhere; auto-aim on and off in every cell.
- **Round ends**: per volley — guard hits, hits on other drones, world hits with the sample
  point (the "no-damage" cells before the fixes all ended on the flight face above the player or
  on the 0x40 pane; after the fixes the guard-facing cells end on the guard).

## What is left

- The parked hang-gate overshoot: from the east connector the guard's hang lands ~2 units below
  the 3.0 gate (`stairwell_guards_07000011_live`'s ignored measurement saw ~10 units from a
  different spot). Movement-model work, separate from the damage symptom; the pins here tolerate
  it (the guard hangs *above* Bond, within 4.0).
- The probe's "rounds reach the guard without damage" mode never reproduced — every
  config that put a round on the guard dropped his health, matching the decompile's
  HandleImpact(0,0) arms. No test asserted otherwise.

## Live checks for the main agent

1. Fresh boot 07000011 (any difficulty), stand on the east connector at the 0x13 plate
   (22.34, 29.37, 3.52), let the plate fire, and shoot the roping guard with **auto-aim on**:
   expect the lock pointer to take him through the well railing (the game's mask-0xa LOS), and
   the kill to drop him from the rope. In the port pre-fix the lock pointer never appeared.
2. gdbwatch the rope guard's `Drone+0x170` from the wake on with Bond unseen (e.g. Bond parked
   on the west landing at (14.80, 31.40, −2.60) below the guard's sight): expect it to hold
   Bond's object pointer (FindOpponent re-points it every tick), the Slide to run, and the guard
   to hang 3.0 above Bond — NOT to sit at the rope top y ≈ 46.5 as the old port did.
3. On the same setup, `Collide_LineOfSight` from Bond's head to the guard's bone 2 with mask 0xa
   through the landing's 0x40 pane: expect `!= 0` (clear) — the pane's surface byte 0x40 sets
   HITDATA+0x51 bit 1 and the mask's 0x8 bit drops it [M 0x1ebaa8:305-309/317-319].

## Usage

not visible to me — this sandbox exposes no token, cost or context-window figures to the worker
(no `--usage` output, no billing panel; the shell's `env` shows only the IM gateway metadata).
