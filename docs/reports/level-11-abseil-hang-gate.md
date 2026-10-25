# level-11-abseil-hang-gate — the rope guard's hang gate verified against the decompile; the test un-ignored green on the current tree

Branch `s/level-11-abseil-hang-gate` (from `origin/master` e6e3c899). **No port-code change was needed:**
the symptom the task carried (p258 sliding to y 22.34, ten units past the 32.37 hang gate) does not
reproduce on the merged tree — the gate fires exactly where the decompile puts it, and the test
`stairwell_guards_07000011_live::the_mode_1c_abseiler_hangs_within_the_game_s_3_0_of_the_opponent` passes
un-ignored. This report is the verification trail.

## The game's rule, from the decompile

`NDrone2_DSTATE_AbseilSlide` 0x169730, UPDATE (case 3), mode-0x1c arm:

- the opponent pointer `+0x170` — in SP `NDrone2_FindOpponent` 0x1424e0:100-121 re-points it at Bond's
  object EVERY tick and zeroes it only on the player-death gates [M];
- the fold `+0x674 = min(+0x674, +0x184)` [M 0x169730:52-54], where `+0x184` is the opponent's object Y —
  `Drone_GetOpponentInfo` 0x138f68:24-26 copies `opp+0x30..0x3c` into `drone+0x180..0x18c` [M];
- the hang compare `obj+0x34 < opp->+0x34 + FRAME_RATE_MUL·3.0` → `Drone_SM_SetState(0x90)` (AbseilHang)
  [M 0x169730:55-58]; the no-opponent arm `+0x170 == 0` → 0x90 [M :46-51].

The port's slide arm (`crates/nf-game/src/drone.rs`, `(state::ABSEIL_SLIDE, msg::UPDATE)`) transcribes this
line-for-line: `rope` is the port's `+0x674` (folded `if self.player_pos[1] < self.rope[1]`), `player_pos`
is the port's `+0x184` (Bond's position refreshed every tick), and the gate is
`if self.pos[1] < self.player_pos[1] + 3.0 { ABSEIL_HANG }`. The `+0x170 == 0` arm is the port's
`bond_alive` fold (the pointer dies only when the player dies [M 0x1424e0:100-121]).

## The per-tick slide log (the task's step 1)

A scratch probe (d3, Bond parked at (22.34, 29.37, 3.52), the 0x13 plate fired, p258's state/pos/rope/
velocity/descend_speed/player_pos printed each tick) on the merged tree:

- ABSEIL_INIT 0x8e for 30 ticks at the rope top (18.89, 46.52, 1.43), descend_speed 0.075, then
  ABSEIL_SLIDE 0x8f (descend_speed 6.9 = the +0x50 store 0x40dccccd [M 0x169730:22]);
- the rope folds to Bond's y **29.34** on the first slide tick (the +0x674 fold);
- the descent accelerates and **equilibrates at ~5.5 u/s** — the shared ×0.98 flying-velocity damping
  [M 0x140968:139-141, the `+0x4F8 & 0x40` bit test, landed by standins-drone-921 8582f61a] caps the
  accumulation at descend_speed/FRAME_RATE_DIV/0.02 ≈ 5.75 u/s;
- **the gate fires at 32.34** (Bond 29.34 + 3.0): the crossing tick is +225 (y 32.339, vel −5.53) and the
  state is ABSEIL_HANG 0x90 at +226 (y **32.247**);
- the hang's own ×0.98 velocity decay [M 0x169940:33] coasts the guard down ~2.7 units, settling at
  ~29.6 — just above Bond's feet, inside the game's own Hang window (the Hang UPDATE re-slides only when
  the opponent is 3.0+hang_rand BELOW [M 0x169940:42-47]).

## Why the 2026-10-18 measurement saw y 22.34

The measurement predates two abseil-path changes now on master:

1. **The shared flying-velocity damping** (8582f61a, standins-drone-921, 2026-10-22): before it, the
   ×0.98 damping was gated `mode == 0x1D` (astronaut-only) and an abseiler's slide velocity accumulated
   WITHOUT BOUND. The hang then entered at an arbitrarily high downward velocity, and the hang's own
   ×0.98 decay [M 0x169940:33] coasted the guard ≈ v₀·50/60 units below the gate before stopping — at
   the old tree's entry velocity that is the ten-unit drift into the y-24 landings (the mid-well steady
   state the playtest saw). [S for the exact old-tree tick values — not re-measured; the mechanism is
   the only material change in the slide/hang path between the measurement and the current tree.]
2. **The opponent-pointer semantics** (95a7bb72, combat-sight-loss-exit, 2026-10-24): `+0x170` follows
   [M 0x1424e0:100-121] per tick; the sight-based stand-ins (`ever_seen`/`unseen_ticks`) that could hang
   the roper early are gone.

With the damping in place the compare catches the guard at the gate on a slow, equilibrated descent —
no port change is required, and none was made.

## Verification (what ran)

- `stairwell_guards_07000011_live` (the hang test **un-ignored**): **3 passed / 0 failed** —
  `the_mode_1c_abseiler_hangs_within_the_game_s_3_0_of_the_opponent`, the disc-pairs load pin, the
  0x13-plate wake/alert pin.
- `stairwell_ghosts_07000011_live` 3/0/1 ignored, `stairwell_walls_07000011_live` 2/0/1 ignored.
- `abseil_death_live` (nf-game): **5 passed / 0 failed** (the rope release on bullet/explosive kills, the
  death re-route, the disc-only rope placements).
- `abseil_damage_matrix` (nf-view): `auto_aim_locks_the_hanging_guard_through_the_well_pane` and
  `mouse_look_rounds_kill_p258_from_the_east_connector` pass; `the_difficulty_2_abseiler_takes_the_rounds_
  the_same_way` is a minutes-long soak (see the totals line below for its final state).
- The fast runner (`tools/test/suite.py`) on this tree: see the totals line below. Its `exit -9` CRASH
  lines are the sandbox's memory-pressure kills (the same binaries pass standalone — e.g.
  `stairwell_walls_07000011_live` CRASHed in the runner and passed in the foreground run above); the
  `disc_golden` trio and `identity_disc` (which asserts the full 1,191,072-sector retail identity against
  this sandbox's 1.06 GB onfoot image) are the environmental failures the slim-ISO sandbox carries.

## What's left

Nothing from the task's list. Note for a later round: the game's rope-fold source `+0x184` is refreshed
by `Drone_GetOpponentInfo` only while the +0x4F8 & 0x100 eligibility bit stands [M 0x138f68:9-12]; the
port refreshes `player_pos` unconditionally — indistinguishable in SP (the bit is the vision eligibility,
and the fold only lowers the rope), but an MP-side divergence to keep in mind.

## Live checks for the main agent

1. Fresh boot of 07000011 (d3), Bond parked at the east connector (22.34, 29.37, 3.52): confirm p258's
   hang height on the real game — a gdbwatch on his obj+0x34 during the slide should show the rope guard
   stopping within ~3.0 above 29.37 (the port hangs at 32.25 and settles ~29.6).
2. The same window, p258's +0x674 (the rope point): expect it to fold to the opponent's +0x34
   (29.37) at the slide start and stay [M 0x169730:52-54].

## Usage

not visible to me (no token, credit or context-window readouts are exposed in this environment).
Session wall-clock: branch created 04:26 UTC from origin/master e6e3c899; the sandbox window is
~04:22–06:22 UTC.
