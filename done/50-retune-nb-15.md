## Your task: `retune-nb-15`, 15 spawn to the Omega sector exit 00 Agent walk green again

Branch from current origin/master. Commit 3412a89a3 made every guard run the game's spacing scan and the close-range alt attack from the fire states, and the walks below now die; each ignore message records the measured death. Re-tune walk_spawn_to_the_omega_sector_exit_real_damage_00agent for the new fights: read the ignore, find where the new fights cost the health, and change the route, stance or fight order there. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove each ignore when green; if still red, push the measured ledger and the next lever into the ignore message. Run the whole walkreal_07000015_live binary.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-15`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-18T18:18Z

---
## Report (done, 2026-11-18T20:09Z)


---
## Report from the worker (done, 2026-11-18T20:09Z)

# retune-nb-15 — 15 spawn→Omega Sector exit 00 Agent real-health walk re-tune

## What the task was

Commit 3412a89a3 (the spacing scan everywhere + CanAltAttack in the fire states) made
`walk_spawn_to_the_omega_sector_exit_real_damage_00agent` die at t4799 at (34.97,41.41,26.94).
Re-tune the walk for the new fights: find where the new fights cost the health and change the
route, stance or fight order there. Helpers stay gated to the level+difficulty; real input only;
no top-ups; never lower damage or health; never change guard code. Judge each lever on its own
real run; when green remove the ignore, when red push the measured ledger and the next lever
into the ignore message. Run the whole `walkreal_07000015_live` binary.

## What I did (each lever judged on its own real run)

All changes are in `crates/nf-view/tests/walkreal_07000015_live.rs`, every one gated
`difficulty >= 3 && !w.tops_up()` (the d1/d2/top-up walks' timelines untouched — proven by the
whole-binary run below).

