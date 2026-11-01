## Your task: `level-0a-ceiling-gun-route-2`, 0a: finish the crouch-under-#607 real-damage route

Continue `origin/s/level-0a-ceiling-gun-route` (merge `origin/master` first; read its report — blocked/10-level-0a-ceiling-gun-route.md on the queue branch). It found beam #607's band at 1.49-1.76 [D] is clear of a crouched body (1.3 [M Player_Collision 0x194df0 case 3]) so a crouched player never raises ch 0x1F and ceiling gun #640 stays dormant, while the low beams (#621 at y 0.26) are dodge-only. Finish: the outbound nav via and the fatal band legs so `walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch_real_damage` completes on real health crouching under #607 (un-ignore it), pin the laser band rule (crouched passes #607, standing raises 0x1F; the low beam fires either way), and run every *0700000a* binary and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-ceiling-gun-route-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-01T13:17Z

---
## Report (blocked, 2026-11-01T14:10Z)

# level-0a-ceiling-gun-route-2 — the report

Branch `s/level-0a-ceiling-gun-route-2` = `origin/s/level-0a-ceiling-gun-route` (42227169) merged
with `origin/master` (f3dd2eba). Task: finish the crouch-under-#607 real-damage route — the
outbound nav via, the fatal band legs, the laser band rule pinned, `walk_office_floor_qworm_
keyboard_hatch_real_damage` completing (un-ignored — it was already un-ignored on the branch),
every *0700000a* binary and nf-game run with totals.

## What's done

