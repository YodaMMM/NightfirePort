# real-health-00agent-16-1b-2 — report (2026-11-07)

Branch `s/real-health-00agent-16-1b-2`, cut from origin/master 22d5836b. Round 2 of the 16 and
1b sisters (`walkreal_07000016_live` / `walkreal_0700001b_live`, each keeping two ignored
real-health walks: Agent d2 + 00 Agent d3), the first round ON the disc missile countdown
(c70343b3a "Equinox: the missiles launch staggered by their seconds"). **Status: the countdown
re-timed 1b's console phase and moved both sisters' deaths; this round measured the fresh kill
chain, tried the named levers against it, and recorded the measured law that stops them.
Both 1b walks remain red; the ignores now carry fresh tick-exact ledgers and the next lever.**

## What's done

**1b — the fresh baselines (master 22d5836b, [L] this round's runs):**

- d3 (`walk_arrival_to_drakes_defeat_on_real_health_00agent`): death **tick 7590 at
  (32.00,−5.36,−0.00), 30 hits for 100.0, ALL the wave ring** (skin 0x5000078, weapon 0x6a,
  −5.0 a hit Hard [M 0x1d3a80 case 0x700001b: Hard 0x3f800000]) — six shooters at d 8–13.2,
  y 1.6–6.2, first hit frame 6936. Consoles latched 5@4130 / 4@5288 / 3@6488; **k6 never
  latched** — the existing wave-defense triggers on tail consoles only (k 1/2/7), so the k6/k8
  hub waits are naked. The old tick-8426 Drake-fight ledger is DOWNSTREAM of this leg now.
- d2 (`..._on_real_health_agent`): death **tick 8628 at (20.36,−4.41,0.01), 43 hits for 100.0**
  (−3.5 Normal) — the k1/k2 wait, the wave floating **y 4.9–10.6 above** the station Bond
  hovers at −4.4. The tail defense fight fired and could not answer the above-deck band from
  the station.

**The named levers, tried and measured (all reverted; the file is master + fresh-ledger
ignores only):**

- The wave-defense extension (same-station fights, `waves_first`-gated so the Operative d1
  walk stays byte-identical): it SAVES the phase — one run latched all 8 consoles at health
  100 / armour 40 — but **the console phase is deadline-locked**. Every same-station fight
  repositions Bond, the wave's intercept shifts (it pursues his live position), and the UI
  shots slip past their one-tick branch reads: the k3 fight → `MissileDeploy
  [2,2,3,2,2,2,2,2]` (m3 launched, latch 7443 vs the ~7250 read); the k4/k6/k8 fights → k6
  latched 9170 / k8 9446 against the shared +5999 read → `MissileDeploy
  [2,2,2,2,2,3,2,3]` (m6+m8 launched). The UI latch margins are ~720 ticks each (measured
  undisturbed latches 5@4130/4@5288/3@6488) against a 6-shooter wave at ~12/s at d3 — a
  bolted-on fight cannot fit.
- The exposure re-weight (the round-2 named lever: the above-band term ×20 in the fight's
  cover score, the hide-and-snipe scan extended to +6): it worked as designed — the holds
  floated UP into the shelter band (Bond died at y 3.46 instead of −5.36) — but the fight-time
  cost dominates the deadline race, and the wave reaches the console stations before
  `hide_and_snipe` ever runs. Reverted with the rest.
- The Drake early window (the prio r−60 band widened to the fight's first 500 ticks at any
  range, `waves_first`-gated): implemented, never reached (the walk dies in the console
  phase). Reverted with the rest.
- Test-perf side-finding: a dead Bond inside `shoot_consoles` ground the loop for 480+ s
  (the d2 baseline run). Worth an alive-guard in a future round; not shipped this one.

**16 — both sisters re-measured on this tree** ([L], the countdown does not touch 07000016):

- d3: death **tick 1030** (was 1141 pre-countdown) at (12.12,0.93,−88.14), 20 hits for 100.0 —
  the ramp foot / corridor approach cluster.
- d2: death **tick 1638** at the same ramp-foot spot, 44 hits for 100.0 — the south-gallery
  law's spot (the gallery's first landed round tick 455). The round-2 report's four measured
  levers all stand; the levers named in the ignores (a cover-shaped gallery approach from the
  ring, an armour pickup, the live PCSX2 scout of the gallery's foot access) are unchanged and
  still the next work.

## How it was verified

- Every walk cited ran through the real input path (`cargo test --release -- --ignored
  --exact` on `NF_ISO`, `--test-threads=1` semantics, no top-ups, no state writes).
- The lever runs were `waves_first`-gated (the file's existing opt-in switch, set only by the
  d2/d3 walks), so the Operative d1 walks were untouched by construction; all changes were
  then reverted anyway — `git diff origin/master` on this branch is the WORKLOG, the report,
  and the four ignore messages, nothing else.
- Not run this window: the full 4-crate suite and the whole 16/1b binaries (the window went to
  the lever measurements); the branch's only code-bearing diff is the four `#[ignore]`
  strings, which no test reads.

## What's left

1. **1b (both sisters)**: a route-level re-plan of the console phase — the two shapes the
   ledgers point at: (a) a chosen intercept spot the wave is fought at BEFORE it reaches the
   console stations (the wave pursues Bond's live position, so an intercept that ENDS away
   from the stations keeps the undisturbed latch geometry), or (b) an undisturbed k3/k6/k8
   rush with the wave fought only on the x46/x20 crossing legs (the tail's ~1500–2200 margin
   absorbs fights; the k3/k6/k8 waits do not). The d2 shape additionally needs an answer to
   the above-deck band (y 4.9–10.6) that the station-level fight cannot see.
2. **16 (both sisters)**: unchanged from the round-2 report — the cover-shaped gallery
   approach from the ring, an armour pickup, or the main agent's live scout of the gallery's
   foot access (the round-2 report's "Live checks" item 1 stands).

## Live checks for the main agent

1. **0700001b, the k6/k8 hub waits' sight-lines**: a `--bond 32,-5.4,0` render at the d3 death
   spot looking up at the wave band (the measured shooters at y 1.6–6.2, d 8–13) — confirm
   whether ANY undisturbed hold near the hub (±5 u, y −6..+3, the fight's own scan range) is
   out of the wave's lines while the k6 UI (the console at the station y ~3.5) stays shootable.
   That decides lever (a) vs (b) for the next round.
2. **0700001b, the k1/k2 station's above-deck answer**: a `--bond 20,-4.4,0` render looking up
   — is there any body-fit point at y 3–6 near x20 from which the y 4.9–10.6 wave band is both
   visible (shootable) and cut (unseen)? If yes, the d2 tail fight needs a raised hold, not a
   station-level one.
3. **07000016, the gallery foot access** (carried from the round-2 report): a
   `--bond -16.3,5.9,-93.5` render looking south along the station toward z −102..−112, y 6.7
   — is there a walkway/stair from the station deck onto the gallery?

## Usage

not visible to me (no token, credit, context-window or cost figures are exposed anywhere in
this sandbox session; nothing in the environment reports them).

## ntfy

(see the note at the bottom — the delivery attempt is recorded there)
