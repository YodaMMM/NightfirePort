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
