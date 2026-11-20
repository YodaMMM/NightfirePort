# retune-nb-14-6 — the 14 infiltration to the delta exit walks, round 6

Branch `s/retune-nb-14-6`, cut from origin/master ccd79b5be with `origin/s/retune-nb-14-5`
merged in first (it was not on master; the merge was clean — the round-4/5 reports, the two
ledgers and both WORKLOG entries carried over; WORKLOG appended only).

## What's done

**The Operative walk (the task's first priority) is GREEN again and un-ignored** —
`walk_the_infiltration_to_the_delta_exit_real_damage` (d1) passed on this tree:
walked out in 40206 ticks, stuck 0, falls 0, the exit hold drained and 07000015 loaded warm
(health 40.0 into the Rook kill) [L the 2026-11-20 `--nocapture` run]. The merged master had
re-rolled the wake legs' stream back to the retune-nb-14-3-era death; the round-5 ignore
message's candidate family plus two traced laws answered it. The lever, each piece judged on
its own real run:

- Run 0 (the traced re-measure, NF_DMG_TRACE/NF_RF_TRACE): the death is t36801 at
  (5.05,1.13,−23.31), 41 hits for 100.0, 23 fights — the SAME ledger the round-5 ignore
  message recorded; the two new master commits (the laser-watch drain, the pose pool) did not
  re-roll THIS stream.
- Run 1 (the named candidates only — `switch_on_first_blood` + `shoot_blockers`, d1
  wake-legs-gated): BYTE-IDENTICAL death. No retarget fired (the fight#28 stand's blocker is
  not a live ENABLED hostile) and no first-blood switch fired.
- Run 2 (the traced laws added): GREEN. The measured shape the two laws answer [L runs 0-1]:
  (a) the #28 response's stand sat `blocked true` its whole 300-tick cap 16 u west of the post
  (aim Some, err 0.000 from t60, clip 15 spare 12 untouched — zero taps) while #28 converged
  16 u → 4 u; a live-present drone that fails `enabled` held the line — with
  **fire_through_disabled** the stand FIRED and killed #28 in 52 ticks (the green run's
  fight#28 at t35382, killed true) — the game's own collide drops Enable(0) objects
  [M Collide_PickObj 0x1eae80] so a dormant guard never stops a round and must not stop the
  hold; (b) after the failed response the leg's return-fire gate could NOT open a fight for
  ~600 ticks — the plain pick returned the cooled #28 first (his 240-frame damage crossed the
  15.0 lethality override) and the response bailed on the cooldown check while #29 (off
  cooldown, hp 10, d 1.6) chewed unanswered — 35.0 of health gone between the responses; with
  **hurting_first** the fight#29 opened at node 30 (t35705, killed true). The finisher in the
  red runs was a GUARD GRENADE: a world kind-52 explosion at (13.3,0,−21.2) dealt BOTH guards
  83 (their ledger hp −81/−73 is that blast, not Bond's rounds) and clipped Bond 3.8 at 3.8
  health.
- All four flags sit in the d1 real-damage wake-legs branch only (`hurting_first`,
  `fire_through_disabled`, `shoot_blockers`, `switch_on_first_blood` — all pre-existing opt-in
  fields; no shared-helper code changed). The earlier legs, the d2 agent walk, the d3 walk and
  the tops-up walks are byte-identical by construction — and the full-binary run below proves
  the d2/plain walks stayed green.

**The 00agent sister stays `#[ignore]`d with the refreshed round-6 ledger.** The re-measure on
this tree hit the SAME death as round 5: t26979 at (−48.91,1.13,34.28), 34 hits for 100.0,
26 fights [L NF_DMG_TRACE run 2026-11-20] — the round-5 ledger stands verbatim; the two new
master commits did not re-roll this stream either. The ignore message now carries the round-6
re-measure line, the standing round-5 ledger (the door-face law, the Kiko hold's #20 with
clips burned dry into the corridor geometry — BOTH directions blocked), the NEXT levers
((1) a hold/stand spot whose ray to (−51.0,32.6) clears, measured with a probe pass first, or
the fights' creep law; (2) the Kiko-corridor armour fetch (−41.29,1.53,36.48)), and this
round's measured fight-law candidates (`fire_through_disabled`, `hurting_first`) for any stand
that goes silent.

## How it was verified

- **walkreal_07000014_live (whole binary, FINAL tree)** [L 2026-11-20]: **9 passed / 0 failed /
  1 ignored, 168.63 s** — the ignored one is the 00agent sister (its ledger above). Includes
  the un-ignored Operative walk, the plain/tops-up walk, the d2 agent walk, the vent-drop,
  grill, E-cascade and tripbomb tests.
- **interactables_07000014_live (whole binary)** [L 2026-11-20]: **10 passed / 0 failed
  (9.40 s)** — matches round 5's counts.
- No file under `crates/*/src/` changed this round (the lever is four pre-existing opt-in
  flags set inside this test file's own d1 branch), so no other test binary is affected; the
  two required binaries were run whole anyway.
- **The fast suite (`tools/test/suite.py`)**: started in the background in the first minutes
  as ordered, and the sandbox killed it between tool calls (suite.out stayed empty — the same
  failure rounds 4-5 recorded; its first attempt also died on the missing cargo PATH). The
  foreground re-run (with `--only walkreal_07000014_live,interactables_07000014_live` on the
  final tree) was killed by the tool-call timeout before its log line; the two required
  binaries' direct runs above are the same evidence. NO total line this round, same as
  rounds 4-5.
- Every fact tagged [L] comes from this window's runs on the real ISO with
  NF_DMG_TRACE/NF_RF_TRACE; code citations are [M] with the decomp address; no [S] items
  entered pass/fail; no test was weakened.

## Still red / what's left

- The 00agent walk (d3) — the round-6 ledger + the two levers are in its ignore message. The
  next round should START with the ray probe from the candidate hold spots (the corridor's
  west lane / the south leg's own lip) before touching the walk body, then the armour fetch.

## Usage

not visible to me — this environment exposes no token, credit, context-window or cost figures
for the session; nothing to copy.
