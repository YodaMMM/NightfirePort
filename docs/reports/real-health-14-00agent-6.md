# real-health-14-00agent-6 — the 14 d2 and d3 walks' health budget: the grind is fixed and measured; the wrap and the un-ignore are parked on the fresh ledgers

Branch `s/real-health-14-00agent-6` = origin/master (22d5836b7) + the merge of
`s/level-14-landing-hover` (the wedge fix and its laws, conflicts resolved additively) + this
window's levers in `crates/nf-view/tests/walkreal_07000014_live.rs`. Status: **the driver's
named lever was measured to its root — the "stairwell AIM pairs" were a MISREAD, the grind's
real drain is the trench guard's point-blank trade, and the sisters' head-shot law cuts it
65.0 → 15.0 — but the d2 `with_real_damage` wrap and the d3 un-ignore did NOT land: the walk
tree beyond the descent is dominated by corridor-block hunters the fights never pick, and
their cooldown-gap bursts re-roll 30.0–65.0 wide per run. Both tests are parked in their
master forms (the binary green 9/0/1) with the measured record and the ordered next levers.**

## What's done

1. **The merge.** `origin/s/level-14-landing-hover` merged on top of origin/master 22d5836b7:
   both DamageLog opt-ins kept (`eye_ray_stands` from master, `descent_stationary_fights` from
   the landing-hover), the newer 014 notes kept, WORKLOG entries concatenated. The landing
   wedge stays fixed: every run this window crossed the 1.72 landing cleanly and the fights'
   descent stands cost 0.0 [L every run log this window].
