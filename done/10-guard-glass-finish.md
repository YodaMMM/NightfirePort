# Guard rounds through glass: finish

Merge `origin/master` into `origin/s/guard-mode-c4-glass`, push as `s/guard-glass-finish`. Read docs/reports/guard-mode-c4-glass.md. Code is proven. Fix the two Agent walks on walkreal_07000008_live test-side. Do not loosen a pin tagged [L]. Run the binary whole, then every walkreal binary you can, and quote results.

---
claimed-by: sandbox5 at 2026-11-06T00:12Z

---
## Report (done, 2026-11-06T01:45Z)

# guard-glass-finish — report

Branch `s/guard-glass-finish` = merge of `origin/master` (576909ed6) into
`origin/s/guard-mode-c4-glass` (4e4bcd668), merge commit ce2ee9489, pushed. Status:
**the Agent (d2) walk of `walkreal_07000008_live` is fixed and green; the 00 Agent (d3)
walk reaches the bottom terminal's Hind fight and dies on the documented rocket
self-blast** — the one thing left, with the killer list the previous report asked for.

## 1. What the merged tree actually does (the re-measurement)

The previous report's facts were measured pre-master-merge; the merge (11 commits,
incl. "Scared civilians raise the alarm") moved the timeline. Measured on this branch
(NF_RF_TRACE / NF_RF_GEOM runs, 2026-11-06):

- d2 died at tick 25036 at the stairwell foot (107.27, 28.37, −118.22), burst pair
  #19/#20 point-blank (d 0.6-0.7) at FULL hp — Bond never landed a round on either.
- The guard-mode-c4-glass "stairwell mouth" stands ran **eleven 61-tick stands on #19
  with `aim None` at every one** (clip 6 spare 27 — not dry) while the pair walked
  8.5 u from its spawn to point-blank, and the stairwell-top pair #23/#24 landed free
  hits into the `fighting=true` window.
- The blocker is the mouth's SOLID wall band, not glass: the eye ray stops at
  (105.25, 29.11, −118.14) inside the owner-22 runs (bounds e.g. lo (103.06, 27.25,
  −119.75) hi (105.50, 34.53, −112.64)) — the glass-pass sight trace clears nothing
  there. [NF_RF_GEOM dump]
- d3 died at tick 25116 on the west ledge (105.69, 27.85, −126.83): a failed #20 stand
  (he ducked behind the wall band mid-fight, aim None at t60) put #20 on the 240-tick
  cooldown and the pick BAILED the whole response (#19, fightable at 6 u, was never
  considered); the next stand fought the zero-damage pistol guard #23 at 14 u (his
  ledger total: 0.0) while #20 (59.0 in 7 hits, 4.4 u, visible) chewed health 68 → 0
  inside the `fighting=true` window.

## 2. The fixes (test-side only; no [L] pin loosened; no src/ changes)

- **08-local (`walkreal_07000008_live.rs`)**: `fight_close_guards` gained a VISIBLE-only
  sister (`fight_close_guards_visible`): the pick keeps only close guards a sight ray
  reaches, and a buried nearest guard opens no stand (a stand on an invisible guard is
  pure exposure). The "stairwell mouth" foot stand became visible-only; a NEW d2/d3-gated
  **"stairwell top" stand** clears #23/#24 before the climb (climb_stairs has no
  return-fire hook); the "platform mouth" stand fights the pair from the platform, where
  it IS visible — the d2 run killed #19 in 57 ticks and #20 in 223 from "the lower level
  east". Every pre-existing call site passes `false`, so the Operative/Agent-pinned
  timelines are byte-exact.
- **Opt-in shared-helper changes (`common/damage_log.rs`, lesson 7: flag + the only
  setter is the 08 d2/d3 wrapper)**:
  - `hurting_first` (+ wrapper `with_real_damage_glass_fights` = real damage +
    head-hunting + glass sight + hurting-first, set only by the 08 Agent sisters):
    `return_fire`'s pick orders the VISIBLE shooters by the damage they dealt Bond over
    the same 240-frame window (nearest on ties), cooled candidates are dropped instead of
    bailing the whole response, and a cooldown bends to active lethality — a guard
    visibly dealing > 15 in the window is switchable even inside his cooldown (the
    cooldown guards against INVISIBLE re-stands; visibility is re-tested at every pick).
  - `stand_and_fire` gained a mid-fight RETARGET under `hurting_first`: every 30 ticks
    the fight switches to the worst visible, off-cooldown shooter when his 240-frame
    damage beats the current target's by > 3.0 (the fight#23 trap: the pick is evaluated
    once, at the fight's start).
  - `exposed_impl` gained the opt-in GLASS_SIGHT sight rule — the sight ray passes
    intact glass exactly like the game's own round flights do [M Collide_Update
    0x1ea398:216-229; L weapon_data every gun's record bit 0x800 clear]. Kept for the
    through-pane fights (and it is the symmetric rule: a guard whose rounds pass the pane
    can be shot through it), though the mouth blocker proved solid.
  - `stand_and_fire` gained the env-gated (NF_RF_GEOM) geom-dump probe on a stand's first
    blind tick — the diagnostic return_fire already had, wired to the direct stands. No
    behaviour change with the env off.
- d3's launch-gate experiments were all measured and REVERTED (see §4); the fight ships
  with the previous session's gates plus the retarget machinery above.

