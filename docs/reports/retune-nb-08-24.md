# retune-nb-08-24 — 08 whole-mission 00 Agent real-damage walk, round 24

Task: get `walk_the_whole_mission_on_real_input_real_damage_00agent` green again (round 24).
Branch: `s/retune-nb-08-24`, created from origin/master (7562f1e70) with s/retune-nb-08-23 merged
(commit 1e855b72e — the branch-point tree already contained the round-23 merge). Round 23's tree
was the starting point: death t29609 at the crouched anchor, the deepest walk of the campaign
(+8059 over its own baseline).

## What was tried (each lever judged on its own real run)

### 1. The draw-settle hypothesis — TRIED and DISPROVEN

The hypothesis (from the previous sandbox's uncommitted WIP): the rocket switch loop breaks on the
weapon-FIELD flip (f27454) while the draw animation still plays, and a fire press under the draw is
spent on the view-state gate [M Player_WeaponFiring 0x1a5460 — tick_fire's own `drawing` guard,
walker.rs]. Measured on the WIP run: the draw had ALREADY finished when the fight loop opened —
`[08 draw] switch_state 0 f27490`, 36 ticks after the flip [L the run log]. The settle loop would
have added ZERO input ticks; the round-23 fight-entry binder is not the draw. NOT shipped; the
`[08 draw]` evidence eprintln stays in the test (no sim tick, byte-exact).

### 2. The dive-window launch package — TRIED on one real run and DEAD by the depth criterion

The package (finished from the previous sandbox's WIP; one compile-order fix was needed — the
closing-dive forecast-skip arm moved into `ready`'s fore-conjunction because `dive_window` was
used before its definition):

- the per-tick dist history `machine_closing` (the dist trend over the last 10 ticks, `> 0.4 u`);
- the dive-window geometry: the launch line's crossing of the west band's x-plane must fall outside
  the band's measured z extent (-183..-176 with 1.0 u margins), with `cp[1] <= 16.0`, `l <= 50.0`;
- the closing-pass forecast-cast skip (a closing pass with the direct lane verified skips the
  extrapolated-cast gate);
- the closing aim tolerance 0.20 (vs the shipped 0.10);
- the weave hold-off below y 16.0.

Judged on its own real run (the whole mission from the level load, real input, real damage):

- the launch FIRED at fight t18 at d 42.8 — the dive geometry admitted what the shipped `l <= 20`
  cap refuses — and LANDED 100.4 at t211 (Hind 598 -> 497.6), the first Hind landing since rounds
  11/13 [L the run's `[08 rocket]` t18/t211 lines];
- but the landing RE-ROLLED the fight into the aggro class: a w62 salvo round crossed the west
  band's wall gap and detonated 0.59 u off Bond's dodge line at fight t881 — the dodge was live
  (dh 24) and its side step produced miss 0.59, inside the blast radius — the -13.00/-8.00 rows
  into health 21 killed Bond at t28369 total, 1240 ticks SHALLOWER than the shipped t29609
  [L the run];
- THE LESSON (new measurement, the round's real result): the stalemate's shield IS the machine's
  distance. Its w62 rounds detonate on the west band and pay the -2.00 x2 residue (the shipped
  stalemate grinds 8300 fight ticks on exactly that); a landing draws the machine through the gap
  and the full blast reaches the anchor.

Every behavioural arm was reverted; the tree's behaviour is byte-identical to the round-23 tree
(re-confirmed below). The definitions stay (for the `[08 gates]` dump's new `closing`/`band_x`/
`dive_win` fields, eprintln only — byte-exact), and the verdict comments live at the gates.

## Byte-exactness proof

- The reverted tree re-run on the 00agent walk: death t29609 (frame 29611) at (112.95, 13.09,
  -179.32), 53 hits for 77.0 + other 23.0, 14 return-fire fights — identical to the branch-point
  tree run immediately before it on the same sandbox [L both run logs].
- The whole `walkreal_07000008_live` binary on the final tree: **6 passed / 0 failed / 1 ignored**
  (186 s) — the round-23 signature exactly.

## The ledger

The ignore message on the 00agent test now carries the round-24 block: the draw disproof, the
dive-window verdict with its numbers, and the next levers, in priority order:

1. THE AGGRO ANSWER — the launch lever must ship WITH a salvo answer: the through-the-gap round
   needs a dodge miss >= ~6 (the wall-band detonations' own -2.00 x2 class) instead of 0.59 — a
   bigger/earlier side step on the w62 read, or hold the launch until the machine sits on the
   band's far side so its own salvos keep hitting the band.
2. THE ARMOUR ENTRY (08-23 item (3), still untried): fight entry armour was 20.0 on this deal (the
   ledge pickup landed 32 -> 50 pre-boarding; the boarding pair drained 30) — the salvo rows met
   HEALTH 21 because the fight's zaps had drained the 20; with armour 50 the blast meets armour.
3. THE SALVO CLOSE BAND (08-23 item (1), now measured sharper: the through-the-gap round at miss
   0.59).

## What was run

- `walk_the_whole_mission_on_real_input_real_damage_00agent` three times on this sandbox: the WIP
  package run (t28369), the branch-point baseline run (t29609), the reverted-tree run (t29609,
  byte-identical to the baseline) — each through the real input path from the level load, no
  top-ups, no damage/health changes, no guard-code changes.
- The whole `walkreal_07000008_live` binary on the final tree: 6/0/1.
- The fast runner (`tools/test/suite.py`) could not complete here: its build step ENOSPCs — the
  four crates carry ~1300 test targets (~1169 test files + 140 examples in nf-view alone, ~5.4 GB
  of binaries) and this sandbox's 9.9 GB disk is full at 100% (the image-layer caches cannot free
  space; overlayfs). This reproduces round 23's documented constraint (its report: the fast runner
  not run, the background runner killed between tool calls). The runner's RUN step was replicated
  over the binaries that are built (`scripts/run_bins.py`, same semantics: `--test-threads=1`,
  NF_SILENT=1, per-binary wall-clock limit, the same totals shape; results appended to
  `suite.out`). 165 built binaries were run directly (the seven walk/08 canaries, all 67 built
  nf-game unit binaries, 90 nf-view unit/census binaries and the `nf_view` lib binary), totalling
  **823 passed, 19 failed, 48 ignored**:
  - the seven walk/08 canaries (`walkreal_0700000c_eastroute_live`, `walkreal_0700001b_live`,
    `walkreal_probe0c_barrelrow_live`, `walkreal_probe0c_door6_live`,
    `walkreal_probe11_48_live`, `zipline_07000008_live`, `pickups08_live`):
    **28 passed, 0 failed, 2 ignored** — green;
  - the 19 failed test runs are 18 unique tests inside the ten `drive_*`/`driving_*` binaries,
    and every one is a disc-reading failure the slim ISO cannot satisfy (`/DRIVING/MIS01.VIV:
    too small for a BIGF header` — the onfoot ISO stubs the driving VIVs; the failure text is
    uniform across the family), not a regression: the branch's only change is the one test
    file those binaries do not compile;
  - the `nf_view` lib binary: **151 passed, 0 failed, 4 ignored** (green after its first 90 s
    cap TIMEOUT was re-run at the runner's own 300 s limit).
- Nothing under `crates/*/src/` changed; the changed file is used by exactly one test binary,
  which ran clean whole. `nf-audio`/`nf-formats` sources are untouched (their targets compiled
  during the build attempts; their tests are out of the touched-code scope).

## What's left (for round 25)

- The three ledgered levers above; the aggro answer is the gate — any launch lever without it
  re-rolls the fight into the salvo-kill class, as measured this round.
- The full four-crate suite on a sandbox with the disk for the whole build (or the runner's build
  step taught to skip examples — a shared-file change, not made here).
- Live checks for the main agent (not runnable in this sandbox): none required this round — the
  measurements came from the port's own run logs. A gdbwatch on the machine's aggro trigger (the
  damage-landing tick vs the w62 salvo spawn) would pin whether the aggro re-roll is
  damage-driven or launch-driven; that decides whether the band's-far-side hold can reuse the
  `machine_closing` trend already shipped as diagnostics.

## Usage

not visible to me

## ntfy failed

All three fallbacks failed (2026-11-23 02:51-02:53 UTC, this sandbox could not reach ntfy.sh):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "retune-nb-08-24: done ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds` (rc 28).
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'retune-nb-08-24: done ...', headers={'Title':'NightfirePort'}), timeout=20)"`
   → the python call exceeded its 30 s shell timeout and was killed (no response from ntfy.sh).
3. `wget -q -O- --post-data "retune-nb-08-24: done ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → rc 124 (timeout, no output).

