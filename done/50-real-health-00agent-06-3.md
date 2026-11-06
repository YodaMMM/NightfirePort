# real-health-00agent-06-3 — the 06 00 Agent deck-guard lane (low priority)

Continue origin/s/real-health-00agent-06-2 (merge origin/master first) and read its report: the sister reaches the courtyard descent and dies in the deck-guard lane. Take its next levers. Real input only; no top-ups; helpers level-local. Done = walk_the_castle_interior_to_the_party_real_damage_00agent green, or ignored with a measured reason; every walkreal_*_live; the full suite total line.

---
claimed-by: sandbox3 at 2026-11-06T06:47Z

---
## Report (done, 2026-11-06T07:13Z)


---
## Report from the worker (done, 2026-11-06T07:13Z)

# real-health-00agent-06-3 — the report

Task: the 06 00 Agent deck-guard lane. Continued `origin/s/real-health-00agent-06-2` (713f4c8),
merged origin/master (c2c4a6e "The last two ignored walks are back"). Real input only; no
top-ups; the helpers stayed level-local (no shared-helper changes — `common/walker.rs` is
untouched, read-only).

## Status: NOT finished — the 00 sister is still `#[ignore]`d, now with this session's measured
## reason in the ignore text. The walk moved from the guardroom wedge to the strip.

The merged master changed the game's facts on this level twice over, and both changes bit
before 06-2's deck-guard lane was even reached:

1. **The guardroom approach wedged.** The doorway pre-clear's budget windows (the full
   6000-tick arms that kill the door-route guards) outrun door #47's 360-tick auto-close
   [M Door_Update]: the nook nav then met the leaf at pos01 0.50 state CLOSING and burned
   4 × 3000 stuck ticks at the jamb (the stuck ladder's 2.8 u door-find radius misses the door
   at d 2.96), and the 5th attempt's recovery diverged to the hedge under the deck snipers —
   the deck posts' sight param 48-49 u [D census #446 sight 48, #448 sight 49] reaches the whole
   guardhouse court, and the run's ledger shows the y-47 deck post landing 45.0 in ONE hit at
   d 35.9.
