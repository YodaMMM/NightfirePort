# retune-nb-08-11 — 08 whole-mission 00-Agent real-damage walk, round 11

Branch `s/retune-nb-08-11`. `origin/s/retune-nb-08-10` was not on master, so it was merged in
first (clean). Still red — the walk dies t25822 at (113.08, 13.47, −179.02) — but the round-10
priority lever (compress the tail so the fight loop opens inside the machine's dive) LANDED,
and the walk fired the first dive-window rocket launch of the whole retune.

## What was done

1. **The shoot-out instrumentation** (eprintln-only, no sim tick, the timeline byte-exact, the
   same pattern as round 10's diagnostics): the round-10 tail numbers split as follows [L the
   2026-11-20 baseline run, 21.0 s]: the drop block f24872-25309 (437 ticks), then the
   "887-tick pane shoot-out" is **scoop_drops 0 / arm_fullest 762 / switch_to_firearm 0 /
   reload_full 100 / the pane windows 25**. The wall was never the glass: arm_fullest's
   12-cycle weapon scan (12 × 61 ticks) cycled the whole 6-weapon end loadout
   [(20,32),(44,8),(53,2),(1,1),(2,5),(10,35)] — the scan never revisits `first` inside its
   12-press bound, so it always paid the full bound — to pick 0xa (35 rounds); reload_full
   waits a blind 90 ticks after one R press; the pane windows themselves die ONE pane to ONE
   converged round (25 ticks for both end panes, (110.36, −180.70) and (110.61, −181.63)).
2. **SHIPPED lever (1) — the arm_fullest direct pick, TAIL-ONLY.** The pick is read straight
   from the inventory (the same candidate law `switch_to_firearm` uses: owned firearm with
   clip or carried ammo > 0, fullest = max clip+carried) and the cycles run only when the pick
   is not already in hand. The first lever run took the fast path at the MID-GAME fights too
   (the east deck, the walkway pair) and the timeline WOBBLED — death t24113 at the trio 36.0,
   the boarding chunk drained HEALTH (was: armour) [L the 2026-11-20 first lever run, 14.1 s]:
   those fights' guards fire during the shipped cycles and their shapes are the shipped tree's
   own pace. The mid-game calls keep the shipped scan verbatim (`arm_fullest_at(.., false)`);
   only the pane shoot-out uses the direct pick. MEASURED [L the lever run]: the pick is the
   SAME 0xa (35 rounds) the shipped scan made, reached in 5 presses — the arm phase 396 ticks
   with the reload, was ~853.
3. **SHIPPED lever (2) — the stand-off east walk's wedge break.** The shipped walk pressed the
   pit's northeast corner from k80 to k220 parked at (113.08286, −179.23654) (the (113.3,
   −179.25) target 0.22 u short of reachable — the round-10 ledger's ~140 wedged ticks). The
   walk now breaks 30 ticks after its last progress (≥ 0.01 u): the park spot is BYTE-IDENTICAL
   (the k-stamps were frozen to 5 decimals), ~142 ticks freed — the drop block runs 437 → 295,
   ends f25167 [L the lever run].
4. **MEASURED — the dive is caught.** The whole tail drops ~1324 → ~841 ticks: the shoot-out
   ends f25713 (was f26196), the fight loop opens ~25714 — INSIDE the machine's measured dive
   (f25710-25830, y 12.84-13.93, x 69-93, z −211..−219 [L the round-10 approach dump]) — and
   **rocket 1 LAUNCHED at fight t36** from (113.05, 14.17, −179.05) at the machine (74.57,
   13.10, −210.84), dist 49.9, hp 598 [L the lever run]. The shipped fight loop opened f26248,
   ~450 ticks after the dive ended — this is the retune's first dive-window launch.
