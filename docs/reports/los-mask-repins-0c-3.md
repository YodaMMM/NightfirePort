# los-mask-repins-0c-3 — report

Branch `s/los-mask-repins-0c-3` = origin/master `0ee8478fd` (the los-mask-repins-0c-2 merge) +
work commits. Task: the 0700000c handoff walk's real-damage sister green with a real margin,
un-ignored.

## What's done

**`walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff_real_damage`
is green and un-ignored.** The whole binary runs 4 passed / 0 failed / 1 ignored (the Agent
sister keeps its own ledger — untouched, as the 0c-2 report scoped).

**The 0c-2 ledger's post-ride lever closed it — and the lever alone was not enough; the run
found the deeper cause.** Measured on this branch:

- The 0c-2 ledger reproduced exactly: the #282 ride lands at the real-damage tree's own shifted
  spot (4.9831, −223.2948) [L this branch's run] and "the ledge to the arc" fell at +114 ticks
  to (8.99, 2.01, −222.09) — walk_to's arrival check is dxz-only, so it read "arrived" MID-FALL
  and the next leg wedged on the yard floor for its 6000-tick cap [L the run].
- With the ledger's stops inserted, the sister fell EARLIER: (6.26, 1.13, −221.83) — 0.84 u
  north of the straight line at the same x. A temporary hand-steer diagnostic (removed) from
  the same landing walked to the same target clean at (7.117, 12.718, −222.412) [L the rd-diag],
  and `NF_RF_TRACE` showed no fight in the window [L] — the divergence was the walker's own
  **stale sidestep**: the stuck ladder's nudge counter survives the route's direct `step_bond`
  stretches (the wire grab and the rides never decrement it) and strafes the first post-ride
  walk_to leg sideways off the thin band — a stale LEFT nudge facing east is a NORTH strafe.
  The same bug family the los-mask-repins-12-2 walk measured on 07000012 (that fix,
  `sidestep_decays`, lives on its own branch, unmerged; this task needed the ride-specific
  reset instead).

**The fixes (all opt-in; real input only; no health, armour, ammo or position writes; no
top-ups):**

1. `crates/nf-view/tests/common/walker_b2.rs` — `pub fn reset_sidestep()` (additive, one
   assignment; default behaviour unchanged; the sister's scope is the only caller).
2. `crates/nf-view/tests/common/walkreal0c.rs` — the `STRIP_RD_POST_RIDE` thread-local scope +
   `with_strip_rd_post_ride` (set ONLY by the real_damage sister; off = master behaviour for
   every other caller), checked POST-ride (after `strip_fights_resume()`) so nothing can
   re-time the #282 grab: the reset, then the 0c-2 ledger's mid-ledge stops — the plain run's
   own hold spot (7.1, −222.42) and the arc west end's midpoint (10.5, −220.25) — so every
   thin-floor arrival circle is met at walking speed.
3. `crates/nf-view/tests/walkreal_0700000c_live.rs` — the sister un-ignored, wrapped in the
   new scope, the mechanism in its doc comment.

**The green run:** the strip crossing runs the 0c-2 no-stand choreography verbatim (the moving
west-lane leg, the 12-tick settle, the mid-band stop, the diagonals, the #282 ride); the stops
held at (6.64, 12.77, −222.56) → (7.09, 12.74, −222.39) → (9.76, 12.45, −220.02) →
(11.84, 12.50, −218.51); wire #283 crossed, landed (36.71, 12.15, −208.66); walked out in
18237 ticks, **stuck 0, falls 0**; the 0d handoff lands at **health 84** (lowest 83.8 — a real
margin, nothing lowered). The fights are the walker's own return fire on the real pipeline
(20+ fights won; the level's own medkit restored 75 → 100 on the real pickup path mid-route —
that is the game's own offering, not a write).

## How it was verified

Per-binary, whole, `--test-threads=1`, release, this branch's tree. The changed helpers are
`walkreal0c.rs` and `walker_b2.rs`, so every binary that includes either ran whole:

| binary | result |
|---|---|
| walkreal_0700000c_live (whole) | **4 / 0 / 1 ignored** (the sister green; the Agent ledger stands) |
| walkreal_0700000c_eastroute_live | 1 / 0 |
| interact_0700000c_live | 10 / 0 |
| interactables_0700000c_live | 7 / 0 |
| walkreal_0700000c_barrelrow_live | 1 / 0 |
| comms_chatter_0c_live | 1 / 0 |
| guard1_rounds_0700000c_live | 2 / 0 |
| upward_shot_0700000c_live | 2 / 0 |
| walkreal_0700000d_live (the handoff consumer) | 1 / 0 / 2 ignored |
| walkreal_07000009 / 0a / 0b / 11 / 11_route / 12 / 13 / 14 | 9/0, 13/0/2i, 9/0, 3/0/1i, 1/0, 3/0/1i, 3/0/1i, 7/0/3i |
| interactables_07000003 / 06 / 07 / 0a / 0b / 0d / 12 / 13 / 14 | 9/0, 5/0, 4/0, 14/0, 14/0, 11/0, 12/0, 10/0, 10/0 |
| nf-game --lib + nf-formats + nf-audio | 901 passed / 0 failed |

Every ignore that remains is a pre-existing ledger from an earlier task (the Agent sister
here; walkreal_07000012's Operative sister is ignored on THIS tree because the 12-2 un-ignore
lives on its own branch — no regression, the binary's three active tests are green).

`tools/test/suite.py` did not run: started twice, it died between tool calls both times
(`suite.out` 0 bytes, no process survived the tool-call gap) — the same constraint the
los-mask-repins-12, -12-2 and -0c-2 sessions recorded. The per-binary coverage above is the
substitute.

## What's left

- Nothing on this task's item. The remaining 0c ledger is the Agent sister's crossfire cadence
  (its own ignore message carries the levers). The stale-sidestep-carry mechanism is now
  measured on TWO levels (0700000c here, 07000012 on s/los-mask-repins-12-2); a general fix
  (`sidestep_decays` + a ride/grab-scope reset) could land as one walker change when both
  branches merge.
- The 0d legs of the sister end at the 0d arrival plate (objective 3 complete, health 83.8) —
  the same end the plain walk pins.

## Live checks for the main agent

None new — this task's fix is in the test walker and the test route; the game-facing facts it
rests on were live-checked in the earlier rounds (the 0c-2 report's three checks stand). If a
render pass is cheap: the real-damage run's Bond at the #282 ride landing
(≈ 4.98, 13.16, −223.29) should stand on the thin ledge with the drop visible north-east, and
at the mid-ledge stop (≈ 7.1, 12.7, −222.4) facing east toward the arc — both are the spots
the walk asserts indirectly.

## Usage

not visible to me — this environment exposes no token, credit, cost or context-window figures
for the session, and I will not estimate them.

## ntfy failed

All three fallbacks failed; the sandbox has no outbound route to ntfy.sh this session (the
same outcome the los-mask-repins-12, -12-2 and -0c-2 sessions recorded):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "los-mask-repins-0c-3: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds` (exit 28)
2. `python3 -c "… urlopen(Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'los-mask-repins-0c-3: done', headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "los-mask-repins-0c-3: done" https://ntfy.sh/nightfire-yoda-8k3x`
   → exit 124 (timed out, no output)
