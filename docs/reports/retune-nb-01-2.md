# retune-nb-01-2 — 01 spawn-to-exit-with-the-contact real-damage walks, round 2

Branch `s/retune-nb-01-2` (from origin/master a128047d1 with s/retune-nb-01 merged in; it was
not on master). **d1 (Operative) and d2 (Agent) are GREEN — their ignores are lifted.** d3
(00 Agent) stays red but moved 8668 ticks deeper on its ledger's first lever; the refreshed
ledger and the next levers are in its ignore message. Everything below is judged on its own
real run through the real input path (`walkreal_07000001_live`), no top-ups, no guard-code
changes, no damage/health changes.

## What was measured first

- The round-1 ignore named the pocket-edge tap loop as d1's next lever. Before writing any
  fight code I probed the real end-state on the failing run: a scratch probe at the clear-0
  stand printed, per lane guard, the exposed-aim result, the crouched/standing flight-lane
  ladders, the hit-box geometry dump (`DamageLog::geom_dump`) and a virtual-stand exposure
  grid (the exposed_aim ray set from 16 candidate stands). Facts ([L] the probe on this tree):
  - every stand EAST of x -27 is fully walled for the pocket set — no part ray, no flight lane
    at dy 0.3-1.5, standing or crouched — the old round-0 east-lane fight never fired (the
    partition found all three walled; the run's log shows no fight between the stand arrival
    and the next round);
  - #2 sits on the stairwell structure TOP (park (-30.01,10.22,13.58), y 10.22 — the floor
    slab, run owner 67 bounds y ≤ 9.09, blocks every east ray); #4 parks at (-30.26,9.02),
    #17 at (-31.10,11.80), all south of the pocket wall;
  - #17's HEAD (part 5, dy +0.51) exposes from the corridor's NORTH edge —
    (-28,15.5)/(-28.5,16.5)/(-29,16.5) — local openings in the wall pieces; an arm (part 51)
    from (-27,16);
  - the parks VARY across runs (the round-1 ledger's (-30.1..-31.1, 13.1-15.0) vs this run's
    z 9.0-13.6), so the pocket-edge transit is a dice roll — and the probe run died exactly
    there: the transit to (-30.0,10.2) crossed the guards' advanced spots, #17 parked ON the
    lane at (-31.1,13.5), the forced band [M DroneWeap_DoBulletAccuracy 0x179740:100-110]
    took 50.8 (the v5-v9 shape).

## Levers landed (all level-01 test code + one opt-in walker field)

