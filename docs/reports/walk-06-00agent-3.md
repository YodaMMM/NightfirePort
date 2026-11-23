# walk-06-00agent-3 — the 06 castle-interior-to-the-party 00 Agent real-damage walk, round 3

Branch `s/walk-06-00agent-3`, from origin/master 7562f1e70 with origin/s/walk-06-00agent-2 merged
in first (clean — no conflicts; the merge brought the round-2 report, the round-2 WORKLOG entries
and the round-2 walk-test levers). **Still red** — the ignored walk is
`walk_the_castle_interior_to_the_party_real_damage_00agent` in `walkreal_07000006_live` — but the
pool ledger is materially corrected and three pool-trim levers are measured dead on their own real
runs. The tree ends comments-only: every walk stays byte-identical to round 2's merge.

## What the round did

1. **The baseline reproduced round 2 exactly** [L this branch's NF_DMG_TRACE run]: death t8556 at
   (12.97, 41.83, −36.04) — the SE corner — 27 hits for 100.0.
2. **The armour-soak dealers are NAMED for the first time.** The 9 zero-delta soak hits (they
   record no health drop — the bullet pain type absorbs into the armour word [M Player_HandlePain
   0x1902c8:84-104] — so the per-hit trace skips them) were measured with a temporary env-gated
   soak print in the shared damage_log (reverted before the finish; `tests/common/` ends the round
   untouched). Result [L this branch's runs]: **all 9 soaks (armour 50 → 0.5, ticks 6826-6914)
   are the ch-55 pair THEMSELVES at the strip approach** — #19 6 hits (d 5.1-5.9,
   (11.2..10.4, −35.5..−35.2)) and #20 3 hits (d 7.5-7.9) — during the fixed tower duel's #11
   grind. The mouth/balcony phases landed ZERO soak hits: **round 2's "un-answered mouth/balcony
   stream" ledger line was this strip pair, misattributed.**
3. **The final-tree grenade dealer is #22**, skin 0x500002f, at (38.4, 51.0, −37.6) d 27.1 [L the
   baseline death trace] — round 2's "#23 (38.5, −32.2)" citation was the lever-a5 run's timeline,
   not the final tree's. The corner answer's 8.0-wide east-floor finder catches #22 too (dz 5.4).
4. **Three pool-trim probes, each judged on its own real run, all MEASURED DEAD** (the ledger
   comments stand in the test file; the ignore message carries the full ledger):
   - **The pair answer before the tower duel** (a fixed duel on #20 at the proven (6.0, −34.0)
     stop): #11 WAKES when the pair fight runs first — 4 soaks + health hits at d 23.6, ticks
     6874-6925 — the tower grind's own pain-stagger is what suppressed his volley in the baseline
     (zero #11 hits all walk); #19 charged to d 4.0 and landed 10 health hits while the torso
     grind on #20 stalled at hp 0.2. Bond died t7198 at the strip approach.
   - **The strip stop 2 u east (8.0, −33.0)** for the ≤6 u head-box band (a head tap kills a
     20-hp guard [M NDrone2_HitDamage × DroneDamage_Head]): the pair's AI rolls CHARGE from the
     closer approach — both charged to d 3.7-3.8 during the nav and the point-blank fight
     murdered the walk (Bond died t7087). The (6.0, −34.0) posting distance is the proven one.
   - **The west BodyArmour #754 (−15.85, 43.16, −36.92) fetched from the strip approach** [D the
     pickup census]: the walk_nav lane routes south into the descent band and wedges at the
     tunnel arch (the 3000-tick give-up at (−1.45, −47.47), d 17.9 short), and the
     return-fire-off crossing of the descent shooters' lane cost 24.8 health (40.0 → 15.2) —
     net-negative; reverted.
5. **The strip phase's irreducible cost on this tree** is now measured: the pair's 9 soaks (the
   whole 50 armour) + #20's 6 health hits (32.5) once the armour died. The tower-first order is
   the proven strip optimum.

## What's left (the round-4 levers, in the ignore message)

- (a) The #754 fetch on a quieter lane (after the descent's fight_through, an east-first
  approach, or a return-fire-ON leg) — with the armour word up for the tail, #5's 2.75, #24's
  16.5 and the first grenade splash all soak, the corner duel answers #22 (the recent-hitter pick
  reads the explosion's owner), and the tail walks with health ~40.
- (b) #20's re-pick-grind 32.5 — the grind STALLS at hp 0.2 (the last drop never lands while the
  duel keeps tapping, ~100+ ticks locked on a not-dying guard): moving the lock on when the tap
  stream runs dry trims the exposure window.
- (c) The east-floor grenade answer's 8.0 finder catches #22 — moving that leg before the corner
  is a route change (the corner is the pass-through).

## How it was verified

- Every probe above ran on its own real run of the real input path (the load → the truck spawn →
  the walked route; no top-ups, no teleports, no health/ammo writes; NF_DMG_TRACE traces cited
  per run).
- Whole-binary runs on the final tree: **walkreal_07000006_live 8 passed / 0 failed / 1 ignored**
  (the ignored one is the ledgered 00 walk), **interactables_07000006_live 5 passed / 0 failed**.
- The fast suite (`tools/test/suite.py`) was started in the background in the first minutes (pid
  1791) and the sandbox killed it between tool calls, as in round 2 — this sandbox kills
  background jobs. Only the one test file changed (comments-only), so per the merge lessons the
  touched-binary runs above are the verification set. `tests/common/` and `crates/*/src/` are
  untouched this round.

## Usage

- Input/output tokens, context used, cost: not visible to me. The session's environment shows no
  token or cost figures; the WORKLOG entries for this task are the progress record.
