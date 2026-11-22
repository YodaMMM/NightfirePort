# retune-nb-0d-17 — round 17 of the 0d arrival-to-C5-door real-damage walks: the ignore levers judged on their own runs, the (b)+(c) floors committed, the walk passes the plate and dies in the photograph leg — still RED, the ledger refreshed

Branch `s/retune-nb-0d-17`, from origin/master 7562f1e70 with origin/s/retune-nb-0d-16 (181f7c942)
merged in first (0d-16 is not on master; 188 commits ahead, the accumulated 0d line). The merge
conflicts: README.md / sfx.rs / the gadget census take master's newer state (the credit line stays
dropped, the line_ledger field and the SkyBridge break stay), `walkreal_0700000d_live.rs` takes the
0d-16 side — verified safe: master's copy of that file is byte-identical to 6691acd8c (the point
0d-16 built on), so the 0d-16 side is master's content plus its own round-15/16 work; nothing was
lost. **The task walk `walk_arrival_to_the_c5_door_real_damage_agent` is still RED — the ignore
stays, refreshed with the round-17 ledger and the next levers.**

## The baseline re-measure on the merged tree

- **17-0, the Agent walk with its committed shape (3000/60 + 11000/45)** [L, the direct test
  binary]: DIED t20142 at (141.31, 5.11, −292.40), 34 hits for 70.6 + 29.4 other (falls), 0
  return-fire fights — 0d-16's 16-5 result reproduced exactly. The merged tree behaves as the
  round-16 ledger says.
- **17-1, the same run under `NF_DMG_TRACE`** [L]: THE NEW FACT. The pool dies at the LADDER
  TOP: #22 6.0 at t2694, #20 28.0 at t3874 (the climb's tail — the ladder-base act cannot
  converge on the crane's own structure), #15 28.0 at t4728 (inside the #17 act's 452-tick
  stand) = **62.0 of the run's 70.6 health damage**, then ZERO printed hits t4728..t19587. The
  annex armour grab (+50 armour at 26.8 health) holds; the yard receives pool ~19 and the
  landing/re-open phases starve it to 2.25 before the first act converges. Every later phase is
  pool-starved from t4728 on.

## The ignore message's levers, each judged on its own real run

- **(a) the NF_0D_HOP_HOLD grid re-roll — MEASURED DEAD.** 8/12/14/20/24 + the committed 13
  (17-0), six runs [L]: all red, deaths t19752-20322, the falls 27.3-32.2 across every hold.
  The girder slot fall is inherently ~30 at d2; no hold saves it.
- **(b) the 0d-15-proven d3 acts gated to d2 — MEASURED UNREACHABLE, then COMMITTED.** 17-3
  [L]: with the gates widened (d2+real-damage only) and the acts armed (NF_0D_NW_KILL=1 +
  NF_0D_NORTH_S1=1), the run is the EXACT baseline death t20142 and the acts never fire — the
  sites sit in the yard legs, which the death precedes (0d-16's 16-4 flag finding, now measured
  for the acts). After (c) preserved the pool the acts DO fire (17-7, below) and are committed
  ON for d2: at yard-south-1 both targets were already down — zero ticks, the run 14-f
  self-cancel law.
