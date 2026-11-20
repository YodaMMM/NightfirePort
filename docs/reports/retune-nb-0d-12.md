# retune-nb-0d-12 — round 12 of the 0d arrival-to-C5-door 00 Agent walk

Branch `s/retune-nb-0d-12`, from origin/master ccd79b5be with origin/s/retune-nb-0d-11 merged in first
(0d-11 was not on master; clean merge). The walk `walk_arrival_to_the_c5_door_real_damage_00agent`
stays RED — but this round **IDENTIFIED THE "LIVE WIRE"**: it is not a Hurt volume at all — it is the
oil-barrel pair the garrison's stray bullets detonate, with the exact blast law from the level data
and the port's own decompile-backed code. Five real runs this round; the refreshed ledger and the
round-13 lever are in the ignore message item (13).

## What was done

1. **The wire census** (the new `hurt_census_0700000d` probe test, ignored, run explicitly):
   `Level::parse` of 0700000d shows **NO 0xE4 hurt placements anywhere** — `sim.hurts` is empty and
   `folded_hurt` admits nothing. Every 0xE4 in the level's 50 maps: none. The "live wire / blast /
   hurt volume" damage the walklog prints is the **0xE1 Destroy oil barrels** [D the census]:
   `object_oilbarrel01` placements at **(143.30, 4.01, −298.41)** and **(144.22, 4.01, −298.34)**
   (a 0.92-u chain pair), each **health 20** (param 3), **blast radius 5.0** (param 5 500 ×0.01
   [M 0x12c45c-0x12c470]), **damage 40** (the type byte 4 ×10 [M 0x12bcd8-0x12bd0c]), ONE
   propagate per boom [M the EXPLODEINFO 0x3A latch], falloff `40 × (1 − gap/5)` with
   `gap = d − 1.0576` [M explode.rs falloff]. Other barrels: the −308/−310 cluster (out of reach
   of the landing/stand) and (130.71, 4.01, −304.07) radius 1.0 (harmless at our positions).
   The "~1300-tick cadence" of the old ledger was the strays' accumulation rate (11 × 1.8-chips
   to kill a barrel), not a timer.
   - The law's geometry: the landing spot (141.22, 5.06, −296.08) sits at d 3.29 (gap 2.23 → a
     boom costs 22.2); the escape corridor's closest approach ~4.4 dxz (→ ~13 mid-walk); the
     committed stand (142, −291.5) sits ON the radius edge (d 5.97 → gap 4.91 → 0.7 a boom).
2. **The round-12 lever (a) implemented and measured DEAD in all three stand variants** —
   `NF_0D_WIRE_STAND="x,z"` (the stand target, unset = the committed byte-identical walk; the
   walk walks the full distance at arrive 0.5 under the knob):
   - run 12-a (the far-west stand (139.5, −290.0), corridor 2): the RIDE died this roll (g18's
     27.6 hit t21924 — the ride lottery, pre-ride roll unrelated to the lever);
   - run 12-b (far-west stand (139.5, −290.0), corridor 1): the ride survived, the escape walk
     took **217 ticks and cost 18.0** (the armed pack's 0x12 chips on open ground: #1 d 23.8,
     #2 d 10.6) — the floor refused the sweep at 9.8, the run legs died t22738. The 5.5-u west
     walk costs more exposure than every boom it dodges.
   - run 12-d (the nudge stand (141.3, −292.5) + `NF_0D_PRETURN=1`): the committed arrive 1.2
     stopped the walk 1.2 u SHORT of the target — exactly into the barrel band — and the pair
     **BOOMED ON CAMERA: −4.40 + −0.00 from the world at t22689** [L], killing Bond mid-pass —
     while g2 STILL died at +58 (the exact fire law's third independent confirmation: cd(snap)
     converged 0.342/0.420 at +58, hp −22).
   - run 12-e (the nudge stand + arrive 0.5, the full distance): the walk took **241 ticks and
     cost 18.0**, arrived at gap 5.79 (boom cost 2.11), the floor refused the sweep at 9.8, the
     run legs died t22860.
   The verdict: **the escape walk's exposure (~0.106 health/tick while the pack's converged fire
   has the line, × 167–241 ticks = 18–23) dominates every boom cost (0.7–7.4, once)**. The
   committed stand's edge cost (≤ ~4.56 once — run 11-d's −4.56 now fully explained) is the
   cheapest of all measured shapes.
