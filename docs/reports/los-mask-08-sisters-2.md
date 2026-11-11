# los-mask-08-sisters-2 — the 00 Agent walk's first two thirds fixed; the death moved from tick 9592 to the boarding platform's last exchange (26491)

Branch `s/los-mask-08-sisters-2` = origin/s/los-mask-08-sisters (041c73180) + origin/master
(784ba2d99, already an ancestor — the merge was a no-op). Task: the sister report's "What is
left" item 1 — the 00 Agent (d3) sister's two new levers (the d3-only dy-5.0 mouth variant,
then the post-wake re-stand), then the Hind dodge window if the fight still leaks. **Both
levers are in, work exactly as ledged, and moved the death from tick 9592 (the east-hall
bleed, health 8.0 by tick 2475) to tick 26491 (the boarding platform's last exchange, health
72.0 entering the boarding, 17 hits for the whole run vs 25 by 9592 before).** The walk is
still red — the ignore stays, carrying the sharpened ledger and the next levers. Everything is
in `crates/nf-view/tests/walkreal_07000008_live.rs`; no `crates/*/src` file changed.

## What's done

All d3-gated (`difficulty >= 3`) — the Operative and Agent sisters' timelines stay byte-exact
(both passed unchanged in the whole-binary run):

1. **Lever 1 + 2 (the ledger's own pair, in order): the d3-only dy-5.0 mouth re-stand after a
   bounded 240-tick post-wake wait.** `fight_guards_within` now delegates to a
   `fight_guards_within_dy` variant (the plain call sites keep the 4.0 band); the station-hall
   mouth site adds, d3-only, a 240-tick wait then a re-stand with the 5.0 band. Measured: the
   ch-0x2a plate fires on the arrival tick, the guards wake on it, and the post-wake stand
   kills ALL FOUR east-hall guards (#11/#2/#14/#3) at ticks 613-1309 with **health 100.0
   untouched** — the old ledger's 84.0 upstream bleed (health 8.0 by tick 2475) is gone.
2. **The lower-landing wedge skipped (d3).** The leg wedged 3.65 u short for ~3900 ticks
   (STUCK twice, `segment_to_target` None — a nav-web hole at the landing's edge) while the
   station-pit pair plinked from below (−20 in the arc). The tops_up walk's own
   `walkway_pair_holds` skip is the precedent; d3 now skips the landing and takes the approach
   legs ("the stair head" on) directly.
3. **The station-pit ambush pair answered at the ramp mid flank.** #15/#16 (skins
   0x500002e/0x5000030, ambush spawners off the walkway's zone plates) climb from the pit to
   the ramp-mid perch (151.7, 40.0, −111.5) — 0.7 u off the walk's own ramp-turn nav-2 spot —
   and overlook the fusebox: their drips ran −4.0 at a 60-tick cadence through the whole wire
   work (health 84 → 0 at ticks 7976-8396 on the early-arrival run; × 12 on the wedged run),
   and the perch is unpunishable from every west stance (the landing, the fusebox, the
   secure-room door, the ramp turn — every sight ray answers blocked while their bullets fly;
   the sister report's live check 3 asks exactly this). The fix: a d3 walk up the ramp mid to
   the perch's own level and a visible-only dy-5.0 fight there — **both died in one aim cycle
   each** (#16 at 7003, #15 at 7253) at health 84. TRIED AND RETIRED on the way: a plain
   pit-pair fight at the secure-room door (the unseen creep's 64-tick stand cap cannot open the
   ramp-mid line from there; the stood ticks ate −4 × 6, death 14084) and a ramp-south stand
   (the parapet eats every ray from below — the plain pick spun while #16's bursts landed).
4. **The d3_door_man gate widened 6.0 → 10.0.** On the early timeline the doorway burst pair
   (#5/#6, weapon 0x15) stood 5.8-9.6 u out at the fusebox arrival — past the 6.0 gate — and
   the unengaged pair's interval-4 −6.0 bursts later ate health 36 → 0 in 35 ticks at the room
   door. The widened gate engages them at the fusebox itself.
5. **The secure-room door shoot loop (d3).** The tops_up walk's walkway-pair pattern ("a player
   answers the two guards standing in his doorway with his fullest gun before walking on")
   brought to the E-door stance; measured firing ("the secure-room door guard 6: shooting
   drone 6 ... dist 5.1").
6. **The lower-level-east lip fight, BEFORE the visible platform stand.** On that leg the lip
   stands are skipped and the visible-only stand picked the nearly-dead #19 (12.5 u, visible)
   while the active killer #26 sat behind the half-open leaf unseen — Bond died mid-stand at
   26491 with the lip fight ordered after it (measured: the lip never reached). The lip fight
   (the plain pick + the unseen creep, the proven fusebox-doorway pattern) now runs first;
   d3-only.

Also corrected a stale comment in the same file: the rocket-blast note said "the
(int)-truncated linear falloff" — Explode_Propagate 0x17c518 has no int cast (0x17c608 is
`max.s f0,f0,f3`, EE funct 0x28 — the explode-falloff-int round's finding); the comment now
cites that.

## How it was verified

- Whole binary `walkreal_07000008_live` (`--test-threads=1`, NF_ISO on the slim on-foot ISO),
  final tree: **5 passed / 0 failed / 1 ignored** in 115.0 s — the topped-up whole-mission walk
  (byte-exact, passed unchanged), the Operative real-damage walk, the Agent real-damage walk,
  both ally tests; the 00 Agent sister is the ignore.
- Whole binary `interactables_07000008_live`: **15 passed / 0 failed / 0 ignored** in 11.8 s.
- The d3 sister itself was run end-to-end seven times this window (NF_DMG_TRACE on), each run
  against the then-current tree; the runs are deterministic given the input stream (run 1 and
  run 2's timelines matched byte for byte), so each code change was judged on its own run.
- The d2/d1 byte-exactness is proven by the whole-binary green — every change is behind
  `difficulty >= 3`.
- No `crates/*/src` change, so no other binary's code path moved; the merged-tree checks the
  sister report carried (01 7/0/1i, 11 3/0/1i, nf-game --lib 782/0/4) stand — this branch adds
  test-file-only deltas on top of that exact tree.
- Real input only: no teleports after the load, no damage/health/ammo writes, no top-ups; every
  tactic cites its run evidence ([L] with tick numbers) or addresses already pinned in the file.

## The fast suite

suite.py was started once more this window (setsid-detached) and was killed between tool calls
again — this sandbox kills background jobs (three more documented deaths last window; the
full four-crate build does not fit the killed-background model). The obligated binaries ran
directly from `target/release/deps` instead, per the established precedent; both green above.

## What's left

1. **The boarding exchange's economy** — the walk's last leak. The unseen leaf pairs (#20
   −6.0/−12.0/−6.0 at 2.6-7.4 u; the dead-by-then #26's −6.0 × 5; the hp-2 #19's −3.0 × 4 at
   12.5 u) ate health 72 → 0 across ticks 25710-26491 while the lip/visible stands ground on
   guards the aim gates could not land on (their bursts need no stand gate). The legs' return
   fire killed #23/#24/#26/#27 and wounded #19/#20 to hp 2-16 — winnable on paper, but the
   stand-grind at 12 u against leaf cover burns the 12-health margin. Levers: a pre-boarding
   stand spot with an OPEN line through the leaf gap (the leaves at (100.8-106, 28.2,
   −121.1..−122.2) face east — a stance east of x 107), or the close answer at their own leaf;
   the −12.0 #20 point-blank double-burst at 25894 (d 2.6) where the back-off's 4.0 ring and
   the platform edge bound the retreat.
2. **The Hind fight's dodge-window widening (30→60 u)** — the driver's lever 3, reachable once
   item 1 lands.
3. The fast suite on the final tree (the sandbox's background-kill problem; the next session
   with a working background runner).

## Live checks for the main agent

The sister report's live check 3 stands and now has sharper numbers: on this branch's runs the
pit pair's bullets REACH Bond from the ramp-mid perch (151.7, 40.0, −111.5) at d 8.2-9.7 while
the port's masked sight query answers BLOCKED from every one of Bond's stances (the landing
(146.35, 44.87, −115.94), the fusebox (153.7, 43.1, −120.5), the secure-room door (154.6, 42.5,
−121.0), the ramp turn (149.58, 39.56, −111.13)). A gdbwatch on ONE #16 sight ray from the
fusebox stance (address the masked query reads, e.g. the box-point walk at Collide_PickObj
0x1eae80) would settle whether the port over-blocks these lines or the game really does.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage panel is exposed to the agent).
