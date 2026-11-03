# real-health-agent-difficulty-08-09-0a-0b — report

Branch `s/real-health-agent-difficulty-08-09-0a-0b`, cut from origin/master f9c1d69c.

## What's done

**The Agent pin, inherited from the merged work** (all already on master — no 01-07 merge was
needed, see the note below): the menu's Agent pick is GameState+0x28 = 2 through the front
end's launch path [M C_SBNFDFCTY 0x216ca8:15; the `difficulty[]` table 0x2df4c0 row 1 carries 2;
Txt 0x14e = "Agent"], the port's `load_with_carry_difficulty(..., 2)`. The pick scales Bond's
pain by `Plr_DMod_*` and the guards' hit chance by `DroneFiring_Accuracy_*` per the level's
tuning group [M Player_HandlePain 0x1902c8; DroneWeap_DoBulletAccuracy 0x179740:74-88;
tuning.rs ← ReadTuningVars 0x1d3a80]: 08 sits in the 05-08 group (Agent pain 0.7 vs Operative
0.6, accuracy 1.1 vs 0.8), 09/0a/0b sit in the 09-0b group (Agent pain 0.8 vs 0.6, accuracy
1.0 vs 0.8). The min-difficulty placements pass Drone_Create's gate [M 0x136be0:49].

**Why no `origin/s/real-health-agent-difficulty-01-07` merge.** The task said to merge it "if it
is not on master yet". Its load machinery (`load_with_carry_difficulty`, the tuning rows,
Drone_Create's gate) IS on master. Its 01-04 sister TESTS are not — deliberately: master's
2879088f reverted them ("01/06 red on the batch") and the separate
`real-health-00agent-02-03-04` branch (1f6d00bc, pending) re-lands them byte-exact with the d3
sisters. Merging the 01-07 branch would have resurrected the reverted tests and clashed with
that pending branch, so the merge was skipped and the sister pattern was copied from the 05/07
sisters already on master (4daa1b67).

**The four sisters.** Each of walkreal_070000{08,09,0a,0b}_live's real-health walk got an
Agent-difficulty sister (`..._real_damage_agent`), the same route and the same end, loaded the
menu way and asserting `Drones::difficulty == 2` at load [M GameState+0x28 0x2a3790 →
Drones::difficulty]:

- 08 `walk_the_whole_mission_on_real_input_real_damage_agent` — the walk body parametrized
  (`..._at(difficulty)`); the d1 call keeps the plain `load` byte-for-byte, d2 rides
  `load_with_carry_difficulty(..., None, 2)`. The master's carried-latch pokes (0x45/0x70/0x02/
  0x48 — the 07 hand-off stand-in the master already pins) run on both paths.
- 09 `walk_tower_a_start_to_finish_real_damage_agent` — same parametrization on the plain load.
- 0a `walk_office_floor_qworm_keyboard_hatch_real_damage_agent` — the warm-carry loader
  parametrized (`load_carry_at`): the COLD capture load (which only harvests the boot's own
  health/switch/inventory/channels — none difficulty-dependent) stays as-is; the WARM load
  branches to `load_with_carry_difficulty(..., Some(&save), 2)`. The snapshot keypad-code
  override applies to both paths.
- 0b `walk_fresh_spawn_to_the_mission_end_real_damage_agent` — `load_fresh_at(2)`, the same
  NO-channel-poke fresh boot (`fresh_boot_asserts` pins the carried channels off) and the same
  fight package (`lane_fights` + `tranq_fights` + back_off 3.0).

No shared helper changed (crates/nf-view/tests/common/ has zero diff on this branch — git
shows none), so every other level's walks are untouched by construction.

## Sister results (Agent, real health, real input)

- **08 GREEN**: the whole mission (intro → fusebox → safe → briefcase → gondola → Hind) in
  29870 ticks, the Hind shot down (5 rockets, 505 vs hp 448.5), ends **24.8** health.
  One tactics fix: the first d2 run died at tick 25137 — the walkway shotgun guard #20
  (skin 0x500002f, weapon 0x15) ground 10 × −4.2 = 42.0 at d 8.1 through the plain
  nearest-body fight pick, with #16/#19's bursts behind him (19.6 + 16.8). The sister now runs
  `with_real_damage_head_hunting` — the same tactic the 00 Agent sisters shipped for this level
  group ("every tick a fight stands is a burst tick"): the fights take the clear head box first
  [M NDrone2_HitDamage 0x145558 × DroneDamage_Head], and the grind disappears.
- **09 GREEN**: start-to-finish in 140678 ticks, **100.0 health + 50.0 armour, 0 hits, 0
  return-fire fights** — the stealth patrol-beat clears held at Agent outright (the guards'
  accuracy 1.0 never mattered because nobody was left alive to shoot).
