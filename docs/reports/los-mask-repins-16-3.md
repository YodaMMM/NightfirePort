# los-mask-repins-16-3 — report

Branch `s/los-mask-repins-16-3`, base `origin/s/los-mask-repins-16-2` (76368fc6d; origin/master
784ba2d99 was already merged there — the merge was a no-op). Task: the 00 Agent d3 defeat walk
(`walk_the_chasers_defeat_completes_the_mission_on_real_damage_00agent`) green by the 16-2 ignore
message's levers, judged over several real runs; helpers gated to 07000016 + difficulty 3; real
input only, no top-ups, never lower damage or health.

## Done

**The ignore message's three levers are all implemented and measured on this tree.** The walk is
still red — the ledger and the next levers are in its `#[ignore = ...]` message (commit e517fe5db).
What each lever measured:

1. **Lever 1 — skip the ring east leg's own walk, cross to the ring 1 sight line.** Implemented
   as the task's first lever: at d3 the sprint arm's nav goes straight to the ring 1 target
   (6.0, 5.9, −78.5), and the crossing runs with the reactive fights OFF (restored after the leg —
   the measured fights-on crossing stalled at (10.5, 6.75, −78) from t900, a fight holding Bond
   mid-stretch while the quartet poured). The crossing now works: 201 ticks, no stalls, and the
   probe at the arrival (7.19, 6.75, −78.58) shows **#0 (d 2.2, hp 25), #2 (32.1), #3 (34.3),
   #4 (16.0), #5 (31.7) ALL `vis true` at level** — the punch-proven sight line is real. But Bond
   arrives with **2.0 health** (47.5 at the ramp top − 45.5 on the crossing) and dies at 968,
   15 ticks after the arrival.
2. **Lever 2 — a pre-ring stand from the ramp top's east edge.** Measured twice: the 916 death,
   back at the pre-sprint spot (14.51, 4.77, −82.64). A probe there shows EVERYTHING exposed
   (#0 6.7/25, #1 29.9 ground, #2 28.6/20, #3 36.1/10, #4 21.5/10, #5 35.8/20 — all vis true),
   but the answer's weapon preparation (the Delta 0x11, clip 3 spare 9, in hand → P2K) burns
   ~120 ticks at a 0.35/tick pour before the first shot flies. The stand window costs more than
   the kills save. Removed after the measurement.
3. **Lever 3 — the unseen-fights creep (hurting_first OFF at d3).** Applied (`hurting_first =
   false` in the d3 block): the run came back **byte-for-byte identical** — the fights never run
   in ANY d3 timeline (0 return-fire fights in every run; the quartet is never level+close enough
   for the pick), so the creep never engages.

**The new ledger facts** (all [L] runs on this tree, determinism byte-identical across two
sandboxes — the phase-split diag run):

- The lane costs 7.0 (93.0 health at the ramp foot, t446). **The ramp costs 45.5** (t446 → t752;
  three walk_to legs + their per-leg overhead, 306 ticks for 10.6 u under the Hard pour). The
  ring arm + crossing cost 52.5 more.
- A **single-nav-leg ramp+ring** (one fights-off `walk_nav_to` from the ramp foot to the sight
  line — 482 ticks, 6 web sub-legs, no stalls) shaves ~80 ticks of leg overhead and the pour total
  is UNCHANGED: the quartet lands **21 hits for 100.0 in every shape measured** (base 1002,
  crossing 1004, fights-off 968, pre-ring stand 916, single-nav 1033).
- The accuracy law has **no crouch factor** [M 0x179740 — the full factor list is
  TooClose/TargetFirstMoved/TargetFirstStopped/TargetMoving/difficulty/TimeToHit; a STANDING
  target loses the TargetMoving discount, so stands are strictly worse than walks].
- The sight line's y matters: the y 6.75 ring-floor spots see the whole quartet; the y 6.28 rest
  spots (the ring east target's arrival) see none of it — measured both ways.

## How it was verified

- `walkreal_07000016_live` whole: **17 passed, 0 failed, 1 ignored** (the parked d3) — release,
  `--test-threads=1`, `NF_ISO` the slim ISO, final tree. The 17 include the punch walk and both
  tops-up defeat walks: every change here is gated (`station_armour && difficulty == 3` or the
  `difficulty >= 3` block), and d1/d2/punch timelines are unchanged.
- `interactables_07000016_live` whole: **13 passed, 0 failed**.
- No `crates/*/src/` and no `crates/nf-view/tests/common/` file changed (the fights-off gate is a
  test-local `return_fire` toggle around two `walk_nav_to` calls), so no other binary's coverage
  is owed.
- The fast runner (`tools/test/suite.py`) cannot run in this sandbox: background processes die
  between tool calls (one attempt this session — the nohup process was gone at the next tool call;
  16-2 documented three). Per-binary coverage is the substitute, same as the base tasks recorded.

## What's left

- **00 Agent d3** — the one still-ignored walk. The budget, not the sight line, is the wall:
  the quartet's Hard pour (21 hits for 100.0) exceeds Bond's 100 health across lane + ramp + ring
  with zero completed kills, and every stand window measured costs more than it saves. The next
  levers are in the ignore message: (a) the P2K prepared during the cheap lane phase so no switch
  burns the ramp-top window; (b) the fights-off nav leg to the ring east END (14.4, 6.75, −78.7)
  with ONE bounded head-shot stand at the nearest exposed hp-10 dealer (one P2K head round each,
  17.5 = 3.5×5 [M NDrone2_HitDamage's part-5]); (c) then the sight line as measured; (d) [S] the
  +0x270 TimeToHit gate question for a live check.

## Live checks for the main agent

None this round — every fix is harness-side and carried by trace evidence ([L] runs on this
tree). The one [S] question worth a live look: whether the quartet's +0x270 TimeToHit sighting
timers [M 0x179740] explain why the ramp pour only starts around t446 — a gdbwatch trace of
+0x270 on drones #0/#2/#4/#5 across the lane and ramp legs would settle lever (d).

## Usage

The environment did not expose any token, credit or context-window figures for this session —
not visible to me. (No usage numbers were shown by the sandbox, the CLI, or the git host.)

## ntfy failed

All three fallbacks failed:
1. `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x` →
   `curl: (28) Connection timed out after 20002 milliseconds`
2. python urllib → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "..." https://ntfy.sh/nightfire-yoda-8k3x` → timed out after 30 s,
   killed.
