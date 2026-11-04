# gas-rng-draws-5 - finish the gas RNG draw landing

Continue from origin/s/gas-rng-draws-4; merge origin/master first. Read docs/reports/gas-rng-draws-4.md. Land the remaining list below and finish with a full suite pass, reporting the total line.

## Not done (the next session's list, in order)

1. The walk re-routes: 01's three remaining reds (ledgers above), 02 (to_the_exit — not run
   this session, the walk alone costs ~6 min), 03 (4/2+1i — the friendly death + the east-lane
   wedge), 14 (not run this session — the corridor-door chain at (30.3, 35.4)), 16 (12/2+2i —
   the punch pin + the defeat walk), 1b (15/2+2i — Drake's defeat + one more).
2. fix_finale's 16 shelter pulse — fresh ledger this session: Bond died tick 6493 at
   (-14.50,-22.00,-170.18) BEFORE pulse 2's window, to a 13.3 "the world" hit (a live wire or
   the pit's lethal-floor edge) plus 29.6 of chaser plinking accumulated on the walk back —
   the chaser-fight block's loose timing (+2600 gate, grenade retries) and the INTO_THE_SHELTER
   route need a phase-accurate re-derivation.
3. Every walkreal_*_live whole (this session ran only the six task levels + 01), then the full
   suite via tools/test/chunk_suite.py (the reports measure its run phase at ~3 h on this box —
   start it early).

## What was run (direct binary runs, --test-threads=1, slim ISO)

- `enemy_accuracy_live` 5/0+4i. `fix_blast_scripts_live` 3/0+1i. `fix_finale_live` 3/1.
- `walkreal_07000001_live` 2/3. `walkreal_07000003_live` 4/2+1i. `walkreal_07000016_live`
  12/2+2i. `walkreal_0700001b_live` 15/2+2i.
- `walkreal_07000002_live` and `walkreal_07000014_live`: built, not run (the window closed
  mid-sweep; 02's walks alone cost ~6 min).
- nf-view lib test builds clean (an earlier ALSA-1.2.16 glibc-2.43 sqrtf link failure was the
  sandbox's user-prefix being too new; 1.2.11 fixed it — no repo change involved).

## Live checks for the main agent

1. 01's far pair: in PCSX2 at 00 Agent, gdbwatch the two 0x500003a guards at ≈(-7.9, 6.3,
   -0.4) — WHEN do they start (the switch channel) relative to the hall AND? The port needs
   their enable tick to place the pre-clear.