## 3. Verification (each whole binary, quoted)

| binary | result |
| --- | --- |
| `walkreal_07000008_live` (whole) | **5 passed / 1 failed** — the d3 00agent Hind-fight self-blast (§4); d1 real-damage, the plain walk, the Agent (d2) walk and both ally tests pass |
| `walkreal_07000005_live` (whole) | 14 passed / 0 failed |
| `walkreal_07000011_live` (whole) | 3 passed / 1 ignored |
| `walkreal_07000012_live` (whole) | 3 passed / 0 failed |
| `walkreal_07000013_live` (whole) | 2 passed / 0 failed / 1 ignored — the ignored test is the pre-existing `#[ignore = "the scripted bot does not yet survive this fight on real damage; the plain walk passes"]` at `walkreal_07000013_live.rs:405`, unchanged by this branch |

One anomaly, reported verbatim: a single run of the 13 binary at ~01:40 (during the
`-j4 --no-run` build sweep — the binary it invoked was mid-build/stale) printed
`walk_the_warm_chain_to_the_front_entrance_and_out_real_damage ... FAILED` with the
death at tick 1831 at (5.40, −368.39, 32.45), 17 hits for 62.0, **other 38.0**, 7
return-fire fights, and "0 ignored" (the final build prints "1 ignored" — a compile-time
attribute cannot differ between two runs of the same sources, so that run did not test
this branch's code). It did not reproduce on the freshly rebuilt binary (twice, quoted
above). I flag it for honesty, not as an open defect.

The full suite did not fit: `cargo test --offline --release -j4 -p nf-view --no-run`
(970 test binaries) exceeded the 10-minute tool cap twice; the binaries listed above are
the ones it had finished. **No `src/` file was touched** — `nf-game`, `nf-formats` and
`nf-audio` are byte-identical to the merge state, so their lib suites are unchanged from
that state (the merge itself was green per master).

## 4. What's left: the d3 Hind-fight launch self-blast

d3 now runs the whole mission to the bottom terminal (its death moved from the mid-route
burst pair at tick 25116 to the Hind fight at ticks 27552-29996 across variants). The
killer list the previous report asked for:

- The explosion: **(113.6, 14.4, −177.6), d 3.4 from Bond, radius 8, −26 after the
  falloff** [NF_DMG_TRACE] — exactly the pane-8 frame band (the pane itself was shot out
  with the other 8; the mat-0x06 frame around it remains).
- The round flew ~0.13 rad off the gated corridor's bearing and died on that frame ~20
  ticks after launch; the corridor gate (12 u, eye + both gun stand-ins, the flight
  query) read CLEAR at the gate tick.
- The round's bearing is fixed at the AT-420's SPAWN tick (the fire script's release
  frame, ticks after the trigger [M gun.rs +0x44]), not at the gate tick — the free-aim
  cursor moves in between (the measured launches sat 0.13-0.23 rad off [L the diag runs;
  this session's −26 at 27552]).
- Four gate variants tried and measured this session: the yaw ±0.15 cone (death 28712 on
  the south frame band); the yaw ±0.30 × pitch ±0.15 cone (starves the fight — from
  inside the cabin every bearing is within ~3 u of a frame band and only the shot-out
  pane holes thread at all — 0 launches, the Hind's own blasts chip Bond down, death
  29996); a `|cross| < 0.05` cursor-magnitude gate (starves it the same way); a 20-tick
  cursor-stillness gate (opened t43 with the cursor frozen; the round STILL flew ~0.13
  rad off — death 27559). The spawn-tick bearing is not recoverable from the gate-tick
  cursor state.

Next session's candidates, in order: (a) hold the trigger through the launcher's release
frame so the gate tick IS the spawn tick (fire once, watch `player_bullets`, verify the
spawn delta, then gate on the release-frame cursor); (b) a live check of the game's own
AT-420 spawn-bearing law (see below); (c) fight from the terminal's open pad if the game
has walk-out floor (the reposition machinery is present but disabled at
`let _ = exit_ticks;`).

## Live checks for the main agent

- `--bond 111.0,13.5,-179.7,~0.75 yaw toward (130,15.5,-160), aiming` (level 08 bottom
  terminal, 00 Agent, the 9 cabin panes shot out): fire the AT-420 at the Hind on its
  north-east pass and record the ROUND's actual initial bearing vs the crosshair bearing
  at both the trigger tick and the spawn tick — the 0.13-0.23 rad offset's source (the
  missile-cam cursor law? the launcher's spawn-side spread?) settles §4 in one probe.
- `--bond 107.25,27.7,-119.5,~90,0` (the stairwell foot): confirm a player standing at
  the foot CANNOT see the platform west (the owner-22 wall band) — i.e. the port's sight
  geometry matches the game's (the d2 tactic now walks out and fights from the platform).

## Usage

- Explore/general subagents: none this session (the work was single-threaded measurement
  + fixes).
- My own session's input/output tokens, context used and cost: **not visible to me**.

## ntfy failed

All three fallbacks failed from this sandbox (the same wall the last three sessions hit;
git and copyparty hosts resolve, ntfy.sh does not):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "guard-glass-finish: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds` (exit 28).
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'guard-glass-finish: done', headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`.
3. `wget -q -O- --post-data "guard-glass-finish: done" https://ntfy.sh/nightfire-yoda-8k3x`
   → no output; the call hit the tool timeout and was killed.
