# retune-nb-0d-19 — round 19 of the 0d arrival-to-C5-door real-damage walks: the composition's CHEST act landed on #4 (+201) and the walk reached t23067 (the deepest any round), the residual bill proved CONSERVED too, and the roof-armour lever is the next live path — still RED, the ledger refreshed

Branch `s/retune-nb-0d-19`, from origin/master 7562f1e70 with origin/s/retune-nb-0d-18 merged in
first (0d-18 is not on master; the merge was a clean carry — 0d-18's tip sat on the 0d-17 line).
**The task walk `walk_arrival_to_the_c5_door_real_damage_agent` is still RED — the ignore stays,
refreshed with the round-19 ledger and the next lever.**

## The lever this round

The round-18 ignore message's next untried lever: **(i) the D20-TOP + WARMOUR composition** — the
#4 pitch-pin law needed a different act class on his flat line, with the +50 yard-west armour
then absorbing the trench/return legs. The measured-dead levers were skipped as the ledger
marked them (S15_HEAD, D2_ROOF, WARMOUR's head act, the hop grid). New knobs, all gated
`difficulty == 2 && real_damage`, all default OFF (unset = byte-identical):

- `NF_0D_WARMOUR_ACT=1` — the exit-stand act swaps the HEAD class for the CHEST class
  (`kill_guard_at_present`, the plain `shoot_far_aim` aim_dy 0.3) on the corridor band.
- `NF_0D_WARMOUR_S1=1` — the DETOUR-FIRST order: no exit-stand act, the armour walk before the
  yard-south-1 leg, the #4 act at the yard-south-1 arrival.
- `NF_0D_WARMOUR_EAST=1/2` — chest acts on the yard-east armed pair (#7/#1); =2 walks the lane's
  first leg before firing.
- `NF_0D_ROOFARM=1` — the 18-12 scan's fourth armour grab (147.0, 21.76, −252.77) at the
  west-roof phase.

## The measured runs (each its own real roll [L, the direct test binary, --test-threads=1])

- **19-1, the composition D20_TOP + WARMOUR + ACT [L]**: the **CHEST act LANDED — #4 down at
  +201** (the head act's pitch-pin law was an exit-stand HEAD-aim measurement; the chest aim
  converges on his flat line). Death **t23067** at (122.70, 5.11, −274.36) mid-detour — **+1596
  past 18-3's t21471, the deepest any round has reached**. The bill: the conserved 62.0 tax
  (#19 28.0 + #15 28.0 + #22 6.0 — #20's window re-rolled onto #19, the conserved-tax law's own
  prediction) + the yard-EAST armed pair #7 (145.5, −280.2) 7.6-in-12 and #1 (148.1, −281.6)
  1.0-in-13 chewing the 23.75 armour then the 8.6 pool across the ~900-tick lane (armour 0.0 at
  death).
- **19-2, the DETOUR-FIRST order [L]**: death **t22235 at the SAME choke point x 122.70** — the
  armour at (107.75, −275.16) sits ~15 u beyond the budget death, unreachable in either order.
- **19-3, the east-pair acts at the exit stand with the 19-1 hit-time boxes [L]**: both
  self-cancelled — #7's box missed his exit-stand spot (141.30, 5.04, −281.75) by 4.2, #1 not
  present yet.
- **19-5, with the measured spot box [L]**: **#7 down at +148** — but **the residual bill is
  CONSERVED TOO (the round's finding)**: drone #0 (164.3, −282.6) re-attributed the 7.6 as
  6.7-in-14, the same 8.6 residual landed on the same spot, death t22180. Killing the lane's
  chipper just re-rolls the chips onto the next armed guard; the composition's missing piece is
  armour VOLUME in the yard, not kills.
- **19-6, the ROOF ARMOUR grab [L]**: measured DEAD as wired — `walk_nav_to` to (147.0, 21.76,
  −252.77) walked off the roof edge, the fall death t7314 at y −0.09. The roof armour remains
  the strongest lever (the west-roof phase is already at y 21 through its own committed
  `walk_to`/`Wp` legs), but it must ride the route's own waypoint class, not `walk_nav_to`.
- **19-0, the committed shape** (no env): the whole-binary run below carries it — the plain and
  d1 walks green, byte-identical with all knobs OFF.

## How it was verified

- The whole `walkreal_0700000d_live` binary on the final tree: **2 passed / 1 failed / 2
  ignored** in 68 s — the plain walk + the d1 Operative walk GREEN (the byte-identity holds),
  the 00agent failure the pre-existing batch-3 signature (t3995, 4 hits for 100.0, identical to
  [L 16-0]), the Agent walk ignored (its own runs above).
- Lesson-10 grep: the diff touches ONLY `crates/nf-view/tests/walkreal_0700000d_live.rs` (no
  `crates/*/src/`, no common helper, no other test file) — the nf-game / nf-formats / nf-audio
  suites are unaffected by construction (0d-16 measured them 2153 passed / 71 failed / 11
  ignored on this same merged line). Lesson-7: no common-helper change, so no cross-level
  walkreal re-run is required by the rule; the whole 0d binary above ran anyway.
- The fast suite (`tools/test/suite.py`): started in the background in the first minutes per the
  task; this sandbox kills background processes between tool calls (the rounds-13..18 signature,
  re-measured this round at ~05:57Z — the process was gone at the next check, `suite.out`
  empty). The four-crate build + run did not fit the remaining window. The substitute, per the
  0d-16/17/18 precedent: the touched binary whole (above); the round's diff cannot change the
  other three crates' suites.
- No guard code changed, no damage/health constants changed, no top-ups: every new path is a
  REAL-input act (the walk's own act machinery) or a REAL level-pickup walk (the touch path),
  and every knob defaults OFF so the other walks and replays stay byte-identical (the whole
  binary's plain + d1 greens are the proof).
- Live checks for the main agent: none required (no visual work; the deaths are fully
  instrumented by the test's own damage log).

## What's left

- The Agent walk is still red. The refreshed ignore message carries the round-19 ledger and the
  next lever: **the ROOF ARMOUR via the committed route's own waypoint class** — a `Wp::new`
  leg inserted between the west-roof acts (the route is already on the roof through this
  phase; `walk_nav_to` fell off the edge [L 19-6]) grabs the +50 BEFORE the yard: the yard
  armour ~73.75, the conserved residual bill (~32 raw across the lane/trench legs) absorbs into
  it, the 8.6 pool survives — composed with the proven acts (D20_TOP + WARMOUR_ACT +
  WARMOUR_EAST=1: #4 +201 [L 19-1], #7 +148 [L 19-5]). The yard-west armour then becomes
  redundant.
- The 00agent walk still needs its own re-green on the batch-3 tree (pre-existing, unchanged by
  this round).
- The full four-crate suite still needs a bigger window/rootfs (the rounds-13..19 wall); the
  per-binary evidence above is this round's substitute.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures for
the session (no gauge, no billing panel, no API counters are exposed to the agent).