2. **The grind's drain located — the named lever was a misread.** The landing-hover report's
   next lever was "the infiltration grind's smoother early game (the 05-07 sniper-stand law on
   the stairwell's AIM pairs)". The fresh d3 ledger [L /tmp runs on the merged tree,
   2026-11-07, reproduced across five runs] shows the AIM pairs never landed a hit: **65.0 in
   13 hits came from trench guard #0 (skin 0x5000090)** — the 25-hp hunter closes across the
   40-tick weapon raise and trades at 1.2 u, where every drone round hits [M
   DroneWeap_DoBulletAccuracy 0x179740:113-115], while the torso loose gate needs eight 3.5
   taps. The stairwell lane fights (#1/#2) ran "killed false" for ~0 health cost in every run.
3. **The sisters' head-shot law (the fix that landed).** New file-local `shoot_head_loose`
   (no shared-helper change): the aim rides the part-5 box centre with the game's own
   `t_box < world_t` entry comparison [M 0x1ea398:216-229; walker.rs `exposed_aim`'s head
   law], torso +0.3 fallback, the caller's `shoot_loose` as the backstop. One P2K head round
   is 35 [M NDrone2_HitDamage 0x145558 × DroneDamage_Head; the P2K's 3.5 → 35] — the guard
   dies at **+49 ticks** and the grind then costs **15.0** at d3; the d2 twin's grind through
   the hand-back measured **3 hits for 9.0** (health 91.0 at the pad west) [L the d2 wrapped
   run]. Gated `AGENT_PATH_CLEAR && !w.tops_up()` — d1 and the d2 TOP-UP sister keep master
   behaviour (see 7).
4. **The leg-2 block answered: the sisters' reloads.** The stationary descent window skips
   the fights' resupply fetches (the field doc's own law), so the fights reached the leg-2
   spot with clip 1 and the dry stands could not trade — the measured deaths clustered there.
   A file-local `reload_full_b2` (60 blind R pulses, walker.rs's `reload_full` contract;
   walker_b2's tick_inner cannot pulse the reload key) runs for the sisters at the corridor
   floor and again before the post-leg clear. With rounds the fights killed **SEVEN guards
   for 27.5** through the leg-2 block [L the d3 run, ticks 4158–5049].
5. **Three levers TRIED and measured NEGATIVE / NO-OP, and reverted** (the records live in
   the test file's comments):
   - a corridor-mouth `clear_hostiles` pre-clear: the block HUNTS — the hunters closed onto
     Bond while the clear stood at far picks (17–21 u), death at 5021 [L];
   - the loot pass inside the block: the fetch walked a drop at (28.56, 32.02) INTO the
     surviving pair's 1.7–2.2 u band — 47.5 in 230 ticks, death at 5281 [L]; the sisters'
     post-leg clear now runs loot-less (`clear_hostiles` gained a `loot: bool`; every d1 call
     site passes `true` — unchanged);
   - `eye_ray_stands = true`: a byte-identical no-op (the d3 run repeated tick-for-tick) —
     the hunters are never PICKED at all (no fight line names them across five runs), so the
     stand-aim law has nothing to aim. The blocker is the fights' shooter pick, not the aim.
6. **The parked park, re-measured.** The d2 wrap (measured this window via NF_WALK_NO_TOPUP
   over the top-up test, same tree): the grind is fixed but the run's re-rolled phases pinned
   the drop-west — death at tick 5151 at (33.71, 1.20, 34.23), the hover pocket's north-west
   corner held ungrounded (the -5 report's hover dither, reached by the drop-west leg's own
   path), while three 0x5000090 guards burst through the fights' five failed stands. The d3
   walk's remaining blocker is the same never-picked hunter class (skin 0x5000087): the 4904
   death is #9 at 27 hits for 65.0 at d 3.6 with zero fights answered. Both `#[ignore]`/top-up
   forms stay, their notes rewritten with this window's ledgers.
7. **A self-inflicted regression caught and fixed (lesson 10).** The first gating used
   AGENT_PATH_CLEAR alone — but the d2 TOP-UP sister also sets it, and the reloads/clear/
   head-shot re-rolled its phase-pinned route into the corridor-south door wedge (the 6000-
   tick cap at the door (32.0, 2.1, 35.8) [L]). All three laws now carry `!w.tops_up()`; the
   whole binary went back to green.

## How it was verified

Every run whole, release, `--test-threads=1`, real ISO (`NF_ISO`), `git status --short`
checked before each commit, only the task's files staged:

- `walkreal_07000014_live` whole: **9 passed / 0 failed / 1 ignored** (149.17 s) — the d1
  real-damage walk green (the driver's bar), the d2 top-up sister green again after the gate
  fix, the cascade test green.
- The fight-path twins on the merged tree (the shared `damage_log.rs` came from the merge):
  `walkreal_07000013_live` **3 passed** (37.66 s), `walkreal_07000012_live` **3 passed**
  (20.18 s), `walkreal_07000011_live` **5 passed / 0 failed** (60.32 s).
- The measurement runs: the d3 ignored test ×5 (the levers' A/B ledger), the d2 wrapped
  measurement ×1 (NF_WALK_NO_TOPUP), each quoted above.
- `nf-game`: **92 passed / 0 failed**. `nf-formats` + `nf-audio`: 233 passed / **4 failed** /
  3 ignored — the 4 are the slim-ISO disc-golden trio (`bigf_header_fields_behave_as_
  documented`, `bigf_tables_match_the_golden_manifest`, `directory_tree_matches_the_golden_
  manifest`) plus `the_real_disc_passes_the_identity_check`; both crates are byte-identical
  to origin/master on this branch (`git diff origin/master -- crates/nf-formats
  crates/nf-audio crates/nf-game` is empty), so this is the environment's state, not this
  branch's.
- **The rest of the walkreal battery did not fit** — the full nf-view binary set (~100
  binaries) needs a build longer than the two 10-minute windows this session could give it at
  `-j2`. Same merge-time note as the landing-hover window: every opt-in flag defaults to
  false, every touched gate compiles to the identical expression when unset, and this
  window's only source changes are the 014 test file (its binary green whole) and the merge's
  already-verified flag sites; the three twins above cover the fight-path family. Please run
  the remaining `walkreal_*_live` set at merge time.

## What's left

1. **The fights' pick question** (the d3 un-ignore's root): the corridor block's hunters
   (skin 0x5000087 + the 0x5000090 south block) are never picked by `DamageLog::shooters`/
   the lane pick — no fight line names them in five runs — while they chase and burst
   30.0–65.0 in the cooldown gaps. A pick-level trace (why the nearest 3.6-u shooter is not a
   candidate) against the real game is the next lever.
2. **The d2 wrap** — one line, on ground that needs the drop-west pocket crossing
   de-rolled: this window's re-roll held Bond ungrounded at (33.71, 1.20, 34.23) while three
   guards burst; the sim-side scoped hover fix (the -5 report's lever 1, with the fresh-boot
   [L] reading) remains the structural unblocker.
3. The full suite + the remaining `walkreal_*_live` battery (see above).

## Live checks for the main agent

1. **The under-slab pocket (07000014, ~(33.9, 0.7), z 31..35.5)** — carried over from the -5
   and landing-hover reports, unchanged: with the game's Bond dropped into it, does the REAL
   game hover `grounded=false` over the corridor floor, or land? Decides the scoped sim-side
   rule's shape.
2. **Guard #8/#9/#56-class (skin 0x5000087) at the descent's south block** — the sharpened
   form of the landing-hover's live check 2: from the corridor floor spots where the port's
   fights DID land rounds on the 0x5000090 guards, does the REAL game's Bond have a firing
   answer to the 0x5000087 hunter while it closes (d 3.6 in the measured death)? A
   `fieldprobe.py` read of the hunter's pos over 10 s of play plus a screenshot from the
   corridor floor would settle whether the port's pick windows (or its ray set) diverge from
   the game's own target choice.

## Usage

not visible to me — no token, credit, context-window or cost figures are shown anywhere in
this environment's interfaces for this session.