1. **The laser band rule is pinned live, both #607 stances** — new
   `walkreal_0700000a_live::laser_band_rule_crouch_clears_607_and_low_beams_fire_either_way`:
   - **standing raises 0x1F**: the interactables laser test's exact through-crossing leg
     (−3.9,15.5)→(−10,17.5) walked STANDING fires ch 0x1F (the standing body 1.86 [M
     Player_Collision 0x194df0 default arm] crosses beam #607's band 1.49-1.76 [D tris: x
     −8.59..−8.32, y 1.49..1.76, z 14.99..20.25]);
   - **crouched passes silent**: the same plane walked CROUCHED (r 0.6 arrive) plus a second
     crouched leg clear of the beam's standing reach leaves ch 0x1F off (sister: off before) —
     the crouched pick arm tops ≈ feet+1.26 [M 0x194df0 case 3], under the band — so OR #667
     never raises ch 0x0F and ceiling gun #640 stays dormant [D the OR wiring].
   - **the low beams are dodge-only, both stances**: the live low-beam crossing pin was tried
     (standing and crouched through #614's plane at x −20.5) and pulled — the port's toucher
     reaches the low band only MARGINALLY on both stances (the standing measured box bottoms
     pos−0.05 [Toucher::player], 0.47 off the band's top 0.52; the crouched segment bottoms
     pos+0.45−cc = 0.49 [M case 3's B arm with the park 0.607], 0.03 inside the band) and the
     crossings did not fire in either stance. The pin that HOLDS is the route's own: the walked
     dodges plus the final no-alert assert (no 0x22/0x24/0x25/0x26, no 0x0F-0x12/0x81) in
     `walk_office_floor_qworm_keyboard_hatch` — the low beams never fire because the route never
     lets a stance stand in their reach. The marginal-reach observation is recorded as a live
     check for the main agent (below).
2. **The outbound nav via** — the A* to the Q-Worm PC now goes through a via east of low beam
   #614's east end (x −19.45 [D tris]): `follow_nav_route([-15.8, 0.4, 10.0])` first (the
   south-west corridor, the walk-back's own via pattern), then the PC leg — the A*'s own west
   lane ran (−21.10,17.15)→(−22.12,13.38)→(−22.19,8.62), through #614's line at x ≈ −22.1,
   which arms gun #640 (0x22 → OR #667 → ch 0x0F) — the walk's crossing of #614 is gone.
3. **The band legs are fatal** — the outbound pocket/#621-dodge legs are `walk_to` (the
   6000-tick cap panics [walker]) instead of the lenient `try_walk_to`, so a silently-stranded
   leg can no longer leave Bond short of the install stand.
4. **The crouch is held clear of the beam's standing reach** — the route's hall crossing is
   r 0.6, then a second crouched leg to (−12, 14.5) BEFORE the uncrouch: the walker's 1.2-arrive
   path swings east into the toucher's 0.55 reach of the beam's tri face (the standing box at
   x −9.06 reaches the tri at 0.47 [D tris] and fires the beam — observed this round), so the
   uncrouch happens only 1.2+ west of the face.
5. **The branch's compile break fixed** — the prior session's `drone::Player::crouched` field
   landed without the crate's own test updates; 25 nf-game test files' Player/Bond initializers
   now carry `crouched: false` (nf-game builds clean again).
6. **The walk's remaining failure, precisely diagnosed**: the real-damage walk's CROUCHED
   outbound (the hall + the clear leg + the via — the crouch walk is slow) walks the south-west
   corridor past the panel-side challenge civilian's sight cone; his challenge → RunToAlarm →
   PressAlarm lands during the slow legs and the mission ends (level_exit) before the install —
   every later leg no-ops at +0 ticks. The ring legs dodge his cone by route; the outbound's new
   crouched pace needs the same treatment (the cone-avoidance vias or the ring's
   walk_leg_watched-with-catch pattern on the outbound legs). Bond took 10 hits / 24 damage
   before the fail (health 76) — the turrets never armed (the beam pin held).

## How it was verified (the totals, this session)

- `interactables_0700000a_live`: **7 passed / 0 failed** (7.2 s).
- `walk_0700000a_live`: **3 passed / 0 failed** (7.7 s).
- `fix_0a_rain_indoors_live`: **2 passed / 0 failed** (2.8 s).
- `walkreal_0700000a_live`: **6 passed / 4 failed / 2 ignored** (19.7 s) — the 4: the two route
  walks (the base + the real-damage wrapper) and `walk_office_floor_to_the_security_keyboard`
  (the same shared route — the challenge-runner fail above), and
  `bond_stands_at_the_mouth_bridge_edge` (the pre-existing mouth-edge repro, failing on the
  merged tree — not touched this round; see What's left).
- `nf-game` lib unittests: **636 passed / 0 failed / 5 ignored** (0.84 s) — one more than the
  prior session's 635 (master's newer tests ride the merge).
- The full suite (`tools/test/suite.py`, started at 13:21 UTC) died in its build phase on the
  `ai_live` E0063s above; after the initializer fix nf-game builds clean (`cargo test -p
  nf-game --no-run` Finished). The suite was not re-run to a total in this window — the next
  message should start it fresh on this tree.

## What's left

1. The outbound challenge-civilian problem (the route's last bug): the crouched pace needs the
   ring's cone-avoidance/catch pattern on the outbound legs; then the real-damage walk should
   complete and the final laser pin assert holds it.
2. `bond_stands_at_the_mouth_bridge_edge` (a repro test, not the route) fails on the merged
   tree — bisect master's player changes vs the branch's toucher work before the next route
   iteration.
3. The low-beam marginal-reach question (Live check 2) — worth a live answer before anyone
   relies on a stance clearing a low line.
4. The full suite total on this tree.

## Live checks for the main agent

1. Fresh-boot 0700000a: walk Bond crouched through beam #607's plane (x −8.45, z 15-20.25) —
   expect ch 0x1F stays 0 (the port's crouch arm matches [M 0x194df0 case 3]); the same STANDING
   — expect 0x1F fires.
2. **The low beams' stance margin**: fresh boot, walk Bond standing and then crouched through
   #614's line (z 13.03, x −24.7..−19.45, y 0.26-0.52 [D tris]) — does the game's ch 0x22 fire
   on either stance? The port's toucher sits 0.47 (standing) / 0.03 (crouched) off the band —
   the game's answer pins whether the port's measured standing box needs the game's capsule
   (A = pos+0.275, B = pos+0.55−cc, r 0.55 — whose reach bottoms pos−1.0 and would fire).
3. The challenge civilian's cone during a CROUCHED walk of the south-west corridor (z 10-14,
   x −12..−15.8): does he challenge/press on the game's timing (the port's runner presses at
   +≤700 ticks of slow crouch walking)? The route's vias depend on his real reach.

## ntfy failed

All three fallbacks (this sandbox has no route to ntfy.sh):
1. curl → `curl: (28) Connection timed out after 20002 milliseconds`
2. python3 urllib → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. wget → hung past the 120 s tool timeout (the same unreachable host).

## Usage

not visible to me — this sandbox exposes no token, context-window or cost figures for the session.
