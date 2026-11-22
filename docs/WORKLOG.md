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

### walk-06-00agent-2 progress (2026-11-22, round 2)

Merged origin/master 7562f1e70 + origin/s/walk-06-00agent (three conflicts resolved: README and
the gadget census from master's newer side, the walk's ignore keeps both sweep texts + the round-1
ledger). The baseline reproduced exactly (death t7212, #11 at hp 2.4, the whole 150 pool). The
NF_DMG_TRACE run split the pool: 27.5 pre-fetch, 50 armour soaked by 9 un-answered mouth/balcony
hits, then the strip duel's 72.5. Round 2 landed four file-local levers (each judged on its own
real run, all gated difficulty>=3 && !tops_up()):

1. duel_target's fixed arm drops `enabled` — #11 stands dormant (en0) through the balcony phase;
   the enabled-gated fixed duel bailed 0 taps.
2. The blocked-line early-out (25 tapless taps): the first balcony tower duel burned 206 taps for
   ZERO damage and emptied the load; the 40-tick window then measured premature (bail at hp 9.0 /
   6.8); the tap-count window holds the peeks and still caps a blocked line at ~25 rounds.
3. The #24 finder 3.0 -> 6.0 (patrol (29.2, -36.0) is 3.9 off; his 56.0 in 12 shredded the
   finder-missed run).
4. Lever (a) ANSWERED at the strip approach: the fixed tower duel (the only proven line to #11 —
   0.57 rad; the balcony 0.26 and the sight edge 0.51 measured 0/206 and 0/10). #11 dies mid-duel;
   the pick re-locks #20 and grinds him to 0.2; #24 VANISHED at +43; #25 VANISHED at +51.

State: death t8566 at the SE corner (12.97, 41.83, -36.04) — 27 hits for 100.0, the killing blow
#23's ONE grenade (20.75 splash at d 27.4, weapon 0x24). Next levers: the #23 grenade answer, #5's
blocked line, the tower duel's own 32.5 cost. The d3 walk stays ignored; the ledger is in the
ignore message.