- **0a GREEN**: the office-floor ring — Q-Worm, the security keyboard, the keypad door, the
  hatch — in 16006 ticks, **100.0 health, 0 hits** (the watched legs + the crouch/jump beam
  passes held; the ring's `watch_and_clear` caught every runner pre-sight).
- **0b GREEN**: the fresh-spawn mission end in 16466 ticks, **72.4 health** (the Operative
  master ends at 15.4 — the Agent run ends HEALTHIER on the same route, see the tactic below).
  Two port-divergence candidates were investigated and both resolved GAME-TRUE:
  1. The first d2 run died at tick 16103 with 73.6 in 23 hits from "the world" + 26.4 from the
     deck guards. The world pain is the terminal-room GT wall guns #174/#175 (class 0x27,
     p4 = 14 [D census]): 23 × −3.2 = 73.6 is exactly the d1 crossing's 23 × −2.40 scaled by
     Plr_DMod_Normal 0.8 / Plr_DMod_Easy 0.6 — the game's own Player_HandlePain pipeline
     [M 0x1902c8], not a port over-damage. The hit COUNT (23) matches d1's run because the
     turret fire law is cadence-deterministic [M 0x180498 case 4: the 0.1 s fire gate + the
     eased barrel Quat_Slerp_Acc — gt.rs's `combat_fire_rides_the_eased_barrel` pins this].
  2. What arms the pair: OR #290 = OR(ch 0x0D, ch 0x06) → ch 0x0E = 14 = p4 [D #290; the level
     census] — the dome laser fence's touch (0x0D) OR the files take itself (ch 0x06, the
     spawned use-plate's out). The fence crossing is mandatory (the files plate sits past it),
     so the turrets arm no matter what — game-true trap. The exec trio #291-293 (p4 = 28) has
     NO ch-28 writer in the census [D], so it never wakes.
     **The tactic (the player's answer): the lane sprint.** The turrets fire on the fixed
     0.1 s cadence with the barrel easing 6.67%/tick [M 0x180498 case 4], so every tick in the
     lane is a fire tick the ease can land — the walk_to legs' node pauses were the cost. The
     0b test gained a local `lane_sprint` helper (the deck-sprint pattern generalized: the raw
     stick loop, forward 1.0, turn clamp — real input only) applied to the POST-ARM transits
     under the walk's own `real` flag: the dome-floor approach to the spawned files plate, the
     return plate→room→n14→n15→n16 lane. The GT drain fell 76.4 → 19.2 and the run completes
     at 72.4 with the fights sleeping/killing the deck guards (#4/#5 taken at the take slot).

No port code changed: `crates/nf-game` has zero diff on this branch. The whole task is the
difficulty wiring (already merged) exercised headlessly, plus the two per-level tactics above.

## How it was verified

- The difficulty assert at every load (`Drones::difficulty == difficulty`) — the sisters cannot
  silently run the wrong row.
- The four binaries re-run COMPLETE and green on the final tree, serially under the suite's own
  conditions (NF_SILENT=1, --test-threads=1): walkreal_07000008_live 5 passed / 0 failed
  (111.2 s), walkreal_07000009_live 7/0 (142.6 s), walkreal_0700000a_live 11/0 + 3 ignored
  (44.4 s), walkreal_0700000b_live 6/0 (35.4 s). The d1 masters (plain + real_damage) are in
  those counts — the refactors kept them byte-identical and green.
- `cargo test --offline --release -j2 -p nf-game`: 102 test binaries, all ok, 0 failed.
- nf-formats + nf-audio: green except the **3 `disc_golden` tests** (`bigf_header_fields_…`,
  `bigf_tables_match_the_golden_manifest` — "missing: /DRIVING/MIS01.MUS#0",
  `directory_tree_matches_the_golden_manifest`): the slim/onfoot ISO lacks the full disc's
  files — exactly the driver's "only the 3 disc_golden tests may fail" allowance on this ISO.
  Zero diff on both crates from master.
- The full `tools/test/suite.py` run did not fit the window: this sandbox RESET mid-task (the
  second session), so the target dir started cold, and the suite's build step alone (all four
  crates' ~100 test binaries, serially linked on 2 cores) exceeded the 10-minute tool timeout
  before the run loop began; the run phase would not have fit the remaining window either. The
  blast radius the task names — the four touched walkreal binaries + nf-game (+ nf-formats/
  nf-audio) — was run complete as above instead. No FAILED/TIMEOUT outside the disc_golden
  allowance.

## What's left

- The full 4-crate suite.py total on the merged tree (the next window can start it on a warm
  target dir; this branch's blast radius is green as itemized above).
- Optional Agent-margin polish (none required to pass): 08's 24.8 is the thinnest end; if a
  later task wants more, the fix-up candidates are the dead guards' long-range_arcing rounds
  (#15/#16 hit from d 72-74 after death — in-flight rounds, game-true) and the single 26.2
  world hit late in the descent.
- Live checks for the main agent: none filed — the difficulty pipeline is pinned by [M]
  addresses and exercised headlessly; no visible-render surface changed. If the main agent
  wants one: on a fresh 0700000b boot at Agent, a gdbwatch on ch 0x0E (0x2df4c0-adjacent
  switch_channels slot 14) during the dome fence crossing should show the terminal-room GT
  pair's enable rising the tick the beam fires (the OR #290 law the port runs).

## Usage

Input tokens / output tokens / context used / cost: not visible to me (no token or context
figures are exposed in this environment). What I can see: the sandbox wall clock (the session
started ~15:41 UTC after a sandbox reset; the setup — clone, work-slim, the onfoot ISO, rustup,
the user-prefix ALSA headers, cargo fetch — was redone from scratch and took ~30 min of it).
