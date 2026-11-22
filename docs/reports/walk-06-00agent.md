# walk-06-00agent — the 06 castle-interior-to-the-party 00 Agent real-damage walk, round 1

Branch `s/walk-06-00agent`, from origin/master 2bd059afe, merged origin/master f0796ede8 at the
end (be3d52ee1). The ignored walk is `walk_the_castle_interior_to_the_party_real_damage_00agent`
in `walkreal_07000006_live`. **Still red — but the measured state moved from "dies at the strip
approach at ~20 health" to "dies ONE-TO-TWO TAPS short of clearing the strip fight", with the
whole lever set landed, gated, and ledgered.** The refreshed ledger and the next levers are in
the test's ignore message; the next message can resume from the branch.

## What the round started from

The ignore message's ledger listed the timing/stop space as measured dead (stream batch 1121b:
37 stop sets; walks-1121-06-16: 330 more), with the survivors' deaths at the strip approach
(5.97, 42.03, −32.77) at about 20 health to the tower post #11 and the ch-55 pair #19/#20, and
the court dealers' lines noted as clear world lines. The next untried levers per that ledger and
the sisters' proven patterns: the BodyArmour fetch (the 16 walk's lever-(a) shape) and the
fight/aim levers.

## Landed levers (each judged on its own real run on this branch)

1. **The guardroom's BodyArmour #755 fetch** (`difficulty >= 3 && !w.tops_up()`, file-local).
   #755 at (36.06, 40.67, −22.21) [D census], ~2 u east of the ammo boxes the d≥3 leg already
   visits; kind-armour sets the pool flat 50.0 and refuses only at pool ≥ 50 [M Pickup_Handler
   0x18e650:220-224]; the walk-over take is proven by
   `interact_07000006_live::walking_over_the_body_armour_takes_it`. The first fetch wedged on the
   pickup's own prop at the 0.6 arrive (d 1.75, grounded=false, the 3000-tick cap — the take
   itself fired from that band, armour 50.0 [L the run]); the landed fetch arrives at 1.8 with a
   bounded 90-tick nudge and takes it clean (~65 ticks, no wedge). **Proven the margin**: the
   dealer sums across the runs show ~150 incoming against a 122.5 pool (100 health + 50 armour)
   — without the armour the strip phase is unwinnable regardless of aim.
2. **The muzzle-flash duel** (`duel_the_muzzle_flashes`, same gate). The strict-ray stands cannot
   take the pair fairly — their muzzle rays open only inside their 12 u sight ([L runs 10-12 in
   the file's ledger]; my diagnostics: the stands bail UNSEEN from every floor spot while the
   guards shoot through the same geometry). The duel aims at the shooter's measured position,
   fires through the game's own symmetric hit model, and the run diagnostics measured the rest:
   - **Torso aim beyond 6 u, head boxes only ≤ 6 u**: head-aim measured 21 taps / 0 damage at
     d 26 (the game's own spread covers the 0.2 u head box past 6 u — the same window
     `walker.rs prefer_head` documents); the torso line lands every tap (2.2 a tap).
   - **The grind cadence is game-gated**: 8 ticks between taps; 4-tick requests changed nothing
     (the fire-state rate gates it).
   - **The 600-frame recent-hitter window misses between bursts** (every duel exited
     "no recent hitter" mid-fight) — widened to 2400.
   - **The target lock + the high-post swap**: without the lock the duel exited mid-grind at
     #20 hp 0.2; with the lock it grinds #20 20→13 and SWAPS to the tower #11 13→2 when his
     y-55.5 full-auto opens (the pick prefers the highest recent hitter; the swap fires when a
     y>44 shooter is active while a ground target is locked).
   - **The strip strict-ray stands measured NEGATIVE at 00** (9 landed vs 38 taken standing
     across 1500 ticks) — the sight-edge and strip-arrival `aimed_stand_the_mouth` calls are
     now skipped at 00 real damage only; the d2 walk keeps them (its gate is
     `!(difficulty >= 3 && !w.tops_up())`).
   - **Duel passes added** at the searchlight descent and the ledge legs (#24's solo-spot stand
     then VANISHED him at +99 after the duel left him at hp 2; #19 died to the return fire).
3. The corner #25 finder widened 4.0 → 6.0 (he patrols (30.9, −33.3)..(26.5, −34.2); at 4.0 the
   finder missed him and both the stand and the duel silently skipped).

## Measured state (the red that remains)

Baseline (the branch, before the levers): death t7109, #20 44.0 in 8 + #11 28.5 in 6, armour
0.0. After the levers: the strip phase costs ZERO health while the armour is up (50 → 6.0 across
the phases in one shape), #20 and #24 die, and the final shape is

- death t7212 at (5.97, 42.04, −32.77) — the ignore message's own original spot;
- #11 ground 13 → hp 2, #20 ground 20 → hp 13, pool 0 (the whole 150 incoming);
- the margin is ONE-TO-TWO TAPS: the grind is 2.2 a tap every 8 game-gated ticks; ~90 ticks of
  tower full-auto drain the whole pool while the duel grinds.

## What's left (the next levers, also in the ignore message)

(a) Answer the tower #11-class BEFORE the strip: a fixed-target duel at the sight-edge stop —
his lines are proven open from the plaza band (the mouth stand killed him at +43 in one measured
run). (b) Hunt the mouth/balcony phases' un-answered stream that ate ~45 armour before the strip
duel in the pre-swap shapes. (c) The corner #25 fixed-target duel and the tunnel/gallery phases
beyond (the walk has never reached them on the post-1121b stream).

## How it was verified

- Every lever above ran on its own real run of the real input path (the load → the truck spawn
  → the walked route; no top-ups, no teleports; the death positions/tick and dealer lists cited
  are from these runs).
- The sisters stay green and byte-identical: every lever is gated
  `difficulty >= 3 && !w.tops_up()` inside the 06 test file only; no shared helper under
  `crates/nf-view/tests/common/` changed (the duel is file-local).
- Whole-binary runs on the final tree (merged with origin/master): **walkreal_07000006_live
  8 passed / 0 failed / 1 ignored** (the ignored one is the ledgered 00 walk);
  **interactables_07000006_live 5 passed / 0 failed**.
- The fast suite (`tools/test/suite.py`) was started on the merged tree; its total line is
  appended below when it finished (the sandbox's 2-core window is the constraint).

## Live checks for the main agent

1. **The armour absorb rule** — walk Bond with armour 50 into a guard's round and record
   whether the hit reduces armour only (and by how much per hit vs the weapon) during the 06
   guardroom phase. The port's dealer sums assume armour soaks first and fully while pool > 0
   [M Pickup_Handler 0x18e650:220-224 sets the pool; the pain split itself I read from the
   sister walks' behaviour]; a PCSX2 watch on the health/armour words across one volley settles
   it.
2. **#11's line openings from the strip approach** — with Bond parked at (6.0, 42.0, −34.0) and
   #11 alive at (−0.4, 55.5, −51.0), record the trace of instruction accesses on #11's box for
   ~600 ticks: whether his torso line opens at all from that spot (my duel grinds him there —
   5 landed taps — but the mouth-stand path killed him in 43 ticks from the landing; the
   difference decides whether the fixed-target tower duel belongs at the sight edge or the
   landing).

## Usage

- Input/output tokens, context used, cost: not visible to me. The session's own environment
  shows no token or cost figures; the WORKLOG entry for this task is the progress record.