5. **THE DIVE-WINDOW FIGHT IS STRUCTURALLY DEADLY — the new wall, measured.** Death t25822 at
   (113.08, 13.47, −179.02) [L the lever run's damage rows]: (i) 10 dense −2.00 zaps t25702-
   25807 — the terminal hurt volume's dense window arms with the machine's approach (~f25666)
   and runs to ~f25810; the compressed loop OPENS inside it (the shipped loop opened after it,
   which is why the shipped anchor read nearly zap-free) — −20.0 of the 44.0 entry; (ii) the
   machine's reply missile detonated ON the west-band corner (110.4, 14.1, −181.0), d 3.4, for
   −26.00 at t25822 — MID-RIDE (the rocket was still flying, the machine's hp 598 untouched):
   the camera-locked ride cannot dodge. The reply's line from the dive position (74.57, −210.84)
   to the launch chest crosses the west band's x-plane (110.4 [D the measured stops]) at
   z −181.24, 0.64 south of the doorway gap's south edge (−178.25..−180.6 [D the tri dump]) —
   for ANY chest at the anchor's latitude the dive-phase reply self-detonates ~3.3 u from Bond.
6. **TRIED, no effect on this death:** the d3 dodge miss band widens 2.0 → 4.0 (the dodge gate
   AND the drive's salvo_close — the reply passed at miss ~3.4, outside the shipped 2.0 band;
   the d2 sister's band stays byte-exact). The death is tick-identical because the reply
   arrives mid-ride; the widened band stays shipped for the post-ride salvos.

## How it was verified

- Each lever judged on its own real run of the d3 00-Agent walk (NF_DMG_TRACE): the baseline
  21.0 s, the first lever run 14.1 s (the mid-game wobble), the tail-only lever run 17.0 s,
  the dodge-band run 17.4 s (tick-identical death) [L the four runs above].
- `walkreal_07000008_live` whole binary on the final tree: **6 passed / 0 failed / 1 ignored**
  in 108.67 s — the d1/d2 real-damage sisters, the plain walks and both ally tests stay green
  (the d1/d2 timelines are untouched by construction: the direct pick runs only at the pane
  call site, the wedge break and the dodge band are d3-gated, the diagnostics are eprintln
  only) [L the final-tree run].
- The final tree's only code delta vs the merged 08-10 state is
  `crates/nf-view/tests/walkreal_07000008_live.rs`. No `crates/*/src/`, no
  `crates/nf-view/tests/common/` change survives, so the affected-binary set is
  `walkreal_07000008_live` alone, and it ran whole.
- The full suite (tools/test/suite.py) did NOT fit the window, for environment reasons: the
  runner's background start (19:27 UTC, the first minutes) was killed by the sandbox between
  tool calls and left an empty `suite.out` (the same failure as rounds 8-10), and the
  foreground build of the ~200 nf-view test binaries does not fit the remaining window. What
  ran instead, on the final tree: the whole `walkreal_07000008_live` binary above (6 passed /
  0 failed / 1 ignored, 108.67 s), `nf-game --lib` **796 passed / 0 failed / 4 ignored**, and
  `nf-formats` — 97 passed across its six binaries with the disc_golden binary at 11 passed /
  3 failed, the 3 being exactly the documented slim-ISO trio (`bigf_header_fields_behave_as_
  documented`, `bigf_tables_match_the_golden_manifest`, `directory_tree_matches_the_golden_
  manifest` — the BIGF golden manifest tests that cannot run on the slim ISO). No
  `crates/*/src/` change this window, so those binaries are byte-identical in behavior to the
  merged 08-10 state.

## What's left (the next window's levers, in priority order)

1. **The reply-line geometry** — the launch chest clears the west doorway gap only at
   z ≥ −178.35 (≤ −175.83 keeps the crossing inside the gap; the gap is z −178.25..−180.6
   [D the tri dump]). The anchor (113.08, −179.02) is 0.67 short. Move the d3 anchor/stand
   spot north ~0.7-1.0 u so the dive-phase reply flies THROUGH the gap to the far band
   instead of detonating on the band corner d 3.4. The north strip (112.3, −177.6) measured
   WORSE on the OLD phase (the missiles chase it) — but the dive phase's geometry differs;
   judge on its own run.
2. **The dense-zap window** — the ~10-12 −2.00 rows are the fixed cost of standing at the rim
   during f25666-25810. A crouch-drop to the zap-free plate (feet 13.086 [L]) between launch
   windows sheds them (the crouched eye is blindfolded — stand only for the launches).
3. **A live check for the machine's reply cadence** — the reply arrived ~74 ticks after the
   launch: provoked by the launch, or its own schedule? That decides whether a delayed launch
   (after f25810) dodges the reply entirely or only moves it.

## Live checks for the main agent

- The machine's reply cadence: after Bond's rocket launch (any), when does the Hind spawn its
  w62 reply — immediately (provoked) or on a fixed phase? A gdbwatch on the missile spawn
  during a dive-window launch answers it.
- The reply's detonation point for a chest at z −178.0..−177.6 (the north strip, the dive
  phase): does the w62 clear the west doorway gap (z −178.25..−180.6 at x 110.4) and detonate
  beyond, or does the fuse still catch the band corner?

## Usage

not visible to me

## ntfy failed

(filled after the finish step — see the report's final state)
