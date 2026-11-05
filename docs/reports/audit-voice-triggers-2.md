# audit-voice-triggers-2 — the four barks' effect on the walks

Status at the sandbox cutoff (2026-11-05 ~10:00 UTC): **1 of the 4 red walks made green this session
(smg's 1 u control, the original red line) plus the Mayhew walk green from the earlier session on this
branch; the smg binary overall and the two full walks are still red, with the mechanism and the next
step pinned for each.** Nothing was reverted; every change is additive and opt-in or file-local. The
branch carries everything; the next session continues from it.

## The mechanism (applies to all four)

The four added voice triggers read correct against the decompile and were not touched. Their barks
consume the shared bit-exact RNG (the same law the Mayhew commit on this branch measured: the
PlayScript re-bark draws were 150 by t5704 on the 01 escort walk), so every downstream roll — guard
AI states, cover picks, volley spreads, player cone draws — resolves differently than master. The
tests' old pins encoded master's stream position. Per the task, no draw was removed; the pins and
tactics were re-shaped to what the game itself would do.

## 1. walkreal_07000001_live::following_mayhew_down_completes_the_level — GREEN (prior session)

Already fixed on this branch (commit 2f99f073a, before this session's sandbox reset): the guard-14
shotgun volley's moving spread crossed the contact 1.96 u off the aim line (modelled reach 1.39),
killing him (d#1 10.0 → −39.5, by_player, 0x1c) and failing the escort (ch 0x60 → 99 → auto-restart).
The walk's shield now models the game's spread cap (0.2464 rad [M bullet::player_spread's 176-unit
cap, pinned by nf-game's player_spread_blooms_and_the_aim_gate]) behind the opt-in `shield_bloom`
flag. I re-ran the 01 binary's contact_real_damage sister this session but not the Mayhew test itself
(see "What did not run").

## 2. smg_close_range_live::smg_rounds_damage_a_close_guard_and_the_pistol_sister — improved, still red

Which draw moved what: the bark draws shifted the stream so the single pistol pull's cone draw (the
spread block is UNCONDITIONAL — every spawned round draws r/θ/φ [M Bullet_init 0x12c5a0:171-180,
pinned by nf-game's cone_draws_three_even_when_dead_on]) missed the box the walker aims at.

Three honest fixes landed (all in crates/nf-view/tests/smg_close_range_live.rs, file-local):

1. **The pistol is semi-automatic** [M gun.rs Player_WeaponFiring — "the semi P2K held the same way
   fires ONCE; the pull re-arms on release"]. `fire_at` held the trigger every tick, so a leg could
   only ever fire ONE round; master's pass hid this because that one round happened to land. Now the
   pistol legs press 2 ticks / release 4 through the same real input path.
2. **Part 4 is TORSO, not head** [M damage.rs hit_damage's match: 5 → DroneDamage_Head,
   0x14/0x15/0x17/0x20/0x23/0x27 → Arms, 0x31..=0x38 → Legs, everything else (4 included) → Torso;
   pinned by nf-game's hit_damage_per_part_and_difficulty — h(4, 2, 1) = 5.25 torso, head=false].
   `torso_aim` excluded parts 4 AND 5 as "head bones"; on the 0x5000039 rig part 4 IS the chest
   (half 0.2×0.25×0.15 [L the pistol-leg box dump this session]) — excluding it aimed the control at
   0.07-half arm boxes, which the shifted cone draws miss. The filter now drops only part 5. The
   1 u leg went green: **6 rounds spawned, 5 impacts, hp 24.0 → −2.2 (drop 26.250), 107 ticks**.
3. **The pistol legs take the SMG legs' candidate doctrine**: the clear-eye-to-torso gate before the
   stand, then a retry loop over the live roster — a covered candidate is retried once after a
   300-tick reposition pause (state-0x74 guards re-cover), the gun is resupplied inside the loop
   (the retries spent the pistol dry), and a leg whose target dies to ALLY fire is void (the control
   needs the PLAYER's registration [M the impact record's weapon tag]).

Remaining red, with the evidence: at 2 u d#11 takes 6+2 rounds for 0 impacts (covered), d#6 8 rounds
for 0 impacts (covered), and the earlier run showed d#5 dying 12.0 → 0.0 with 0 player impacts — the
escort's volley steals the leg's kill. Next step: re-route the 2 u / 6 u legs to stands off the
escort's firing line (the escort idles near the meet plate; a candidate whose distance to the contact
exceeds ~8 u cannot be stolen), or pin the escort's interference explicitly and move the legs to the
courtyard roster. The 1 u control — the line the driver flagged — is green.

## 3. walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage — red, diagnosed

Which draw moved what: the shifted stream moved the hall fight's timeline; Bond now dies at t3253
(frame 3282) at (6.26, 6.46, 9.11) with armour 0.0. The last frame is one drone-#6 volley (skin
0x5000036, weapon 0x1c at (6.9, 6.3, 11.6), d 2.5) landing **six pellets in one tick** (5 × −3.00 +
1 × −3.50). The full ledger: 39 hits for 100.0 — d#6 48.5 in 24 hits, d#7 30.0 in 10, d#11 15.0 in 3,
plus two distant grazes — i.e. attrition, not one bad roll. `walk_to_the_exit_at` already arms
`keep_off`/`back_off` 4.0 for the real-damage d1 walk (walker.rs's backing logic), but the guards
close through it and keep shooting from 4–8 u; the 6-pellet frame finished a body already bled to
~18 health. Next step: raise the d1 real-damage personal space to the 6.5 the Agent sister uses (the
TooClose forced-hit band is 3.0 [M DroneWeap_DoBulletAccuracy 0x179740:100-110; TooClose damage
doubling 0x139fac-0x139fc8]) AND thin the attrition before the hall — the walker's return-fire fights
(2 this run) should engage d#7/d#11 (30.0 + 15.0 of the bleed) on the approach instead of only after
Bond is damaged. Each run is ~2.3 s; the fix is testable quickly.

## 4. walkreal_07000003_live::walk_the_full_route_and_complete_every_objective — red, diagnosed

Which draw moved what: the shifted stream changed the east-lane fight's shape; Bond ends DRY
(clip 1 spare 0) fighting a guard the walker cannot hit, and the leg burns its 6000-tick cap:

- `the east lane guard 7: shooting drone 11 at +3600 (health 12, dist 8.0, clip 1 spare 0, exposed
  false)` — four "no damage for 900 ticks — repositioning" rounds, then `drone 11 still alive after
  10800 ticks`.
- `the east lane: no pickup within the walked-path corridor (512 trail points) — not taking the
  off-path detour`, `resupply done (false) clip 1 spare 0`.
- STUCK at bond=(10.28, 6.34, −36.05), target=(10.27, 6.65, −30.01) d=6.04, floor_below Some(5.247)
  (the moat ditch edge), `trig kind=1 out=0x31 mask=0x4 at(11.8, 6.0, −35.6)` nearby.

Next step: the walker's east-lane fight needs the drone-11 answer — either the opt-in off-corridor
resupply detour (its refusal rule is what strands a dry clip 1 spare 0), or break the cover stand
(drone 11 exposed false at 8.0 u) with the walker's existing flank/sidestep fight, or route the return
leg through the west lane the 0x21 plate leg already proved. A verification run of this binary is
~300 s, so budget one fix + one run per attempt.

## How this session was verified

Every claim above is from runs on this branch's tree (`2f99f073a` + this session's three commits):

- smg_close_range_live::smg_rounds_damage_a_close_guard_and_the_pistol_sister, single test via
  `--exact`: reproduced red (1 round, 0 impacts), then the three fixes above, quoted numbers per run
  in the transcript (the 1 u leg green line: `6 rounds, 5 impacts, hp 24.0 -> -2.2 (drop 26.250)`).
- walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage, single test
  via `--exact`: reproduced the death (t3253, the ledger above). No fix committed — see the next step.
- walkreal_07000003_live::walk_the_full_route_and_complete_every_objective, single test via `--exact`:
  reproduced the east-lane cap (the STUCK dump above). No fix committed — see the next step.

## What did not run (honest)

The sandbox reset at session start: fresh clone, work-slim + ISO re-download, rustup + ALSA user
prefix, and a full release build of the four crates on 2 cores (~35 min across three tool calls)
consumed roughly half the window. The **full suite did not run**, and the **whole walkreal binaries
did not run** (only the three single tests above via `--exact`). The smg edits are file-local to
smg_close_range_live.rs; the only shared-file change this session is the WORKLOG append. The next
session should run the full prescribed suite (`NF_ISO=$NF_ISO cargo test --offline --release -j2 -p
nf-game -p nf-formats -p nf-audio -p nf-view -- --test-threads=1`) and every `walkreal_*_live` binary
before finishing, and quote the total line here.

## Live checks for the main agent

None required this round — the remaining reds are Sim-level and fully reproducible in the sandbox.
One optional check that would settle the smg 2 u leg's re-route: in PCSX2 at the meet hall, shoot the
state-0x74 guards (d#11 at (7.94, 6.35, 17.46), d#6 at (6.6, 6.3, 14.6)) with the pistol from 2 u —
confirm the crates at the hall's east side actually eat the rounds (the port shows the ray entering
no box and the guards never losing health from covered stands).

## Usage

not visible to me
