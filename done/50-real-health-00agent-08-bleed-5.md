## Your task: `real-health-00agent-08-bleed-5`, 07000008's 00 Agent whole-mission real-damage walk wins the Hind fight

Branch from current origin/master; if origin/s/real-health-00agent-08-bleed-4 is not on master, merge it in first. Read docs/reports/real-health-00agent-08-bleed-4.md there and the ignore message on walk_the_whole_mission_on_real_input_real_damage_00agent. Take the ride's rocket-ammo pickups the measured roll left untaken and the drain-class lever the ledger names; keep every new block gated to difficulty 3 (an ungated form regressed the d1 sister last round). Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000008_live binary and interactables_07000008_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-08-bleed-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-14T14:18Z

Main 11-14 (master 06db4a156, NF_DMG_TRACE): on the merged bleed-4 stream the Hind fight (t0 at frame 20137, health 24, armour 50 from the secure-room + ledge armours) dies at t20428 to "world" -2.0 hits every ~6 ticks from frame 20332 (the trace flags 0x1C 0x27 0x28 0x06), Bond at (111.17,13.52,-179.81) = the bleed-4 ANCHOR; line_ok false the whole fight, ZERO rocket launches. Moving the start spot / the ANCHOR to (110.98,-179.31) or (110.6,-179.0): the same -2.0 cadence, deaths t20379-t20530, still 0 launches. Open question first: is this -2.0 the terminal volume or the Hind gun attributed to the world (it starts ~190 ticks into the fight); the old master stream (t34277) launched rocket 1 at t531 from (110.98,-179.31).

Main 11-14 answer to the open question: the -2.0 / 6-tick hits ARE the Hind gun (copter.rs fire_gun_tick fires every 6th call, the unit test fire_gun_fires_every_sixth_call; the round is attributed to the world in the damage log). On this stream the Hind sits SOUTH of the gondola (z -193..-208) while the firing slit is the cabin WEST doorway: the launch line from the anchor is blocked by the cabin wall at t 0.11-0.72 (the hind t300/t400 scans), so line_ok never opens while the gun has Bond. Next: a fight spot that sees south (the doorway edge), or wait the gun out in cover until the circuit brings the Hind west.

Main 11-14: a cover-anchor lever (on each gun hit, re-anchor to the nearest cabin point whose chest-to-Hind segment_bullet_scoped is blocked) picks Bond s own spot (111.1,-180.0)/(111.1,-179.6) as blocked while the gun keeps landing there - the Hind gun rounds do not obey that world trace (check fire_gun_tick s round path / collision scope before using sight lines for cover). Death t20415, 0 launches. Not shipped.

Main 11-14: the Hind gun is legit (rounds via the west doorway, see done/20-level-08-hind-gun-through-cabin). An eye-only launch line (instead of the eye + both gun spots) never opens either (0 line_ok samples): the Hind stays above the lintel line for the whole ~290 ticks Bond lasts. Anchors (109.5/109.0,-179.4) are not reachable (Bond stays at x ~110.95 - he is AT the doorway already). The fight needs either more health into it (24 + 50 armour now) or a way to stay out of the doorway s downward line until the circuit brings the Hind low.

Main 11-14 (6ffd3f07d merged: crouched deep anchor, death t26671, rocket 1 landed 100.7): the eye-only launch line from the deep anchor is byte-identical (1 launch) - the binding gate is slit_clear / the aim gates, not the 3-spot line. Bond dies to the Hind missile blasts (-7.13 at 25838, -10.58 at 26552).

Main 11-14 (6ffd3f07d): the fire-gate census from the deep crouched anchor (every 50 ticks, t>300): yaw/pitch/range pass almost always; machine_lane_clear is FALSE in every sample and eye_sees false in ~85% - from (112.5,-179.4) the Hind is mostly out of sight (the deep anchor that hides Bond from the gun also hides the Hind). A shorter slit (1.5/2.2 u) changes nothing (byte-identical). Next: a two-spot dance - wait deep while the gun cycles, step to the doorway anchor (111.09,-179.72) only when eye_sees from there, fire, step back.

Main 11-14: the timed deep/doorway anchor cycle (cycle,door = 600,200 / 400,200 / 900,300; deep whenever the gun just hit) - 0 launches, deaths t24509/t22511/t24508 (worse than the plain deep anchor t26671, 1 launch). The doorway leg must be triggered by the Hind being in sight from the doorway, not by a timer.