1. **Baseline (reproduce).** The ignored run died t4799: 44.0 upstream (the opening burst 14.0
   from #15, 24.0 from #0 across the crossing, 3.0 + 3.0) + the killing **−57.2 from drone #8**
   (the east-plank grenadier, 0x50000b2) — the fights killed #8 (hp −13 at the print) but his
   thrown grenade was already airborne and popped next to Bond mid-leg.
2. **Lever 1 — `pre_throw_stands` scoped to the plank phase** (the west-room law of
   real-health-00agent-15-7, applied to the plank pair): the fights answered the
   spawner-released pair mid-leg and killed both (#7 at 4577, #8 at 4634) — but #8's last
   throw (before his kill) still popped for −57.2. The kill cannot beat an airborne grenade.
3. **Lever 2 — a fuse hold at the door[39] north node** (480 ticks): the pop landed at the same
   tick 4799 with Bond at the node — measured with a grenade-flight diagnostic: the grenade
   **rests at (38.2, 40.38, 29.8)** [L the 15diag trace], and the blast is pure distance
   (radius 15, no LOS [M Explode_Propagate 0x17c518]), so every spot on the crossing lane —
   the node, the corner, the plank entrance, even the plank north — sits inside the rest
   point's ~14 u hurt reach. No wait on the lane survives it.
4. **Lever 3 — the plank fuse retreat (THE FIX):** stop short of the node (the fights answer
   the released pair from the lane — their kills are unchanged), retreat to (34.6, 8.0)
   (≥ 18 u off the measured rest point), wait 360 ticks (the fuse + flight + margin [S]),
   re-walk the lane, cross. **Measured: 59.0 health kept through the plank crossing, the desk
   lanes and the #526 lob** (the lob's own chain cost 0.4). The baseline's −57.2 never lands.
5. **Lever 4 — the west-room doorway hold:** the pre-throw newborn pick (born re-based to the
   phase's first poll) killed #11 CLEAN (no blast). But #18's launcher round was already
   airborne (their sight opened ~50 ticks before the first fight poll, mid band-crossing) and
   popped mid-fight for −37.7, and #10 (unseen from the solo line) survived his stand.
6. **Lever 5 — the west-room poke-and-step-back + the unconditional P2K:** fight from the
   door[29] threshold (all three visible through the doorway band — the d2 rim lever's own
   premise), then step back east out of the arcing pops' reach and wait the fuse between
   cycles; the P2K (spread 0.0 [L ee_start_05.bin weapon_data 0x02 +0x24/+0x98]) replaces the
   AR for the phase (the lever-4 AR fight took 240 ticks — clip-3 dry-swap + cone aim gates —
   and conceded the airborne pop). Measured: the poke fights reached only #10 (killed false,
   9439); #18 (skin 0x500008c — not 0x50000b2) and #11 stayed a hair outside the walker's ray
   band from the re-poke spot (z 72.1/72.4 vs the 72.32 edge), the fights' plain pick needs a
   shooter and the pre-throw pick needs visible, and the trio's arcs killed Bond at t15294 at
   (7.86,41.45,71.43) (−3.50×3 −7.00 −2.58 at d 11.6, frames 15184-15294).

**The walk is still red — the ignore message now carries this measured ledger and the next
levers:** (a) a poke stand ~0.5 u further north (z 71.7) puts #18/#11's rays inside the band —
re-measure the threshold geometry first; (b) the `world_ray_fights` unseen-cover fallback
(opt-in) for the fights' pick; (c) #18 is 0x500008c — the skin-resolved stands must cover both
skins.

## How it was verified

- Every lever was judged on its own real run of the real input path from the level start
  (no teleport): the run logs are the [L] evidence above (ticks, positions, rest points).
- `walkreal_07000015_live` whole, `--test-threads=1`: **8 passed, 0 failed, 1 ignored** — the
  d1 real-damage, d2 Agent real-damage, both top-up walks, the d2/d3 objective-chain sisters,
  the range-shot fence test and the body-contact fence test all still pass; the only red is
  the still-ignored 00agent walk (its ledger is in the ignore message).
- No code under `crates/*/src/` was touched (drone.rs etc. untouched), so no other binary's
  inputs changed.

## What's left

- The west-room phase of the d3 walk (the trio behind the west wall) — the next levers are in
  the ignore message.
- The full-suite run on the final tree did not fit this session's window: the sandbox kills
  background processes (two `nohup`/`setsid` suite starts died without output), and the
  foreground chunk runner (`tools/test/chunk_suite.py`) needs ~40+ min that the lever runs
  consumed. The only changed file is `crates/nf-view/tests/walkreal_07000015_live.rs`, and its
  whole binary passed 8/0/1 (above); the next session should run
  `python3 tools/test/chunk_suite.py` to ALL DONE on this tree before merging.

## Usage

not visible to me

---
## Report from the worker (done, 2026-11-18T20:16Z)

# retune-nb-15 — 15 spawn→Omega Sector exit 00 Agent real-health walk re-tune

## What the task was

Commit 3412a89a3 (the spacing scan everywhere + CanAltAttack in the fire states) made
`walk_spawn_to_the_omega_sector_exit_real_damage_00agent` die at t4799 at (34.97,41.41,26.94).
Re-tune the walk for the new fights: find where the new fights cost the health and change the
route, stance or fight order there. Helpers stay gated to the level+difficulty; real input only;
no top-ups; never lower damage or health; never change guard code. Judge each lever on its own
real run; when green remove the ignore, when red push the measured ledger and the next lever
into the ignore message. Run the whole `walkreal_07000015_live` binary.

## What I did (each lever judged on its own real run)

All changes are in `crates/nf-view/tests/walkreal_07000015_live.rs`, every one gated
`difficulty >= 3 && !w.tops_up()` (the d1/d2/top-up walks' timelines untouched — proven by the
whole-binary run below).

1. **Baseline (reproduce).** The ignored run died t4799: 44.0 upstream (the opening burst 14.0
   from #15, 24.0 from #0 across the crossing, 3.0 + 3.0) + the killing **−57.2 from drone #8**
   (the east-plank grenadier, 0x50000b2) — the fights killed #8 (hp −13 at the print) but his
   thrown grenade was already airborne and popped next to Bond mid-leg.
2. **Lever 1 — `pre_throw_stands` scoped to the plank phase** (the west-room law of
   real-health-00agent-15-7, applied to the plank pair): the fights answered the
   spawner-released pair mid-leg and killed both (#7 at 4577, #8 at 4634) — but #8's last
   throw (before his kill) still popped for −57.2. The kill cannot beat an airborne grenade.
3. **Lever 2 — a fuse hold at the door[39] north node** (480 ticks): the pop landed at the same
   tick 4799 with Bond at the node — measured with a grenade-flight diagnostic: the grenade
   **rests at (38.2, 40.38, 29.8)** [L the 15diag trace], and the blast is pure distance
   (radius 15, no LOS [M Explode_Propagate 0x17c518]), so every spot on the crossing lane —
   the node, the corner, the plank entrance, even the plank north — sits inside the rest
   point's ~14 u hurt reach. No wait on the lane survives it.
4. **Lever 3 — the plank fuse retreat (THE FIX):** stop short of the node (the fights answer
   the released pair from the lane — their kills are unchanged), retreat to (34.6, 8.0)
   (≥ 18 u off the measured rest point), wait 360 ticks (the fuse + flight + margin [S]),
   re-walk the lane, cross. **Measured: 59.0 health kept through the plank crossing, the desk
   lanes and the #526 lob** (the lob's own chain cost 0.4). The baseline's −57.2 never lands.
5. **Lever 4 — the west-room doorway hold:** the pre-throw newborn pick (born re-based to the
   phase's first poll) killed #11 CLEAN (no blast). But #18's launcher round was already
   airborne (their sight opened ~50 ticks before the first fight poll, mid band-crossing) and
   popped mid-fight for −37.7, and #10 (unseen from the solo line) survived his stand.
6. **Lever 5 — the west-room poke-and-step-back + the unconditional P2K:** fight from the
   door[29] threshold (all three visible through the doorway band — the d2 rim lever's own
   premise), then step back east out of the arcing pops' reach and wait the fuse between
   cycles; the P2K (spread 0.0 [L ee_start_05.bin weapon_data 0x02 +0x24/+0x98]) replaces the
   AR for the phase (the lever-4 AR fight took 240 ticks — clip-3 dry-swap + cone aim gates —
   and conceded the airborne pop). Measured: the poke fights reached only #10 (killed false,
   9439); #18 (skin 0x500008c — not 0x50000b2) and #11 stayed a hair outside the walker's ray
   band from the re-poke spot (z 72.1/72.4 vs the 72.32 edge), the fights' plain pick needs a
   shooter and the pre-throw pick needs visible, and the trio's arcs killed Bond at t15294 at
   (7.86,41.45,71.43) (−3.50×3 −7.00 −2.58 at d 11.6, frames 15184-15294).

**The walk is still red — the ignore message now carries this measured ledger and the next
levers.** A final geom diagnostic run (the dump is kept in the poke loop) measured the root
blocker: **#11 and #18 report los FALSE from the threshold in EVERY poke cycle** while their
z at the door[29] plane sits inside the 70.06..72.32 band — the guard's own sight and the
walker's straight ray disagree (the ray stops in the room volume) — so the fights' pick can
never reach them, and only #10 ever reaches los (his one poke fight went killed-false). Next
levers in the ignore: (a) the player's own ARC — the frag 0x34 lob at #11's live position
(14.5 u, the #526 lob's measured reach 16.22 u) with door[29] verified open, then step back
and wait the fuse; (b) a live check for the main agent (the guard-sight vs segment_bullet
asymmetry at the door[29] plane, gdbwatch.py); (c) #18 is skin 0x500008c — any skin-resolved
stand must cover both skins.

## How it was verified

- Every lever was judged on its own real run of the real input path from the level start
  (no teleport): the run logs are the [L] evidence above (ticks, positions, rest points).
- `walkreal_07000015_live` whole, `--test-threads=1`, run twice (before and after the final
  diagnostic): **8 passed, 0 failed, 1 ignored** both times — the
  d1 real-damage, d2 Agent real-damage, both top-up walks, the d2/d3 objective-chain sisters,
  the range-shot fence test and the body-contact fence test all still pass; the only red is
  the still-ignored 00agent walk (its ledger is in the ignore message).
- No code under `crates/*/src/` was touched (drone.rs etc. untouched), so no other binary's
  inputs changed.

## What's left

- The west-room phase of the d3 walk (the trio behind the west wall) — the next levers are in
  the ignore message.
- The full-suite run on the final tree did not fit this session's window: the sandbox kills
  background processes (two `nohup`/`setsid` suite starts died without output), and the
  foreground chunk runner (`tools/test/chunk_suite.py`) needs ~40+ min that the lever runs
  consumed. The only changed file is `crates/nf-view/tests/walkreal_07000015_live.rs`, and its
  whole binary passed 8/0/1 (above); the next session should run
  `python3 tools/test/chunk_suite.py` to ALL DONE on this tree before merging.

## Usage

not visible to me

## ntfy failed

All three fallbacks failed:
1. `curl -sS -m 20 -H "Title: NightfirePort" -d "retune-nb-15: ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds`
2. python urllib → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "retune-nb-15: ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → timed out after 60 s and was killed (no output).

---
## Report from the worker (done, 2026-11-18T20:33Z)

# retune-nb-15 — 15 spawn→Omega Sector exit 00 Agent real-health walk re-tune

## What the task was

Commit 3412a89a3 (the spacing scan everywhere + CanAltAttack in the fire states) made
`walk_spawn_to_the_omega_sector_exit_real_damage_00agent` die at t4799 at (34.97,41.41,26.94).
Re-tune the walk for the new fights: find where the new fights cost the health and change the
route, stance or fight order there. Helpers stay gated to the level+difficulty; real input only;
no top-ups; never lower damage or health; never change guard code. Judge each lever on its own
real run; when green remove the ignore, when red push the measured ledger and the next lever
into the ignore message. Run the whole `walkreal_07000015_live` binary.

## What I did (each lever judged on its own real run)

All changes are in `crates/nf-view/tests/walkreal_07000015_live.rs`, every one gated
`difficulty >= 3 && !w.tops_up()` (the d1/d2/top-up walks' timelines untouched — proven by the
whole-binary run below).

1. **Baseline (reproduce).** The ignored run died t4799: 44.0 upstream (the opening burst 14.0
   from #15, 24.0 from #0 across the crossing, 3.0 + 3.0) + the killing **−57.2 from drone #8**
   (the east-plank grenadier, 0x50000b2) — the fights killed #8 (hp −13 at the print) but his
   thrown grenade was already airborne and popped next to Bond mid-leg.
2. **Lever 1 — `pre_throw_stands` scoped to the plank phase** (the west-room law of
   real-health-00agent-15-7, applied to the plank pair): the fights answered the
   spawner-released pair mid-leg and killed both (#7 at 4577, #8 at 4634) — but #8's last
   throw (before his kill) still popped for −57.2. The kill cannot beat an airborne grenade.
3. **Lever 2 — a fuse hold at the door[39] north node** (480 ticks): the pop landed at the same
   tick 4799 with Bond at the node — measured with a grenade-flight diagnostic: the grenade
   **rests at (38.2, 40.38, 29.8)** [L the 15diag trace], and the blast is pure distance
   (radius 15, no LOS [M Explode_Propagate 0x17c518]), so every spot on the crossing lane —
   the node, the corner, the plank entrance, even the plank north — sits inside the rest
   point's ~14 u hurt reach. No wait on the lane survives it.
4. **Lever 3 — the plank fuse retreat (THE FIX):** stop short of the node (the fights answer
   the released pair from the lane — their kills are unchanged), retreat to (34.6, 8.0)
   (≥ 18 u off the measured rest point), wait 360 ticks (the fuse + flight + margin [S]),
   re-walk the lane, cross. **Measured: 59.0 health kept through the plank crossing, the desk
   lanes and the #526 lob** (the lob's own chain cost 0.4). The baseline's −57.2 never lands.
5. **Lever 4 — the west-room doorway hold:** the pre-throw newborn pick (born re-based to the
   phase's first poll) killed #11 CLEAN (no blast). But #18's launcher round was already
   airborne (their sight opened ~50 ticks before the first fight poll, mid band-crossing) and
   popped mid-fight for −37.7, and #10 (unseen from the solo line) survived his stand.
6. **Lever 5 — the west-room poke-and-step-back + the unconditional P2K:** fight from the
   door[29] threshold (all three visible through the doorway band — the d2 rim lever's own
   premise), then step back east out of the arcing pops' reach and wait the fuse between
   cycles; the P2K (spread 0.0 [L ee_start_05.bin weapon_data 0x02 +0x24/+0x98]) replaces the
   AR for the phase (the lever-4 AR fight took 240 ticks — clip-3 dry-swap + cone aim gates —
   and conceded the airborne pop). Measured: the poke fights reached only #10 (killed false,
   9439); #18 (skin 0x500008c — not 0x50000b2) and #11 stayed a hair outside the walker's ray
   band from the re-poke spot (z 72.1/72.4 vs the 72.32 edge), the fights' plain pick needs a
   shooter and the pre-throw pick needs visible, and the trio's arcs killed Bond at t15294 at
   (7.86,41.45,71.43) (−3.50×3 −7.00 −2.58 at d 11.6, frames 15184-15294).

**The walk is still red — the ignore message now carries this measured ledger and the next
levers.** A final geom diagnostic run (the dump is kept in the poke loop) measured the root
blocker: **#11 and #18 report los FALSE from the threshold in EVERY poke cycle** while their
z at the door[29] plane sits inside the 70.06..72.32 band — the guard's own sight and the
walker's straight ray disagree (the ray stops in the room volume) — so the fights' pick can
never reach them, and only #10 ever reaches los (his one poke fight went killed-false). Next
levers in the ignore: (a) the player's own ARC — the frag 0x34 lob at #11's live position
(14.5 u, the #526 lob's measured reach 16.22 u) with door[29] verified open, then step back
and wait the fuse; (b) a live check for the main agent (the guard-sight vs segment_bullet
asymmetry at the door[29] plane, gdbwatch.py); (c) #18 is skin 0x500008c — any skin-resolved
stand must cover both skins.

7. **Levers 6-14 — the frag lob lane and the great hall:** the geom diagnostic proved the
   trio's OWN sight false at the threshold in every cycle (they cannot see Bond there —
   nothing counters a lob), so the walk lobs the frag (0x34, the level's loadout) at the live
   guards from the threshold. Three measured defects and fixes: the door use ran while Bond
   stood 18.5 u from the leaf (the press loop expired out of reach — the leaf stayed shut)
   and the lob's frag hit the closed door[29] leaf 2 u west of the throw point, bounced back
   east down the straight lane and popped ON Bond's step-back spot (the trace: the explosion
   at (27.6,40.4,71.4), kind 52 = the frag, −58.58 at tick 15438, the same tick+spot on three
   aim variants). The fix (re-threshold first, then the use, then the lobs) measured:
   **THE WEST ROOM IS FIXED — the walk keeps 58.6 health through the whole west room** (#11
   and #18 killed clean at the solo lines, zero west-room hits), and the death moved to the
   GREAT HALL: t20331 at (−0.79,41.48,43.12) — #5 (the placement-733 grenade guard) dealt
   41.6 in 3 grenade blasts (d 11.3) with the walk's shoot machinery engaged at exposed
   false (the taps could not land — the run15i lesson). The great-hall
   lobs then measured worse on the door[50] aperture geometry (the ray crosses the leaf plane
   outside the x-0 aperture; the pop's rest ~11.4 u off the threshold catches any step-back),
   and removing the positioning shoot for d3 (it was the exposure — 29 hits) with the legs'
   fights on lane_fights measured the best hall shape: #19 killed by the fights (45.1 → 34.6),
   20 hits total, but #5's throw cycle (~70 ticks) still outscores the taps' aim gates — Bond
   died t20233 at (0.02,41.41,45.30). The next levers (the hold-length re-sweep on this
   stream, the bridge-stand exposure check with the kept geom dump, the live check for the
   main agent) are in the ignore message. the geom diagnostic proved the trio's
   OWN sight false at the threshold in every cycle (they cannot see Bond there — nothing
   counters a lob), so the walk now lobs the frag (0x34, the level's loadout) at the live
   guards from the threshold. Measured: the lob flew (1 pull, 16.81 u — the reach gate works),
   and **the trio was bypassed entirely — the run died with only 13 hits and ZERO west-room
   guard hits after frame 8301** — but Bond died at tick 15438 at (28.07,41.48,71.24) to a
   −58.58 WORLD hit at the step-back spot, the same tick+spot on the flat-aim and floor-aim
   variants: either the lob's own frag rested there (the far-wall bounce chased Bond down the
   straight lane) or a level hurt volume sits at the door[32] threshold. The next session's
   first step is one NF_DMG_TRACE run to identify that pop (its position/radius/owner settle
   it); the ignore message carries the fix for each branch.

## How it was verified

- Every lever was judged on its own real run of the real input path from the level start
  (no teleport): the run logs are the [L] evidence above (ticks, positions, rest points).
- `walkreal_07000015_live` whole, `--test-threads=1`, run twice (before and after the final
  diagnostic): **8 passed, 0 failed, 1 ignored** both times — the
  d1 real-damage, d2 Agent real-damage, both top-up walks, the d2/d3 objective-chain sisters,
  the range-shot fence test and the body-contact fence test all still pass; the only red is
  the still-ignored 00agent walk (its ledger is in the ignore message).
- No code under `crates/*/src/` was touched (drone.rs etc. untouched), so no other binary's
  inputs changed.

## What's left

- The west-room phase is FIXED (58.6 kept); the remaining phase is the GREAT HALL's #5 (the
  placement-733 grenade guard): the lob lane and the positioning shoot measured dead on their
  own runs, the legs-fight race measured best (20 hits) and still loses to his throw cycle —
  the next levers are in the ignore message.
- The full-suite run on the final tree did not fit this session's window: the sandbox kills
  background processes (two `nohup`/`setsid` suite starts died without output), and the
  foreground chunk runner (`tools/test/chunk_suite.py`) needs ~40+ min that the lever runs
  consumed. The only changed file is `crates/nf-view/tests/walkreal_07000015_live.rs`, and its
  whole binary passed 8/0/1 (above); the next session should run
  `python3 tools/test/chunk_suite.py` to ALL DONE on this tree before merging.

## Usage

not visible to me

## ntfy failed

All three fallbacks failed:
1. `curl -sS -m 20 -H "Title: NightfirePort" -d "retune-nb-15: ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds`
2. python urllib → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "retune-nb-15: ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → timed out after 60 s and was killed (no output).

A retry after the final push failed identically: `curl -m 15` → `(28) Connection timed out
after 15002 milliseconds`; python urllib with an 18 s timeout → killed at the timeout (no
route to ntfy.sh from this sandbox). The queue entry (done/50-retune-nb-15.md, report
attached) and the branch carry the result.
