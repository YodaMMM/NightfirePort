# WORKLOG

One entry per task, newest at the bottom. Append only — never rewrite or reorder.

## level-0d-wire-jump-grab (s/level-0d-wire-jump-grab)

- 2026-11-08 20:07 UTC: started. The ignored real-damage Operative walk on 0700000d
  (walkreal_0700000d_live.rs ~415) fails at the wire jump-grab — Bond does not catch the
  wire. Plan: read the ignored test + the port's grab scan, reproduce on the real path with
  a per-tick Bond-vs-wire log, find the game's grab rule in the decompile (Player_Wire
  0x197658's grab arm / the pick), fix the port if it differs (test red on old code), else
  fix the route as a player would and un-ignore what passes.
- 2026-11-08 21:00 UTC: done. The grab rule is right — the plain twin grabs end to end; the
  ignored Operative walk failed because Bond was DEAD at the grab (hp 6.0 -> 0.0 mid-face-
  loop, the death freeze eats every later input [M 0x193600:316-319]). The killer: the deck
  guard #16 (64.0 in 4 hits) whom the route's chest act could not hit — his fire anims pose
  his hit boxes 0.65-1.0 BELOW his origin (the head-box top 14.03 vs the origin 14.04) while
  the game's standing volume is the origin-centred capsule A = pos+up·r, B = pos+up·(r−hip),
  r 0.4/0.55 [M NDrone2_Collision 0x140e90:91-121] — the port's boxes-only drone_ray_part is
  the documented deviation (flagged, not landed: the shared fight path). The box act (aim at
  the visible crouched body) lands the kill at +209 and the wire #271 jump-grab fires at
  jump tick 60; the ride/deck/annex legs pass. The walk stays ignored: tick 12540 alive at
  4.8 health (26 hits for 88.8, the ride exposure) and the yard-slot hop diverged (the box
  act's RNG stream) — the ledger in the ignore message. walkreal_0d whole: the plain twin
  green. The full suite did not fit (the build phase died twice, 3 GB sandbox). Report:
  docs/reports/level-0d-wire-jump-grab.md.

- 2026-11-09 00:00 UTC: started. 0700000d walkable to the end on Operative: continue the
  route from s/level-0d-wire-jump-grab — the ride-phase sniper stands (88.8 drained by
  tick 12540) then the yard-slot hop under the guard state diverged by then; real input
  only, helpers gated to 0700000d and Operative, never lower damage or health.

- 2026-11-09 01:20 UTC: done (the Operative walk stays red; the ledger + the next levers
  are pushed in its ignore message). The route moved: the west-roof #27 dies FIRST from the
  ladder top (the direct box act, down at +222 — her chest ray full CLEAR, the settled
  shooter the shared sweep only caught ~1000 ticks in); the gantry park wait moved off the
  rim corner to the measured-quiet E-plate stand (the corner: 1 hit per ~45 ticks of 0.6
  0x12 fire; the E-plate region zero hits, ticks 10000-11487); a 40-tick settle rides out a
  pain stagger between failed slot hops (the hop mechanism measured: the 0.6-trickle
  staggers eat the jump input whole, vel [0,0,0] through five attempts); the post-drop sweep
  gains the g8/#2 legs. All real-damage-gated: the plain master replay stays byte-identical
  and green (1 passed), interactables_0700000d 11 passed, nf-game 40 passed, nf-formats +
  nf-audio 142 passed / the 3 slim-ISO disc_golden failures. Best Operative run: tick 13800
  alive at 15.0 (9 hits for 78.6 + other 6.4) — the floor is the bay guard #18 (3-5 × 16
  every run, his bay walls eat every outside chest ray; the head-line counter was TESTED and
  is net negative — it killed g10 at +330 but fed #18 two more shots, 94 vs the baseline 78;
  the leg is out, the story is in the body). The yard pit is unshootable from the ladder
  top, the roof lane and the bay enter (three scans broke on empty picks). Report:
  docs/reports/level-0d-operative-walk.md.

- 2026-11-09 04:10 UTC: started (level-0d-operative-walk-2). Continue 0700000d on Operative
  from s/level-0d-operative-walk's ledger: lever 1 the crouch-dodge test at the corner (the
  bay/yard opening line vs a crouched chest — the corner wait goes quiet, the hop fires
  clean), lever 2 the drop-then-sweep (land in the yard, sweep at once, the garrison still
  asleep). The standing-capsule change stays in its own task. Real input only, no top-ups;
  helpers gated to 0700000d and the difficulty.