Main 11-14: a sight-triggered doorway step (go to (111.09,-179.72) only when a doorway eye has a clear bullet line to the Hind centre) never triggers - byte-identical t26671, 1 launch. The Hind is out of sight from BOTH spots nearly the whole fight; the one launch (t4953) came on its own pass. The fight needs a spot with a wider view (outside the carriage?) or the Hind s circuit timing - read Copter path data for when it passes in front of the doorway.

Main 11-14: the 20.0 at the briefcase start (158.93,-116.52) is #4 (0x5000014, 0xa) at (161.6,-120.8), 5 hits; three bounded stands on him there never kill him and the shoot positioning breaks the briefcase take (assert ch 0x4a, line 818, t13560). The station hall costs 40 (#3/#14/#11/#2, 4.0 each, 684-1114) at the walkway head.

Main 11-14: crouched across the station hall (d697c77f2) and for the briefcase stretch (cb29af974) - death t26671 -> t34720, the Hind fight now lasts ~14500 ticks; still ONE rocket (100.7 of 598). A crouch at the lower landing is byte-identical (already crouched). The Hind fight needs launch windows: the Hind is out of sight from the carriage most of its circuit.

Main 11-14 (cb29af974): over the ~14500-tick fight the Hind is in sight (eye_sees) at 140 sampled windows; in those, corridor_clear and machine_lane_clear are false in ~95%. Shortening the 12 u corridor/effective-corridor casts to 8/6 u is byte-identical (1 launch): machine_lane_clear (the forecast lane of the machine s own rounds) is the binding gate. Read that gate next.

Main 11-14: stand-when-the-Hind-is-in-sight / crouch-otherwise in the fight loop - 0 launches, death t29954 (worse). Not shipped.

Main 11-14: no live armour pickup within 150 u of the Hind fight anchor (Bond enters at 52.0/37.7, t27252).

Main 11-14: the raised surfaces by the anchor read floors 15.1-16.7 at x 110.5-111 (the terminal); a standing jump+forward from (111.1, -179.5) does not get Bond up (13.54 after). Dead.

Main 11-14: the fight-time world hits are NOT the fusebox (fusebox_hurts 0 through the fight, 44 world hits by t7000) - they are the owner-less Hind rounds. (Player_Hurt 0x196060 has no position gate, so a flat fusebox hurt would match the game anyway.)

---
## Report (done, 2026-11-15T02:02Z)


---
## Report from the worker (done, 2026-11-15T02:02Z)

# real-health-00agent-08-bleed-5 — the 00 Agent whole-mission walk: the ride pickups, the drain class, one gate's relaxation

Branch `s/real-health-00agent-08-bleed-5` from origin/master cb29af974 (the bleed-4 branch is
not on the remote; its walk code IS master — the worker reports were dropped in 88051e74f, so
the ignore message on `walk_the_whole_mission_on_real_input_real_damage_00agent` was the
ledger). The walk stays RED: death t34720 in the Hind fight, unchanged from update 3. What
this session shipped is the measurement that closes two of the three ledger levers and one
d3-only gate change that cannot regress the sisters.

## Done

1. **The ride's rocket-ammo pickups: measured, and the pool-growth lever is closed by the
   game's own data.** A full NF_PICKUPLOG run of the walk (run-bleed5-1) traced every take
   test of the ride's three pickups (the kind-0 launcher and two kind-1 boxes parented to the
   carriage, 0x0600004C streams 37-39 [D]):
   - the passive-collection path WORKS in the port: the pickups follow the carriage (the
     `script_pickup_follow` link), the touch test passes with los true in the ride's final
     ticks (dist 1.42-1.48 vs r 1.53), and Bond took the mouth box the moment his carry had
     room — tick 24599, `Some(Ammo { 44, 4 })`, carry 0 → 4 at the gondola mouth.
   - every take after that refuses at the **ammo cap**: `ammo_data` type 0xe (the rocket
     pool) caps carried at **4** and the launcher's clip at clip_size 4 [D ammo_data +0x02
     and weapon_data record 0x2c, both from the ACTION.ELF static initialiser; probed by a
     temporary test this session]. The remaining box and the ride's launcher print
     `[take] → None` every tick from 24600 to the death.
   - so the pool entering the Hind fight is **hard-capped at 8 rockets** (clip 4 + carried
     4) — the same 8 the bleed-4 roll carried. "The pool must grow" is not reachable without
     breaking the game's own inventory rules, which the task forbids. The ledger lever (1)'s
     pickup half is exhausted; this is now recorded in the ignore message.