1. **The plate-lane pocket tap loop** (`plate_pocket_fight` + `pocket_trigger` in
   `walkreal_07000001_live.rs`): the d3 lane-stand fight's shape moved to the plate lane.
   Bond stands at a band-checked north-edge stand (the west-most candidate whose dxz to every
   live pocket guard is >= 3.05, outside the 3.0 forced band [M 0x179740:100-110]), crouched,
   converges the view only — NO approach and NO flank (the shoot machinery's flank ladder
   passed #17's face at 1.1-1.3 u; the v5-v9 deaths). The aim is the exposed part's own point
   (prefer_head's head box within 6.0; the head part is the ×100 record [M NDrone2_HitDamage
   0x145558]); a blocked sight falls to the held-gun's flight-lane ladder (0.75-first
   non-pistol — the d3 head-box law; 0.9-first pistol — the run13 law) with the lane trigger
   [M drone.rs:16465 the round's world trace]. One volley (>= 2.0) ends the window (the
   hall-4 dance's law); the pocket's break is a crouch-hold, not a walk-home (the home
   transit crosses the guards' advanced spots). A walled guard is LEFT AT ONCE — no windows,
   no lure: the first run of the loop measured the blind-lure grind drawing the pocket guards
   out of their parks and onto the unattended CONTACT (he died mid-grind and the exit fired
   as the escort-fail restart [L the v2 run]); the green top-up walks prove the plate walk and
   the recovery succeed with #2/#4 alive, and the plate walk's own return fire killed #4 in
   590 ticks [L the same run]. d3 keeps its old machinery (gated `difficulty < 3`).
2. **The d2 lure/hold crouch** (the ledger's first lever): the d3 crouch law joins Agent —
   the lure burst and the crossing hold run crouched at d2 (d1 keeps standing, byte-identical;
   d3 never runs that block).
3. **The fight-order pass** (the ledger's third lever): measured a NO-OP on the d2 stream —
   agent_order already put the shotgunners first; the sort gate now admits d2 and the death
   tick was unchanged [L the v2 run]. The gate stays (documented measurement).
4. **The d2 volley-race fight kit** (the decisive d2 lever): the SMG pre-arm during the
   entry's quiet waits (`d3_switch_to_smg`), the head-box lane ladder, the loose gates
   (0.035/0.04) and the lane fire join d2 (all gated `difficulty >= 2`, d1 byte-identical,
   d3 unchanged). #7's one-frame 33.6 hall volley is gone.
5. **The d3 ammo economy** (the ledger's first d3 lever): the opt-in walker field
   `no_fetch_zone_x_min` (common/walker.rs, default None — the zone reads exactly as before
   for every other walk) scopes the hall-6 `no_fetch_z_above` zone: boxes west of x 0 are
   fetchable even above z 11.2 — the west-corridor ground is not hall interior. Set only by
   the d3 walk. [L] the d3 death moved t75294 → t83962 (+8668). The field admitted no
   fetchable drops on this stream (the corridor dead were fist guards — no drops; the
   resupply lines read "no pickup within the walked-path corridor") — the depth gain came
   from the changed resupply walk pattern.
6. **The d3 recovery tap posture** (the ledger's third d3 lever, first step): a pocket guard
   (x <= -27) in the recovery loop is fought with the band-checked stand + `plate_pocket_fight`
   instead of `shoot`, gated `!tops_up() && difficulty >= 3`. It never engaged on this stream
   — the death is upstream of the recovery (the next-lever list names the phase).

## What is still red

- **d3 (00 Agent)**: dies t83962 at (-30.64,11.74) — the post-clear `walk_nav_to([-30.8,
  6.4,13.1])` "the stairwell head" NAV's own stuck-flank ladder walked Bond INTO the pack at
  the stairwell mouth (#3 0x5000038's 7.20 pellet at d 0.8 and #17 0x5000039's 3.60×3 at d
  0.9 — inside the ×2 TooClose band [M Drone_ModBulletDamage 0x139f50]; health 17.6 at the
  flank; the flank target (-31.6,16.6) is west into the pocket) [L the v3 run]. NEXT LEVERS
  (in the ignore message): (1) the endgame walk posture — band-check the endgame nav legs
  (the d1/d2 clear's own >= 3.05 rule) or hold the nav flank west of x -27 during the d3
  endgame walks; (2) the #11 window cap (the pool thief — the lane windows on #11 burned
  24.0 across two windows); (3) the recovery tap posture is already in and engages once the
  walk posture holds.

## How it was verified

All runs on the final merged tree (origin/master a128047d1 merged at the branch point; no
master movement since):

- `walkreal_07000001_live` **whole**: **7 passed / 0 failed / 1 ignored** (501 s) — the five
  green walks stay green, d1 and d2 run un-ignored and pass, d3 stays ignored with its
  refreshed ledger.
- `walkreal_07000001_live` individual runs: d1 `walk_from_the_spawn_to_the_exit_with_the_
  contact_real_damage` — exit to 07000002 at tick 29163, end health 46.8 (53.2 damage;
  #17 killed by ONE head tap at the north-edge stand, zero cost); d2 `..._agent` — exit at
  tick 15479, end health 31.8 + armour 50.0 (68.2 total).
- d3 `..._00agent`: t83962 (was t75294) — the ledger's depth gain, ignore refreshed.
- `interactables_07000001_live` whole: **10 passed / 0 failed** (25.4 s).
- The fast suite (`tools/test/suite.py`): **no total line** — its build step needs all four
  crates' test binaries and the sandbox disk filled (9.9G rootfs; the all-tests build wrote
  ~6G into target/ and the linker hit "No space left on device" on four binaries). The
  equivalent coverage was run by hand instead (the round-1 precedent): the required 01
  binaries whole (above) plus the rule-7 walkreal sweep for the shared-helper touch
  (`no_fetch_zone_x_min`, default None): walkreal_07000002_live 5/0 (222.6 s),
  walkreal_07000004_live 10/0 (49.6 s), walkreal_07000005_live 16/0/1 (133.3 s),
  walkreal_07000006_live 9/0 (77.7 s), walkreal_07000008_live 6/0/1 (230.4 s) — 46 passed /
  0 failed across the five that fit the window (the level walks are minutes each and the
  sandbox kills parallel processes above four; 03/07 were OOM-killed, the rest did not fit).
  The field is default-None additive, so the untouched levels are stream-identical by
  construction; the five runs above confirm it in the sim.
- The scratch probe block was removed before commit (the probe's facts are recorded in the
  code comments and this report).

## What is left

- d3's endgame walk posture (the named lever above), then the recovery tap posture engages.
- The rest of the rule-7 walkreal sweep (03, 07, 09, 0a, 0b, 0c×4, 0d, 11, 11_route, 12, 13,
  1b and the probe binaries) on a machine/box without the 2-hour window and the 10 GB disk.
- A full `tools/test/suite.py` total line on this tree.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for
the session (no cost or usage numbers were displayed anywhere in the tooling).
