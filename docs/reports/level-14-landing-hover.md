# level-14-landing-hover — the 1.72-landing hover: the wedge is fixed walk-side; the d2 wrap and the d3 un-ignore are measured to their next levers

Branch `s/level-14-landing-hover` = origin/master (dcd51c7e8) + this window's commit (d96f7863b).
Status: **the driver's lever 2 is implemented and measured — a d2/d3-gated walk-side law set
whose stationary-stand window through the mezzanine descent removes the fights' displacement
and with it the 1.72-landing hover: every run this window crossed the landing cleanly (the
landing leg arrived at +280 ticks, the drop west cleared, the descent assert passed), where the
-5 window's runs wedged at tick 25740/26340. The d2 `with_real_damage` wrap and the d3
un-ignore are NOT landed — the same measurements moved their blocker from the wedge to the
fights' health budget, and both tests are parked in their master forms (the binary green 9/0/1)
with the measured record and the ordered next levers in their notes.** No collision code was
touched (the driver's condition — collision only with a fresh-boot [L] reading — never came
into play: the fix is walk-side only, in the test helper's opt-in flag and the 07000014 walk).

## What's done

1. **The descent's stationary-stand window (the lever-2 core).** New opt-in flag
   `DamageLog::descent_stationary_fights`
   (crates/nf-view/tests/common/damage_log.rs, own line before the private fields, documented
   with the wedge record). While set, the fights still run — the shooters must die (see 2) —
   but the stand's feet are tied: no back_off shove (`crowd`), no creep, and the lane
   resupply fetches are skipped (a dry gun walks on). The 07000014 walk sets it for every
   `difficulty >= 2` body call and the shared `walk_the_west_route_to_the_switch_band` clears
   it right after the descent block (the west legs' fights resume their full law; d1 and the
   cascade test never set it).
2. **The no-fight variant was tried FIRST and measured dead — the fights must stay.** With
   `return_fire` fully stood down during the descent, the kill ledger starved: the unanswered
   far guards drained the whole 100.0 budget (death at tick 5328 in leg 2, only 8 fights,
   #12 21.0 + #9 19.0 unanswered). The stationary variant keeps the fights' kills and removes
   only their displacement — exactly the driver's framing.
3. **The measured wedge is gone.** With the stationary window: the landing leg "the 1.72
   landing" arrived at +280 ticks at (33.87, 2.28, 35.36), the drop-west loop cleared the
   overhang to (30.22, 0.59, 35.45), the descent assert passed at (29.80, 0.57, 37.63) — on
   every d2 run (three runs) and on the d3 run. The -5 window's wedge (tick 25740, (33.90,
   0.68, 35.38), grounded=false; the d3 twin tick 26340) never recurred. The hover pocket was
   never entered by fight steering again.
4. **The escalation lever (new blocker #1: geometry-blocked far guards).** The first
   stationary run died in leg 2 to guard #8 (skin 0x5000087 at (27.7, 1.0, 46.2) [L the death
   run's ledger]): SEVEN stands vs him all "killed false" — his boxes are behind geometry the
   fights' gun rays never clear from any route spot — and each failed stand is stood-tick
   exposure (one stand cost 18.0; the flat 240 cooldown re-took the same spot every 240
   ticks). The existing opt-in `escalating_cooldowns` (the 0700000c Agent walk's law) is now
   set in the d2/d3 gate: with it #8 dies ("returned fire on drone #8 (killed true)", the
   whole corridor chain lands) and the walk reaches tick 18851.
5. **The d2 head-hunt (new blocker #2: the stands' burst exposure).** With the fights
   landing, the budget's remaining drain was the stands' own exposure (100.0 spent by tick
   15579 in the escape-loop grind). Adding `with_real_damage_head_hunting` to the d2 wrapper
   (the d3 sister's own law) halved the stood ticks: the walk then crossed the sabotage, the
   escape touch, Rook's wake approach and died at tick 18851 at (23.08, 1.13, -21.36) — ONE
   LEG short of the delta doorway — with 3.2 health. Two stands remain expensive (#32 14.7,
   #30 13.5).
6. **precise_fights was tried and reverted** (measured): the 0.01 tap gate starved the
   stands' give-up laws and stretched a south-loop leg to 41k ticks (fights at 7809 → 49105);
   the walk died later on the same budget. The plain gate stands; the try/measure is in the
   walk body's comment.
7. **The d2 wrap and the d3 un-ignore did NOT land** — the honest park. The d2 walk with the
   full law set survives to one leg short of the exit but dies at 3.2 health (the fights'
   budget, not the wedge, not the collision); a red wrapped test cannot be pushed. The d3 walk
   dies earlier — the infiltration grind at accuracy 1.3 takes 45.0 before the descent (the
   pad west stood at 55.0), the failed stands vs the far guards take most of the rest, death
   at tick 6786 at (30.27, 0.55, 35.45). Both tests are in their master forms (the d2 top-up,
   the d3 `#[ignore]`d) with their notes rewritten to carry this window's measurements and the
   ordered next levers: the infiltration grind's smoother early game (the 05-07 sniper-stand
   law on the stairwell's AIM pairs — the -5 note's third lever), then the d2 wrap (one line,
   the ground is measured), then the d3 un-ignore.

## How it was verified

Every run whole, release, `--test-threads=1`, real ISO (`NF_ISO`), `git status --short`
checked before the commit, only the task's two files staged:

- `walkreal_07000014_live` whole: **9 passed / 0 failed / 1 ignored** (197.88 s) — including
  the top-up d2 walk (now running with the stationary-descent + escalation laws) and the d1
  real-damage walk, both green.
- The shared-helper battery (lesson 7; `damage_log.rs` changed), the fight-path twins first:
  `walkreal_07000013_live` **3 passed** (38.46 s), `walkreal_07000011_live` **3 passed /
  1 ignored** (100.40 s), `walkreal_07000012_live` **3 passed** (23.89 s).
- **The rest of the battery did not fit** — the sandbox window closed before the remaining
  `walkreal_*_live` binaries could run. Risk note for the merge owner: the flag defaults to
  false and every touched gate compiles to the identical expression when it is unset
  (`!false && X` ≡ `X`), so the untouched binaries' behaviour is byte-identical by
  construction; only the 07000014 walk sets the flag. Please run the remaining
  `walkreal_*_live` set at merge time.

## What's left

1. The d2 `with_real_damage` wrap — one line on the measured ground (the walk dies at 3.2
   health one leg short of the exit; the fights' remaining stood-tick exposure ~13-15 in two
   stands is the named leftover).
2. The d3 un-ignore — needs the infiltration grind's smoother early game first (the stairwell
   AIM-pair stands, the -5 note's third lever): 45.0 of the budget is gone before the descent.
3. The full suite + the remaining `walkreal_*_live` binaries (see the battery note above).

## Live checks for the main agent

1. **The under-slab pocket (07000014, ~(33.9, 0.7), z 31..35.5)** — carried over from the -5
   report, unchanged: with the game's Bond dropped into it, does the REAL game hover
   `grounded=false` over the corridor floor, or land? Decides lever 1's shape (a scoped
   sim-side push rule) if the walk-side laws are ever deemed insufficient.
2. **Guard #8 (skin 0x5000087, placed ≈ (27.7, 1.0, 46.2))** — is his sight line to the
   descent corridor really open in the real game while the port's gun rays to his boxes are
   all blocked? A `fieldprobe.py` read of his pos at load plus a screenshot from the descent
   corridor would settle whether the port's ray set is too strict or his cover is real.

## Usage

not visible to me — no token, credit, context-window or cost figures are shown anywhere in
this environment's interfaces for this session.
