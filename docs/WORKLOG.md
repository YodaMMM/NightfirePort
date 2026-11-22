# WORKLOG

## walk-06-00agent (2026-11-22, s/walk-06-00agent)

Task: branch s/walk-06-00agent from origin/master 2bd059afe. Make
walk_the_castle_interior_to_the_party_real_damage_00agent in walkreal_07000006_live green
on real input; read the ignore-message ledger, skip measured-dead levers, take the next
untried one; keep the Operative and Agent sisters green; helpers gated to level+difficulty.
Remove the ignore when green; otherwise push the refreshed ledger. Progress appended below.

### walk-06-00agent progress (2026-11-22, round 1)

Branch from origin/master 2bd059afe. The ignore's stop/timing ledger taken as dead; round 1 landed
two levers, both file-local and gated `difficulty >= 3 && !w.tops_up()` (the d1/d2 sisters'
timelines untouched — the gates verified by their unchanged paths):

1. **The guardroom's BodyArmour #755 fetch** (36.06, 40.67, −22.21 [D census]) — kind-armour pool
   flat 50.0 [M Pickup_Handler 0x18e650:220-224]; the walk-over take proven by
   interact_07000006_live::walking_over_the_body_armour_takes_it. The 0.6-arrive wedged on the
   pickup's own prop (d 1.75, 3000-tick cap) — refetched at arrive 1.8 with a bounded nudge; the
   take fires from the band. PROVEN the margin: the dealer sums show 150 incoming against a
   122.5 pool without it.
2. **The muzzle-flash duel** (`duel_the_muzzle_flashes`): the strict-ray stands cannot take the
   pair fairly (their rays open only inside the 12 u sight [L runs 10-12]), so the duel aims at
   the measured position, fires through the game's symmetric hit model, locks the target until
   he drops, and swaps to a y>44 shooter mid-fight. Measured facts landed as comments: the torso
   grind 2.2 a tap every 8 ticks (game fire-state gated), head boxes carry only ≤6 u (21 head
   taps, 0 damage at d 26 — the walker's own 6 u spread note), the strip strict-ray stands
   measured NEGATIVE at 00 (9 landed vs 38 taken standing — skipped there now), #24's solo-spot
   stand VANISHED him at +99 after the duel left him at hp 2. The duel grinds #20 20→13 and
   swaps to the tower #11 13→2.

State: Bond dies t7212 at (5.97, 42.04, −32.77) ONE-TO-TWO TAPS short — pool 0 against
#11 40.0 + #20 32.5 + the court's 27.5. The refreshed ledger + the next levers (the fixed-target
tower duel at the sight edge; the mouth-phase stream hunt; the corner #25 duel + the tunnel
phases) are in the test's ignore message. The d3 walk stays ignored.

## walk-06-00agent-2 (2026-11-22, s/walk-06-00agent-2)

Branch from origin/master 7562f1e70; origin/s/walk-06-00agent merged in first (three conflicts:
README + the gadget census taken from master's newer side, the walk's ignore message keeps both
the master sweep text and the round-1 ledger). Goal: round 2 of the 06 castle-interior→party 00
Agent real-damage walk — take the next untried levers from the round-1 ledger, judge each on its
own real run, keep the Operative and Agent sisters green, helpers gated to level+difficulty.
Progress appended below.