2. **The "hurt volume" at the terminal does not exist.** The −2.0/−1.69 world rows the
   ledger's lever (1)/(3) chased are the **Hind's cannon rounds**: their owner index sits
   past the drone list (the copter is not in `drones.list`), which the walker's zap class
   reports as "world". The level's only 0xE4 hurt placements are the kill-floor quads #42-44
   (type 6, 1000, who 0x18 [D — the placement probe run-bleed5-1, matching
   docs/systems/hurt-volumes.md]). This re-dates the whole "volume law" reading in the
   bleed-4 ledger: the −2.0/6-tick rows on the anchor, the east band and the south band were
   cannon rounds reaching through the west doorway's z band (−178.25..−180.6), not a volume.

3. **The ledger's lever (2), "one gate's relaxation": TAKEN — the d3 launch distance gate
   16 → 10.** `walkreal_07000008_live.rs`'s `ready` condition now fires the rocket at the
   sisters' own l ≥ 10 (the blast rule: radius 8 ≈100, falloff past k=8 [L ee_start_05.bin
   @0x2bf150 + 0x2C·0x10C; M Explode_Propagate 0x17c518 case 0]) with every other d3
   self-blast gate kept (the 3 u slit, both 12 u corridors, the eye sight, the machine lane,
   the flight-time forecast). The sisters' expression is unchanged — the old code returned
   10.0 below difficulty 3 — so their timelines stay byte-exact.
   MEASURED (run-bleed5-2, full real run from the load): rocket 1 launched at fight t4907 and
   LANDED 100.7 (the Hind 598). The death stayed t34720: the cannon riddled the anchor (46
   hits for 52.0) and the second launch never opened — with distance no longer binding, the
   lane/sight gates bind (the machine's circuit dist 20-42, its lintel-clear low passes rare).
   No regression: the death tick, position and hit ledger match the master roll (t34720).

4. **The north-strip anchor variant: TAKEN, measured WORSE, NOT shipped.** The ledger's
   lever (3) asked for a spot off the doorway's lane: anchor [112.3, −177.6] (the strip
   between the wall band z −176.6 and the doorway edge −178.25). MEASURED (run-bleed5-3):
   the cannon stopped (no −2.0 rows) but the machine's missiles chase the strip —
   −5.94/−13.16/−15.00/−15.00 blast rows — death **t31593**, earlier than the anchor's
   t34720. Reverted; the measured rationale stays as a comment on the ANCHOR line.

## How it was verified

- **walkreal_07000008_live whole (all 6 tests, --test-threads=1 --include-ignored): 5 passed,
  1 failed** — `walk_the_whole_mission_on_real_input` (d1), `..._real_damage` (d1 real-damage),
  `..._real_damage_agent` (d2), `the_ally_fights_only_the_guards_through_the_noise_path`,
  `the_ally_never_shoots_bond_on_the_walk` all green (the d3-only changes are byte-exact for
  them by construction and by run); only the task's target
  `walk_the_whole_mission_on_real_input_real_damage_00agent` is red at its known t34720 death.
- **interactables_07000008_live: 15 passed, 0 failed.**
- No code under `crates/*/src/` changed this session (test file only), so no other walk or
  replay binary can shift; the full suite did not fit this sandbox's 9.9 GB disk (1111
  nf-view test binaries; the build filled the disk at ~60% and was restarted clean twice —
  see "What's left"). The two binaries the task names were run whole.
- Full suite (`tools/test/suite.py`): **NOT RUN** — the sandbox background-executes nothing
  (detached processes die between tool calls) and the 4-crate release build with all test
  binaries exceeded the disk. The per-binary runs above are the recorded evidence; the next
  session should run the fast suite on the merged tree before its push.

## What's left (the next lever, also in the ignore message)

