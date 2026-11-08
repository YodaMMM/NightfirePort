## Your task: `glass-pierce-3`, the 07000008 walks win their Hind fights with rounds passing glass as in the game

Branch from origin/s/glass-pierce-2, merge current origin/master, and read its docs/reports/glass-pierce-2.md. Take items 1 and 2 of "What's left". On the shifted timeline the topped-up 08 walk's Hind took all nine rockets the walk carries (about 34 each against about 101 on master), because the detonations fall at the blast's edge. Change the fight's launch discipline: wait for the machine's closer approach so the contact blast is full strength, or aim at the body sphere's centre instead of the chase point, then re-pin the ledger. The 08 d3 sister cleared the mission and died at 28105 to a 62.5 blast at about 3 u; fix it the same way. The flight pin is not the problem. Never lower damage or health, and keep the rocket count the walk really carries. Run walkreal_07000008_live, the bullet binaries, glass_pass_every_level_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/glass-pierce-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-08T04:19Z

---
## Report (done, 2026-11-08T05:09Z)


---
## Report from the worker (done, 2026-11-08T05:09Z)

# glass-pierce-3 — the 08 Hind fight's launch discipline; the topped-up walk's rocket economy fixed

2026-11-08, branch `s/glass-pierce-3` (from origin/s/glass-pierce-2 fa1f11131, merged with
origin/master d09f8b2b3 — one conflict in `dead_pane_live.rs`, resolved to master's side: the
`#[ignore]` ledger attribute master added). Nothing lowered damage or health anywhere; the rocket
count the walk really carries is untouched (the ledger asserts still pin clip 4 + the type-0xE
carry cap = 8 on hand, and the ride pickups).

## Landed — the fight's launch discipline (glass-pierce-2's "what's left" item 1)

All three changes are in `crates/nf-view/tests/walkreal_07000008_live.rs` (walk-side only, the
fight loop):

1. **The chase point is the body sphere's centre, not the matrix origin.** The tracker (the
   pre-launch aim and the in-flight guided ride both steer at `cp`) now reads slot 2 of
   `copter_part_targets` — the only part a pick or a rocket can reach [M Copter_Register
   0x17e530:7; Collide_PickObj 0x1eae80:58-82's always-on 0x20 gate] — with the matrix origin as
   the fallback when the slot is absent. Measured on this fight: the 08 body sphere's centre is
   IDENTICAL to the matrix origin and R = 13.56 every tick [L both fight ledgers], so on this
   level the lever is geometrically inert — the distance gates below did the work — but the
   chase target is now the part the blast sweep and the bullet pass actually test [M
   copter_part_targets; Explode_Propagate 0x17c518 case 0].
2. **Launches wait for the machine's close pass: `l <= 45.0` (both difficulties).** The AT-420
   round cruises 0.33666 u/tick [L the Sentinel measurement,
   docs/live/2026-10-14-weapon-measurements-pcsx2.md] off a 38-tick quarter-speed ramp [M
   Bullet_DoTrails 0x12d890:134-138]; a shot at a far pass burns its whole flight on the approach
   and the chase dies short of the machine — the blast at the falloff's edge. That is the
   measured topped-up ledger: 9 rockets, 303 total (~34 each; master's fight: ~101/rocket) [L
   glass-pierce-2]. The 45 u cap is the walk's own tactic [S unverified as game behaviour — like
   the 4.0 back-off]: a 45 u shot flies ~150 ticks, well inside the round's reach.
3. **On 00 Agent the contact surface stays ≥ 8 u from Bond: `l >= 22.0`.** The body sphere is R
   13.56 [L the part-target ledger], so a contact blast lands ON the sphere's near surface,
   `l − 13.56` u from Bond, and the falloff reaches 8 u [M 0x17c518:69-71 — 100·(1−d/8)]. The old
   d3 gate (`l >= 16`) allowed launches that put the surface 2.4-7.5 u out — the measured −62.5
   blast at ~3 u (100·(1−3/8) exactly) and the −2/−2/−1.5 near misses [L glass-pierce-2's d3
   run]. The sisters' lower bounds keep their 10/16.