- **(c) the pool floor + the re-open cap re-tune — MEASURED, COMMITTED.** The committed 18.0
  post-act floor check sat BELOW the arrival pool (~19-20) and never fired (17-5: byte-identical
  to 17-0). The pre-act floor (the d3 sweep's own law) at 40: the landing sweep refuses every
  pass from a starved pool (17-6: the re-open walk arrived 8.6 health + 9.4 armour = pool 18.0,
  vs the committed 2.25) — but the re-open sweep ITSELF burned 18 → 0 in ~344 ticks (dead
  t19798). The re-open's own pre-act floor added (the same knob); it hands to the yard legs.
  Knob: `NF_0D_PREACT_FLOOR` (default 40; 0 = the committed post-act-only shape).

## The round's two trace-driven levers — both measured dead, knobs kept default OFF

- **The d2 ladder-top crouch** (the d1 law's lever): 17-2 [L] death t5574 at the ladder top,
  #15 landed 84.0 in 3 hits — the crouch's 0.45-lower eye drops the walk's OWN uphill act rays
  into the mid-band slab edge (the pair act's ray went 0.947-blocked standing→crouched; the
  standing act killed at +39, the crouched one stalled 400 + 203). The height law bites the act
  lines too. Knob `NF_0D_LADDER_CROUCH` (default OFF).
- **The pair-first reorder** (#15's 28.0 lands in the #17 act's stand; her own act kills at +39
  later): 17-4 [L] death t5750 — at THAT order position the pair act's ray is 0.947-blocked (the
  act burned 400 + 61) and the stall fed the snipers. The ray state is order-dependent. Knob
  `NF_0D_PAIR_FIRST` (default OFF).

## The committed shape (17-7 / 17-8b, byte-identical with no env knobs)

Both floors refuse (pool 19.1 / 18.0 preserved), the yard legs run at pool ~13-15, the (b) acts
self-cancel at yard-south-1, and the walk **passes the ST-3 plate — ch 0x1d fired, the plate
assert green — and dies t20234 at (137.69, 5.11, −313.38) in the photograph leg**: +92 ticks and
~21 u deeper than every previous round's best. Still red: the plate + photograph legs consumed
the pool 18.

**The wall is the pool equation**: 100 − 62 (the ladder top) − ~11 (the mid-run chips/falls) −
~29 (the falls) + 50 (the annex armour) = ~19 at the yard. The yard's leg exposure needs more.
No rule-legal lever found this round reclaims it (the hop grid dead, the ladder-top crouch dead,
the pair-first dead; the (b) acts land nothing new because their targets are already down).

## How it was verified

- 12 measured runs this round [L, the direct test binary, --test-threads=1, each its own roll]:
  17-0 the committed shape t20142 DIED (= 16-5); 17-1 the DMG_TRACE ledger; the hop grid 8/12/14/
  20/24 t20169/20133/20143/20322/19752 DIED; 17-2 the crouch t5574 DIED; 17-3 the (b) acts on the
  committed shape t20142 DIED (the acts never fired); 17-4 the pair-first t5750 DIED; 17-5 the
  floor-18 pre-act inert (byte-identical t20142); 17-6 the floor-40 pre-act t19798 DIED (the
  re-open burned 18); 17-7 the (b)+(c) combination t20234 DIED in the photograph leg (the plate
  assert green); 17-8b the committed default shape no-env = 17-7 exactly.
- **Whole binaries on the final tree**: `walkreal_0700000d_live` **2 passed / 1 failed / 2
  ignored** in 67.5 s (the plain walk + the d1 Operative walk GREEN — the d1/plain byte-identity
  holds; the failure is the 00agent batch-3 regression, pre-existing [L 16-0], signature t3995 /
  4 hits for 100.0 identical; the Agent walk ignored, its own run above).
- **The suite** (`tools/test/suite.py`): this sandbox kills background processes between tool
  calls (the build died twice, empty `suite.out` — the rounds-13..16 signature), and the
  four-crate build + run does not fit the remaining window of this session after the 12 measured
  runs and the report. The round's substitute, per the 0d-16 precedent: the touched binary whole
  (above) plus `interactables_0700000d_live` (below); the round's diff touches ONLY
  `crates/nf-view/tests/walkreal_0700000d_live.rs` (no `crates/*/src/`, no common helper, no
  other test file), so the nf-game / nf-formats / nf-audio crates' suites are unaffected by
  construction — 0d-16 measured them 2153 passed / 71 failed / 11 ignored on this same merged
  line three hours earlier and nothing this round touched can change them.
- Lesson-10 grep: the touched file is `crates/nf-view/tests/walkreal_0700000d_live.rs` only.
  The knobs/levers added are all gated `difficulty == 2 && real_damage` (or the d2 branch of
  the shared knob defaults); the d1/d3/plain runs are byte-identical (the whole-binary evidence:
  the plain + d1 green, the d3 failure signature unchanged from 16-0).
- No guard code changed, no damage/health constants changed, no top-ups on the walk phases, the
  crouch/stand presses are real input edges [M Player_HandleJump's crouch arms, 0x190630].
- Live checks for the main agent: none required (no visual work; the deaths are fully
  instrumented by the test's own damage log + NF_DMG_TRACE).

## What's left

- The Agent walk is still red: the ignore message carries the refreshed ledger and the next
  levers — (a) the deck_crouch extension to d2 (the d1 measured law: the ride starts 63.6 vs
  28.6 crouched; the d2 mid-run bill is only ~2-3, MARGINAL), (b) the photograph/trench legs'
  posture work (NF_0D_PHOTO_CROUCH exists at the photograph leg, d3-gated), (c) a HEAD act on
  #15 BEFORE the #17 act (the head lines clear where the chest rays die — the height law's own
  class, untried), (d) the climb's 28.0 from #20 needs a NEW stand (the base act's up-hill ray
  dies 0.040 on the crane [M the 512 comment]).
- The 00agent walk needs its own re-green on the batch-3 tree (pre-existing, unchanged by this
  round).
- The full four-crate suite still needs a bigger window/rootfs (the rounds-13..17 wall); the
  per-binary evidence above is this round's substitute.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures for
the session (no gauge, no billing panel, no API counters are exposed to the agent).
