# WORKLOG

## retune-nb-16 (2026-11-18, s/retune-nb-16)

Task: commit 3412a89a3 made every guard run the game spacing scan and the close-range alt attack from the fire states; the level 16 walks `walk_the_chasers_defeat_completes_the_mission_on_real_damage_agent` and `_00agent` now die. Read both ignores and the 16 walk changes in that commit, fix the fight that kills Bond (real input only, no top-ups, never lower damage or health, never change guard code), helpers stay gated to level+difficulty. Remove each ignore when green; if still red, push the measured ledger and the next lever into the ignore message. Run the whole walkreal_07000016_live binary.

### retune-nb-16 progress

retune-nb-16 progress (2026-11-18): the Agent walk (d2) is GREEN — un-ignored. The lever is one
gate change, file-local: the squad-phase close stands (the 00 Agent law, 120-tick bounded stands on
any live hostile inside 10 u between the member slices) extended to `difficulty >= 2 && !w.tops_up()`.
The measured death it fixes (t25168, reproduced byte-identical first): with the spacing scan live the
post-chaser pit-shooter clear picks NOTHING (no guard exposed from the fight's end spot), so the
pit-room AR guard #15 (0x50000b2, 0x14, hp 20) enters the squad phase alive, hunts Bond through the
10800-tick far-member stand (invisible #13), closes to 2.0 u (inside the 3.0 always-hit band
[M 0x179740:113-115]) and pours 48.7 in 10 hits; the dealer answer fired once (4 of 20) before Bond
died INSIDE it. Green run: squad audit clean (21 hits, every delta within the game's chain), ch
0xbf/0xc0/0x82, the button, MissionState 6. Sisters/punch/d1 untouched (gates verified).

The 00 Agent walk (d3) is still RED at the furthest measured point; the full ledger + the next lever
are in the test's ignore message. Landed d3 levers (all `difficulty >= 3 && !tops_up()`, file-local):
the east-high ring guards join the peek cycles' fight list; the close pair fights from the peek; the
exit stays fought. Measured-worse levers tried and REVERTED (each on its own real run): level_fights
at d3 (t3912 ramp death), the after-exit resupply (t3608), the ramp-foot perch stand (t4230, the gun
is at clip 1 spare 6), east-high x>8.5 + peek r 0.9 (t6008 fist convergence). The ammo chain (the
exit-fight drops 2-6 u off the path; the resupply trail corridor misses them) is the next lever.

Binaries at the final tree: walkreal_07000016_live 18 passed / 1 ignored (the d3 walk, ledgered);
the three smaller crates' full binary sets run in the background at push time (the nf-view binary
set ENOSPCs this sandbox — the report documents it).

## retune-nb-16-6

Branch `s/retune-nb-16-6`, from origin/master 2bd059afe (stream batch 3 + the 16 pit-squad-aim
re-tune are on it), merged `origin/s/retune-nb-16` (c25e3d00c) per the task — the merge resolved
to master's side: master already carries the round-1..5 levers and their measured history (the
east-high fight list, the close-pair-from-peek, the exit fought, the squad close stands, the
16-2 wide corpse-hop MEASURED-REVERTED note, the 16-5 east-armour fetch + squad laws), plus the
newer stream re-pins. The d3 real-damage walk is still ignored with the 1121b-era ledger
(pocket glass-pane death t2200-5600); batch 3 postdates it, so the first step is a re-measure on
the merged tree.

The batch-3 re-measure (fresh runs on the merged tree): the pocket excursion is dead — the
standing dash bled 100 → 26.5 and the pocket died in 80 ticks to the descended ground closer #0
over the 0.1 u lip (t1473); the crouched dash improved the entry (40.5) but the pocket still died
(t1304). The landed lever: the batch-3 fight-through (d3 skips the excursion, runs the d1 law set
finish_fights/back_off 3.5/hurting_first/level_fights, the ring-east pre-clear gated until ring
level). Measured: the ring REACHED for the first time on batch 3 (the lane crossed ~90 ticks/leg,
the ramp climbed), death on the ring at t1229 — the refreshed ledger + the next lever (the
ring-phase bleed, the south-gallery brute pair #2/#5) are in the ignore message. Whole
walkreal_07000016_live binary: 18 passed / 0 failed / 1 ignored (the d3 walk, ledgered).

## retune-nb-16-7 (2026-11-22, s/retune-nb-16-7)

Task: the 16 chasers' defeat 00 Agent real-damage walk (walk_the_chasers_defeat_completes_the_
mission_on_real_damage_00agent) green on current master (round 7). Branch from origin/master,
merge origin/s/retune-nb-16-6 first (it was not on master), read the 16-6 report + the ignore.