**Result: the topped-up 08 walk (`walk_the_whole_mission_on_real_input`) is GREEN** — the shoot-down
took **4 rockets (was 9), damage 353 vs hp 299, best single rocket 101** (rockets 2-4 gave the full
contact blast 100 + 1.0; launches at l 19.8/44.8/36.2/29.2 — the close-pass discipline) [L this
session's run log]. The ledger asserts are unchanged and pass: ≤ 8 rockets on hand, a body hit
carries the ≈100 blast, the accumulation crossed hp.

## What's left — the d3 sister (`walk_the_whole_mission_on_real_input_real_damage_00agent`) is still red

The self-blast family is FIXED (the 22 gate: this session's d3 run took NO self-rocket damage —
its hit ledger is the Hind's own −2 missile edges plus the final −2), but the fight still stalls:

- **4 of 8 rockets stillbirth**: they die 2-7 ticks after launch with no blast, no copter damage,
  no Bond damage. The per-tick ride trace [L this session] shows the t11507 launch leaving at dir
  yaw −2.59 while the machine's bearing is −2.13 — **0.46 rad wide** (the crosshair cursor was
  still pinned at its clamp; the measured pinned dumps read cross (−0.296, ±0.394) [L the diag run
  2026-11-04]) — and the early guided steer flies the round into the west doorway's south frame
  band 0.6 u out (the muzzle ray exits the doorway gap's z range: the gap is −178.25..−180.6, the
  ray's z at the doorway plane is −180.77).
- With 4 rockets wasted the fight stalls at **202/598** (two full 101 contacts landed), the walk
  stands (its slot never moves) through the machine's circuits without another launch window, and
  the Hind's missiles (weapon 0x62, one per 60·DiffMod ticks [M Copter_FireMissile 0x17d9f0;
  fire_missile], the 08 homing roll) bleed Bond out at the falloff edge — death at tick 41676, 37
  hits for 100.0 [L the run log].
- **Tried and REVERTED**: an un-pinned-crosshair launch gate (`|cross| < 0.25/0.35`). It starved
  EVERY window — the walk died at 32973 with zero rockets launched [L the run log]. The gate and
  the reason are in the test's comments.
- **The next lever** (pinned for the next round): the muzzle-spawn corridor. The rocket spawns at
  the head/datum-0 midpoint [M Player_WeaponInitBullet 0x1a4bf0:212-240 — the same origin block
  the grenade path ports], which the corridor gate's eye±0.25 origins do not cover; the gate
  should test the rocket's own early-steer fan (the launch dir toward the machine's bearing over
  the first ~12 u) from the muzzle-approximate origin, so a pinned-cursor launch fails the gate
  without starving the converged ones. A walk-side reposition (step to the doorway's centre line,
  z ≈ −179.4, before a west/south-west bearing) is the player-model fallback.

## Verified (this session, NF_ISO set, --test-threads=1, all foreground)

- walkreal_07000008_live (whole binary): **5 passed / 1 failed** — the topped-up plain walk ✓,
  the d1 real-damage sister ✓, the d2 Agent sister ✓ (both re-verified under the new gates; their
  winning launches were all ≤ 45 u so their timelines held), both ally tests ✓; the d3 00agent
  sister red (documented above).
- nf-game lib: **692 passed / 0 failed** (5 pre-existing ignored).
- glass_pass_every_level_live: **3/0 + 4 ignored**.
- The bullet binaries: bullet_range_live **3/0**, rocket_glass08_live **3/0**,
  fix_foliage_bullets_live **3/0**, smg_close_range_live **1/0**,
  stairwell_bullets_07000011_live **2/0**, fix_flesh_impacts_live **4/0**, breakables_live **2/0**.
- walkreal_07000002_live (the shared-helper control, rule 7 — the helper files are untouched this
  session, the 02 run is the confirmation): **3/0 + 1 ignored**.
- The full four-crate suite (tools/test/suite.py): **could not run** in this sandbox — background
  processes are killed between tool calls here (measured: both `nohup` and `setsid nohup` dies
  within seconds of the call returning; suite.out stayed 0 bytes), and the foreground full suite
  does not fit the window after the walk runs. The binaries above are the coverage; the next
  session should start the suite first thing and run the full walkreal family.

## Live checks for the main agent

1. 07000008, fresh boot, 00 Agent: stand in the bottom terminal's cabin at the ride's end
   (111.05, 13.54, −179.71), fire an AT-420 rocket at the Hind on a west/south-west bearing
   (machine around (81, 24, −199)) — where does the round die, and does the carcass of the launch
   show the round clipping the doorway's south frame band? gdbwatch on the rocket's BU fields
   (state/BU+0xF8) through the first 10 ticks settles whether the stillbirth round stops on world
   geometry (and why no blast registers — the flush's pierce roll?).
2. 07000008, fresh boot: the Hind's missile (weapon 0x62) blasts — record the radius/damage pair
   the missiles carry (the −2 near misses at ~7.8 u and the −8 finale need the missile's own
   EXPLODEINFO numbers to size the stand-off the fight needs).

## Usage

The session's input/output tokens, context used and cost: not visible to me.
(Wall clock: branch created 04:31 UTC, this report written 05:40 UTC, sandbox deadline ~06:26 UTC.)
