# level-16-pit-squad-real-damage — report

Branch `s/level-16-pit-squad-real-damage` (from origin/master d699b536). Task:
`walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission_on_real_damage` was
`#[ignore]`d — the pit-squad phase (the 4-squad + pit shooter #8) reportedly killed Bond at
~48 health on real damage. Audit the squad's damage against the game, fix the port if it
over-deals, give the walker honest cover tactics, un-ignore.

## What the run showed first

The ignore's death does NOT reproduce on master d699b536's sim: the walked finale completes on
real health (measured before any change: end health 9.4, armour 43.2, lowest 9.4, 75 hits for
90.6 — the top dealers were the chaser's slot 39.6 in 36 hits and the door guard #8 8.0 in 4).
The ignore was written by the merged `level-16-chaser-real-damage` task (68f85f22); the master
work merged since (the drone.rs/bullet first-hit line among batch-1101y) already brought the
phase inside the game's numbers. I could not cheaply bisect WHICH commit flipped it (each
bisect step is a full relink in this disk-starved sandbox); what matters is pinned instead:

## The damage audit (pinned as assertions, not prose)

`finish_after_the_chaser` now audits every squad-phase hit (from the squad spawn frame to ch
0xbf) against the game's chain:

- The squad's four are skin 0x500001e → kind 0xb melee [M NDrone2_DefaultInit 0x14b300's
  switch]: their only deals are the punch — record 99 [D +0x0C = 50.0, chaser_melee_records
  pins the bytes] × 0.4 [M 0x1d4424's Operative row] = 20.0, fired only inside the 2.0 band
  [M DroneWeap_CloseCombatImpact 0x1789c0:26-40] — and their placement weapon's star record
  [D 0x12's +0x0C] ×(1|2 TooClose [M 0x139fac-0x139fc8])×0.4.
- The door guard #8 (skin 0x5000093, weapon 0x14 [D the pit-room placements]) deals its
  weapon record's damage ×(1|2)×0.4 [D +0x0C] — single digits.
- Armour absorbs before health [M Player_HandlePain 0x1902c8:84-104's type-0 arm; the port's
  `apply` mirrors it], so a recorded health delta never exceeds the raw chain.
- The assertion: no squad-phase health delta above 20.0 + ε — the game's strongest single deal
  in that room (the punch). An over-dealing port (the ignore's symptom) fails here. Measured:
  3 squad-phase hits in the real-damage run, all in-chain; the audit also covers the stray
  live route guard (#13) that still clips the window in the top-up run.

Conclusion: **the port deals no more than the game — no port fix needed.** The fire cadence
and accuracy ride the same state-driven path the chaser fight pins (the stars' cadence [D the
behavior streams]; under 3.0 u every drone round hits [M DoBulletAccuracy 0x179740:113-115]);
the walk's absorbed totals (75 hits/90.6 over the whole finale) stayed inside real health.

## The walker's tactics (all input-only, no pokes)

The honest set the walk uses: both blast-room shelters (the pulses), the range-keeping chaser
kill, and — added for the squad phase — the **melee band hold**: while any live clone (skin
0x500001e, death channel 0x17–0x1a) stands inside 3.0, Bond backpedals facing the nearest with
taps on the exposed aim (the chaser idiom), holding the punch band (2.0) and the
every-round-hits band (3.0) off; then stand-and-fire from the shelter's cover as before. In the
measured run no clone ever closed inside 3.0, so the hold is insurance that never fired — the
survival rests on the existing return-fire discipline.

## Verification (what ran, totals)

- `walkreal_07000016_live --test-threads=1`: **14 passed, 0 failed, 0 ignored** (72.5 s) — the
  finale now un-ignored and green, the top-up variant still green with the shared audit.
- `nf-game` (all 100 test binaries, `--test-threads=1`): **1033 passed, 0 failed, 9 ignored** —
  unchanged from master (no src change on this branch; the audit and the hold are test-only).
- The fast suite (`tools/test/suite.py`): started in the background per the driver — killed
  between tool calls again (this sandbox kills nohup'd children, 3× observed last session, 1×
  this one); a foreground pass exceeds the window (the master-delta relink + the ~13 min run
  phase). The binaries this task touched were run directly (above).

## Left

- One full fast-suite pass on the final tree for the next session (the cache is warm; the
  branch is one test-file commit ahead of master).

## Live checks for the main agent

Optional: in PCSX2, take a squad-phase punch on a fresh boot (stand inside 2.0 of a clone) and
read the health delta — it should be 20.0 (50.0 [D record 99] × the Operative 0.4 [M
0x1d4424]); that would upgrade the punch cap from [D]+[M] to [L]. Neither run landed a punch
(the walker keeps the band off), so the cap currently rests on the disc record + the DMod row.

## ntfy (previous task's appendix)

The level-4a-interactables-2 finish notification could not be sent (the sandbox died
mid-finish); all three fallbacks failed from this sandbox too — curl `Connection timed out
after 20001 milliseconds`, urllib `Network is unreachable`, wget killed at 30 s. The exact
errors are appended to `docs/reports/level-4a-interactables-2.md` on that branch (f803f25e);
this task's notification is attempted below per the driver's chain.

## Usage

not visible to me — this environment exposes no token, credit, cost or context-window figures
for the session (no usage header, no billing panel, no context meter is shown to me anywhere).