2. **The ammo economy broke.** The same merged master's guard reactions (the scared-civilians
   merge) made the doorway a three-guard fight (#3 inside unexposable, #4 the door-route
   charger, #27 the chaser), and the walk reached the balcony with clip 0 spare 0 — every
   aimed stand bailed DRY (silently, before this session's logging) and the deck post #29
   killed the unarmed crossing (64.2 in 3 hits, weapon 0x25, 40.0 a hit).

## What's done (all committed and pushed, each measured on a rebuilt binary)

1. **The guardroom lane unblocked** (d3-gated, real input): after the pre-clear the walk
   returns to the proven pre-door spot (35.5, 39.1, −12.7 — the d1 walk's own arrive) and
   re-presses E at door #47 before the nook nav. The nook nav then passes on its first attempt
   (run 3: nav 1 arrived +82 ticks — was 5 × 3000 stuck ticks). An `exposed_only` fast-skip for
   never-exposed targets was measured at the doorway and REVERTED (it trades #4's kill for the
   resupply death: 97.2 in 22 hits, run 2); it is kept ONLY for the two nook clears, where #27's
   court post is wall-blocked from inside at every phase (runs 1/3: two windows burned 287+410
   ticks and 18 resupply rounds on him, then the second resupply detour was refused).
2. **The ammo boxes are looted** (d3): the guardroom's own ammo sits at its south-east [D census
   #666 AmmoPickUp300Soft (34.27, 40.01, −24.12) + #770 Ammo_5_56mm (34.72, 40.00, −24.45)]; when
   the gun is under 60 rounds the walk takes a walked leg to them and resupplies (run 4: clip 17
   spare 90 at the balcony). The walker's own resupply refuses the off-corridor detour, so the
   loot is an explicit leg.
3. **The d3 balcony drop restored**: the d3 flow never walked the drop legs, and on this master
   the mouth nav from the balcony follows the ramp lane back down east — no ch-0x34 ever fires
   (run 4 died on the plate pin at the tunnel approach with the whole courtyard crossed at
   64.2 hp). The drop leg (the d1/d2 branch's own (3.0, 41.0, −23.0) try_walk_to) runs at d3
   after the balcony stands; the plate pin passes (runs 5-10).
4. **The aimed-stand bails are loud** (log-only): DRY / NO-HIT / UNSEEN and an engage line —
   the silent early returns were hiding the tier's failure mode (run 3's dry balcony looked
   like "no stands ran").
5. **The stand pick triages the NEAREST threat** (was recent-hitter first): the tunnel-roof
   guard #11's 0.00-damage grazes from d 29-41 register as player_hits, so hitter-first
   re-engaged him THIRTEEN stands in a row (each a flicker bail — his parts poke in and out of
   the roof parapet) while the ch-55 pair walked unopposed into sight. Nearest-first killed
   #20 at the landing (run 9: #20 hp −11, #19 at 3).
6. **The mouth phase goes silent at 00**: the deck posts the tier answered (#12/#29/#30) are
   already dead by the mouth on this master (they chase during the guardroom fights and fall —
   runs 4-7 censuses), and every burst at the mouth pulls the ch-55 pair west off their strip
   posts while they spawn-walk.

## What's left (the next session's named levers)

The walk now dies at tick 5879 at (5.42, 42.03, −32.91) — the strip arrive — to the pair AT
THEIR POSTS ((13.7, −35.5)/(14.0, −35.4) [D census], d 8.7-8.9, inside their 12 u sight) plus
#11's roof lane (38.5 in 7 at d 23.4). The landing stands engaged only #20 and the stand
returned with NO bail line and NO kill line — the remaining silent path is the
target-vanished return (`corpse || health <= 0 || !present` → true), i.e. the pair reads
present-false mid-settle at the landing. Levers, in order:
1. **Probe the pair's present/enable timeline at the landing** (a census eprintln per second
   during the landing phase settles whether they blink present-false while spawn-walking, and
   whether the landing spot (4.0, 41.5, −23.5) ever gives them rays1 at their posts — run 6's
   post-wait census DID read #19 rays1 d 12.8 / #20 rays1 d 11.8 there, mid-advance).
2. **A stand spot for the pair outside their 12 u sight** that isn't the mouth plaza (#11's
   field): the balcony's west reach is 14.7 u from their posts and level-ish — the drop leg
   currently leaves before the pair enables (they enable on the mouth plate's alarm, after the
   drop). A post-drop re-ascent of the balcony ramp (real input, the d1-proven lane) with the
   stands from the lip is the player's own high-ground answer; it also puts #11's roof post
   (y 55.5) into a shorter ray set.
3. **The strip arrive needs the stands BEFORE the last nav leg**: the pair's posts shoot the
   (5.99, −32.72) arrive at d 8.7 — split the strip nav at (6.0, −28) (probe the sight band
   first), stand, then the last leg.

## Verification (this session, on the final pushed tree)

| Binary | Result |
| --- | --- |
| `walkreal_07000006_live` (whole, 7 tests) | 6 passed, 0 failed, 1 ignored — the ignore IS the 00 sister, with the measured reason in the ignore text |
| `rook06_realpath_live` | 1 passed, 0 failed |
| 00 sister iterations | 10 measured runs (the merged-tree baseline + one per lever), each rebuilt first; the ledgers quoted above are in the session logs (not committed) |

The full suite did not fit the window: the merge re-measure + ten 00 runs + the whole-06 and
rook06 binaries consumed it. The touched file is test-only (no `crates/*/src/` change; the
branch's diff vs the base branch is the 06 test file, WORKLOG, this report and the merge), so
the lesson-10 set is the 06 + rook06 binaries, both green.

## Live checks for the main agent

1. The ch-55 pair's spawn-walk: does #735 drop them AT the strip posts (11.6, 42.0, −35.1 /
   13.2, 41.7, −34.8 [D census]) or at a spawner origin ~10 u west (the port's runs read #19 at
   (0.1-1.2, 42.0, −31.5..−32.8) mid-walk)? A PCSX2 watch on the pair's positions for 300 ticks
   after the ch-55 rise pins which side the port's spawner modelling diverges on.
2. The tunnel-roof guard #11 (y 55.5, the roof post at (−0.6, 55.5, −51.1)): from the landing
   (4.0, 41.5, −23.5) and from the mouth plaza (8.4, 42.0, −12.0), can a standing Bond land an
   AIMED shot on him (the muzzle-ray question the port's strict ray set answers flickering-yes
   at the landing)? One PCSX2 aimed shot from each spot pins whether the port's muzzle stand-in
   is over-strict (the game's own aim may hold the roof post steady).
3. The deck posts' chase-and-fall: the port's runs show #12/#29/#30 dead by the mouth without
   Bond's fire (they chase during the guardroom fights and fall off the deck). Is that the
   game's own AI (the deck posts leaving their posts on the courtyard alarm), or a port nav
   artefact? A PCSX2 watch on the deck posts' positions during the guardroom fight phase
   settles it — it decides whether the 00 tier's balcony stands have real targets to answer.

## Usage

Not visible to me: this environment exposes no token, cost or context-window figures for the session.