retune-nb-16-7 progress (2026-11-22, IN PROGRESS — the session's window ends before the walk is
judged): the merge resolved to master's side for README/census (master deliberately dropped the
credit line; master's census is newer) and to the 16-6 side for the walk file's ignore (the
ledger). The stream moved again on the merged tree: the -6 ledger's t1229/ring-1..2 death
re-measured as death t1027 AT the ring east arrival (17 hits for 100.0; the south-gallery pair
#2/#5 dealt 86.0 — 58.0 + 28.0) — the new master base (48bfea13f's per-tick tables) shifted the
stream. The measured levers this session (each its own real run, NF_DMG_TRACE, all file-local to
walkreal_07000016_live.rs, all d3-gated by re_arm && station_armour && difficulty>=3 && !tops_up
so the sisters' streams are untouched):

- run 1: the 16-6 ignore's ring-1..4 bounded stands (the punch walk's ring-1 gallery-rifleman
  lever extended) — they engaged at the ring east ARRIVAL and lost the race (Bond at 33.5,
  death t1027 mid-stand). Measured: the stands must start at the first ring-level nav node.
- run 2: the ring east split (nav to the node (14.22,5.73,-81.07) then the stands) — the stands
  died INSIDE switch_off_the_delta's cycling (the probe: gun 0x11 clip 2 spare 9 at the node,
  the P2K clip 7 carried 41 unfired; death t1019). The switch is 61 ticks a try — fatal under
  the pour.
- run 5: the Delta->P2K switch moved to the route head (the corridor's quiet stretch, the run's
  first landed hit is t335 on the lane east) + the lane south dealer stand (dealer-first by the
  walk's own player_hits audit): the #2 kill took 335 ticks (the P2K's 2.1-a-round body taps,
  10 rounds) for ~25 health — the walk reached t4313 (the ring phase entered) but the stand's
  540-tick flank nav walked Bond into the trench (death y -4.99).
- run 6: flank_hold_x 15.0 around my stands (no trench flank) + cap 700 — the blocked branch's
  90-tick SIDESTEPS still walked Bond 16 u west off the lane into the trench rim (death t2815,
  y -6.68).
- run 7: the walk-wide shooters_window (40, 7.5) — MEASURED WRONG: it opened reactive fights on
  the GROUND legs (the ground hunter #1 in band there) and shifted the whole stream (the #2
  burst t335 -> t586). Reverted to per-leg.
- run 8: the peek stand (a no-displacement aim+tap stand, file-local) + the per-leg shooters
  window (40, 7.5) on the level-phase legs only: the reactive fight KILLED the gallery brute #5
  mid-nav for 14.0 (the cheapest answer measured on this stream) — then the node stand stood
  still under the strafed-north rifleman pair's combined pour and bled 44.0 -> 0 in 130 ticks
  (death t1598 at the node).
- run 9 (the composition now on the branch): the early switch + the lane south stand (the #2
  kill) + the per-leg wide window + NO ring-level stands (the reactive fights are the ring
  answer; the stands are measured death magnets at level). The ring phase still died at t1598
  in the reactive fight under the multi-guard pour (0.35-0.67 hits/tick standing at level —
  the d3 batch-3 pour cannot be fought standing).

State at the window's end: the walk is still RED (death t1598, the ring east node area). The
whole-binary runs have NOT been re-run this session (the d3 walk is the only changed test; the
sisters' streams are gated untouched by construction). The next levers named for the next
session: (a) the fights' speed — the dropped 0x1a SMGs (7.0 a round vs the P2K's 2.1) via the
ring clears' own loot walk + a deepest-mag swap (the clear's switch_to_a_deep_firearm keeps the
P2K because its 36 rounds pass the >=10 check — a d3 arm should prefer the 0x1a when owned);
(b) the fights' trade discipline at level — the multi-guard pour (0.35-0.67/tick standing)
exceeds any stand longer than ~100 ticks from the node's ~58 health; the fights need to break
off after one kill (the determined pass + finish_fights re-stand is the exposure that killed
run 9).

retune-nb-16-7 final (2026-11-22): the ramp-2 re-stand measured WORSE (run 11: death t1428 at
the ramp 1 arrival, the stream shift again — the run-7 lesson repeats); reverted. The final
composition (runs 10 = 12 byte-identical, the determinism check): the early P2K switch + the
lane south dealer stand + the peek stand + the default machinery — death t1620 at (10.70, 6.75,
-77.94) ~10 u past the ring east arrival onto the ring, the node arrival improved 47.5 -> 58.0.
The refreshed ledger + the next levers (the 0x1a speed arm in the clears, abort_on_wound trade
discipline, the #5/#3 lane batch) are in the ignore message. Whole walkreal_07000016_live
binary at the final tree: 18 passed / 0 failed / 1 ignored (140.1 s, --test-threads=1) — the
d3 walk is the 1 ignored, the sisters' streams gated untouched by construction (no crates/*/src/
and no crates/nf-view/tests/common/ files changed; the diff is the walk test file + docs).