- 2026-11-09 05:30 UTC: done (level-0d-operative-walk-2; the Operative walk stays red at the
  yard, the ledger + the next lever are in its ignore message). Both levers landed: (1) the
  REAL-DAMAGE REEL BUG — the hang loop broke on max_ox > 177 alone and the real-damage twin's
  E-plate wait + corner walk eat the beam's east crawl, so the loop started at/past the apex
  and released Bond mid-reel (the claw latched, pos[1] 10.67 on the deck floor — the -1's
  "tick 13800 best run" was this assert with its stored summary misread as alive); the break
  now waits for the reel's phase 2 [M 0x192860:91-94], the plain replay byte-identical and
  green. (2) The CROUCH-DODGE — PROVEN: crouch ON at the rim corner through the wait, the
  latch, the hang and the ride (the Toggle press edge, a fresh press stands); the baseline's
  12-tick-cadence 0.6 corner trickle (#3) went to ZERO hits for ~1300+ ticks — the opening
  line clips the crouched chest (the aim target drops 0.45 [M 0x195aa0:39-51]) and the
  visibility factor 0.75 [M 0x1390c0:40-42] drops under the 0.7 LOS gate. (3) The DROP-THEN-
  SWEEP — LANDED: the boarding, the hang, the ride (offset 2.00, the apex 138.19) and the
  slot hop all pass, the drop landed at (137.98, 5.02, −296.08), the sweep is the nearest-
  clear-box-first dynamic loop (the fixed order swept the far corner while #3 had walked to
  the drop point, d 55 → 7.9). DEATH MOVED 13800 → 15646 (+1846 ticks). The floor is upstream
  health: the boarding arrives at ~15.0 (9 hits for 78.6 — #29's two ladder-top snipes,
  #18's bay-enter 16, #17's ride-window 16, #10's deck 16), the slot fall ~10-12 skips
  armour [M 0x1902c8:84-104], and 3.2 at the landing dies in 64 ticks — no aim converges
  that fast. NEXT: the Operative ladder-top band sweep (#17/#10) + the in-bay #18 kill audit
  (400 ticks never landed a 10-health guard at d 19.7 in-room, every run). Runs: walkreal
  whole 1 passed/2 ignored, interactables 11 passed, nf-game 1179 passed/0 failed, nf-formats
  +nf-audio 142 passed/3 failed (the allowed slim-ISO disc_golden trio); the full suite does
  not fit the window (1069 nf-view test targets, the build phase alone exceeds it). Report:
  docs/reports/level-0d-operative-walk-2.md.

- 2026-11-09 08:25 UTC: started (level-0d-operative-walk-3). Continue 0700000d on Operative
  from s/level-0d-operative-walk-2's ledger: the upstream health floor — the Operative band
  sweep from the ladder top (#17/#10) and the in-bay #18 kill audit — so Bond boards at 20+
  health, then the landed drop-then-sweep closes the yard, the plate, the photograph and the
  door legs. Real input only, no top-ups; helpers stay gated to 0700000d and the difficulty;
  never lower damage or health.

- 2026-11-09 09:45 UTC: done (level-0d-operative-walk-3; the walk stays red — the ledger +
  the next levers are in the ignore message). Both levers taken. (1) THE IN-BAY #18 AUDIT —
  SOLVED: the walked arrive drifts 1.4 north of the probed stand and the in-room ray DIES at
  0.068 on the west-facing pad wall at x ~195.4 (the world gate never opened, the converge was
  moot); the fix (the re-walk r 0.6 + the clear-box act over the bay pair) is in, untested past
  the annex. (2) THE GROUND-BAND SWEEP extended to Operative (real_damage-gated, 1 × 400; the
  Agent twin keeps 3 × 600): g17 patrols the WHOLE band and her east spot (163.28,−213.99)
  blocks the ladder-top ray at 0.982; the single pass let the route reach the annex for the
  first time (death 9880 AT the annex landing, 6.0 − 6.4 fall). TWO MEASURED DEAD ENDS (both
  removed, both recorded in the code): the strip-mouth #10 widened box (acquired g10, ray
  CLEAR, the cross CONVERGED, 3 on-target rounds missed at d 62; the leg's rounds re-rolled the
  shared stream into a dead climb, 4472) and the west-roof clear-box act (the recoil kicks the
  cursor −0.05→−0.30 per shot, the target walks behind the slab 0.973, ~1200 ticks + 13 rounds
  for ONE kill, deaths 4385/4917). THE REMAINING GAME IS ONE GUARD: #18's INBOUND line from the
  bay is open from the ladder top through the deck (4 hits for 62 in the v6 roll) while the
  OUTBOUND box stays blocked; next: the wait-for-crossing clear-box act at the ladder top or
  the deck-stand act (d 29, the 7479 line). Determinism: comment-only rebuilds do NOT re-roll
  (v7 == v6); only real code-path changes do. Runs: walkreal whole 1 passed/2 ignored (the
  plain master replay green, 21.73 s), interactables_0700000d_live 11 passed/0 failed (32.96 s,
  --test-threads=1); the suite runner not started — the build phase holds the cargo lock and
  exceeds the window (the -2's same-box measurement stands; the two required binaries ran
  whole). Report: docs/reports/level-0d-operative-walk-3.md.