3. **The round-12 lever (b) implemented and measured NO-GAIN** — `NF_0D_PRETURN=1` turns Bond to
   the stand bearing first (the walker's fast 0.2-clamp `face_toward`): the walk still took
   167–241 ticks — the duration is the walker's nav steering + stuck mitigation, not the turn
   drag (run 2's 70-tick walk was that roll's clean steering, not a turn saving).
4. The fixes on the merged tree: the 0d-11 ignore message carried RAW `"` quotes inside the
   string (`"live wire / blast / hurt volume"`) — the merged tree did not compile; the quotes are
   now single-quoted (a syntax fix, no test weakened). The census probe is additive and ignored.

No guard code changed, no damage/health constants changed, no top-ups, no `crates/*/src/` change
(the knobs are test-file only, all default byte-identical). Lesson-10 grep: the touched file is the
0d test file itself; `grep -l face_toward crates/*/tests/*.rs` names the 0d file and the shared
walker (read-only use — no helper changed, so no other binary is affected by construction).

## How it was verified

- The runs (all [L], the direct test binary, --test-threads=1, --ignored, each its own roll):
  12-a t21924 (the ride, g18 27.6), 12-b t22738 (the west walk 18.0, the run legs), 12-c t22748
  (the pre-turn + committed stand, walk 18.0), 12-d t22689 (THE PAIR BOOM −4.40 ON CAMERA; g2
  down at +58), 12-e t22860 (the full-distance walk 18.0, boom cost 2.11 at gap 5.79).
- Whole binaries (knobs unset): **this sandbox OOMs the single-process whole-binary runs** — the
  container has 3 GB RAM and the walkreal process peaks ~2.9 GB anon (dmesg: `Out of memory:
  Killed process ... walkreal_070000 total-vm:3620100kB anon-rss:2944928kB`), the interactables
  binary the same (exit 137 at 2 tests). **Per-test evidence instead** (each test its own
  process — every level load freed between runs): `walkreal_0700000d_live`:
  `walk_arrival_to_the_c5_door` 1/0 (29.7 s), `walk_arrival_to_the_c5_door_real_damage` 1/0
  (33.9 s), `walk_arrival_to_the_c5_door_real_damage_agent` 1/0 (43.2 s), the census probe
  ignored — **3 passed / 0 failed / 1 ignored** in total; `interactables_0700000d_live`: **all 11
  tests 1/0 each** (2.6–5.1 s) — 11 passed / 0 failed in total. The knobs-unset tree is the
  merged 0d-11 tree plus additive test-file code; the round-11 whole-binary proof (3/0/1 +
  11/0) stands for the unchanged paths.
- The fast runner (`tools/test/suite.py`) could not run: the four-crate build needs ~5.5 GB of
  rootfs (2.8 GB free after the ISO + work + toolchain) and the single-process runs OOM at 3 GB
  RAM (the rounds-1-11 disk-wall signature, now plus the RAM wall). The per-test foreground runs
  above are the primary evidence.

## What's left

- The walk: still red — the binding constraint is now measured as **the escape walk's exposure
  cost** (18.0–23.4 across the rolls; run 2's zero was clean steering), not the barrels and not
  the turn drag. THE ROUND-13 LEVER (ignore message item (13)):
  (a) thread the escape walk with explicit mid-waypoints along the container's west face (the
  run-2-proven clean line) so the steering fights cannot stretch the walk — hold the 70-tick
  class EVERY roll (the walker's stuck mitigation is where the 167–241 ticks go);
  (b) shorten the 40-tick land_wait pinned INSIDE the barrel radius under the run variant (the
  landing burst's sight events have fired by then — verify the burst still resolves);
  (c) with the walk cost held ~0–3, the entry budget (~20–27) affords the sweep's pass 0 (the
  certain g2 kill at +58) AND the north-band kill (the round-11 lever (c), the #6-class
  back-fire at the C5-door corner) — then the run legs with the trench certain acts.
- Live checks for the main agent: the census [D] answers the round-11 fieldprobe request (the
  "hurt volume" is the barrel pair — bounds (143.30/144.22, −298.3/−298.4), radius 5.0, damage
  40); no live check is needed for the lever any more. OPTIONAL: on a fresh boot, stand Bond at
  (141.2, 5, −296) with the garrison firing and record the barrel-boom tick + damage (the
  stray-accumulation law — the old ledger's "~1300-tick cadence" claim).
- The suite: needs a bigger rootfs AND more RAM than this sandbox (3 GB) — the whole-binary
  single-process runs OOM. Next session should run `tools/test/suite.py` whole on the merged
  tree, or per-binary with the tests chunked per process.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures for
the session (no gauge, no billing panel, no API counters are exposed to the agent).