The cannon class: the rounds come down through the west doorway (x 103 → 111, y 16 → 14 [L
the round log]) and the doorway's z band covers the anchor. The dodge already holds fire
while strafing; the next lever biases the d3 dodge to step OUT of the doorway band on a
cannon zap (the anchor walk stays for drift) while the anchor remains the firing spot, and
measures whether the machine's cannon cadence pauses while its missiles fly.

**Live checks for the main agent** (PCSX2, fresh boot of 07000008, 00 Agent):
1. Pin the cannon rounds' impact band inside the cabin at the anchor: trace the machine's
   gun rounds' hit positions while Bond stands at (112.5, 13.5, −179.4) crouched — expect
   the −2.0 rows to land inside the doorway's z band (−178.25..−180.6) east of the doorway.
2. Whether the machine's cannon cadence pauses while a missile volley flies (the port's
   burst cadence kept the dodge held through the bleed-4 drain; the game may interleave
   differently).

## Usage

not visible to me — this session's environment shows no token, credit, cost or context-window
figures (no such display exists in the sandbox), so there are no numbers to copy.

Main 11-14 (on 6a9463509): on a cannon zap, walk to a spot OUT of the doorway band instead of the anchor: south (112.8, -181.4) t33275 (1 launch); north (112.5, -177.9) t32271 (2 launches, the first landing 101.0, the second 0) - both worse than t34720. Not shipped.

Main 11-14 (on f3c20899a): the sisters launch gates on the 00 Agent fight (slit / corridor / effective-corridor / forecast relaxed, the sight and lane gates kept): rocket 1 at fight t1174 never lands (0.0) and the death comes at t28440 - worse than t34720. Not shipped.

Main 11-14 (on f3c20899a) - CORRECTION to the pool-cap note: the car carries three live 0x2c pickups through the whole fight at (112.0, 18.0, -179.4) (two ammo boxes + the launcher, 4 rounds each, respawn 65535), 4.6-5.0 u from Bond at the anchor - IN take range. The pool is capped only while carried sits at 4: a reload after a launch (clip 3, carried 4 -> clip 4, carried 3) frees room and a box tops carried back to 4 (NF_PICKUPLOG [take] tick 32450 Ammo rounds 1). So reloading after each launch keeps ~12 more rockets available; the binding constraint is the launch windows (one launch in ~7000 ticks), not ammo. Death unchanged t34720 with the reload.

Main 11-14 (merged 4f3802517): per-gate tally over the fight: dodge_hold > 0 on 5519 of ~7000 ticks (the cannon rounds), corridor 6186, eff-corridor 6084, eye 5619, lane 6754, fore 6102, slit 3443 fail counts. The cannon no longer starts a dodge (missiles still do) and the d3-only slit/corridor/effective-corridor/forecast gates are off: three rockets land, the Hind 100.7 -> 287.7 of 598, death t30165 to the Hind missile blasts (25.1/10.9/13.1) with one rocket in the clip. A reload after each launch (the car boxes refill) did not add a fourth launch (t29203). Next: survive the missiles longer.

Main 11-14 (on 4f3802517): the fight standing (no crouch press) with the new dodge/gates: no rocket lands, t27690 - the crouch stays.

Main 11-14 (on 4f3802517): a wider missile dodge (miss < 3.0, hold 36): one rocket, t28214; no dodge at all at d3: no rocket lands, t31416 - both worse than the merged three-rocket stream.

Main 11-14 (on 4f3802517): the pre-fight bill is 44.0 in t0-2000, nearly all at the east deck head (t684-1114, 40.0 to #3/#14/#11/#2, 0xa, 4-10 u) - Bond arrives there with the gun at 1+0 (weapon 0x1, resupply false) and the return-fire stands carry the fight. Crouching from the east deck head leg reroutes the deck-line walk through the hall door (140.7, -96.8) and wedges (6000-tick cap). Not shipped. Lead: arrive at the east deck armed (the start loadout reads dry there).

Main 11-14 (on 4f3802517): visibility map after the third rocket (fight t1400-2700, 44 samples every 30 ticks, a 21x21 grid at 0.5 u around the anchor on the 13.1 floor, eye +1.1, segment_flight_scoped to the Hind position): NO floor spot in the car sees the Hind in any sample - the Hind is out of view from the whole car in that stretch while its missiles still arrive. A second firing spot inside the car cannot work; the walk must survive until the circuit returns (or leave the car - not on the route).
