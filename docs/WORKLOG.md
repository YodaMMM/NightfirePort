
## level-0a-ring-sweep-timing (started 2026-11-04 09:16 UTC)

Branch s/level-0a-ring-sweep-timing cut from origin/s/trigger-plane-records-2 (9c5406bc), origin/master
(dd78d755) merged. Four tests red, one cause: walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch
and its three real-damage sisters — the laserbeam glists ride key motion (#614 sweeps between z 13.17 and
10.73) and the walk's jump lands in the swept beam, so ch 0x22 fires; a player times the crossing against the
sweep. Task: re-time the ring's crossings with real input (sweep-aware press windows from this branch's
"sweep-watch route experiments" history, or pin the beams' key paths [D] and cross in the measured gaps).
Never weaken the trigger law or the "no laser/alert channel fired" pin. Done = the four green,
low_beam_621_fires_standing, fix_truck_live, interactables_07000011_live green, every trigger/beam/laser test
and every walkreal_*_live green, then the full suite total line.

Round (2026-11-04 ~10:45 UTC): the sweep pinned and the #614 crossing re-built. [D] the #614 key
path: 3 keys, z 13.0287 → 10.27843 → 13.0287, step 1/120 per tick, one pass 240 ticks
(Trigger_Create 0x1b1720:56-87 / Trigger_Update 0x1b1a78:60-70). [L sweep-probe] the live box
parks 12.896..13.173 at t≈224..18 and bottoms 10.146..10.423 at t≈110..148, sliding ≤0.034
u/tick; the standing walk-through stamps 0x22 at EVERY sweep phase (16/16 phases probed — the
walk's monotone z always meets the sweep); the jump arc runs 0.1 u/tick with the feet peaking
1.840, so the standing toucher's segment bottom (B = feet−0.485 [M 0x194df0 default arm], band
top 0.5164 [L]) clears the band by >0.55 for arc ticks k 10..37 — ONE jump crosses the whole
band, its low-y ends (k ≤ 9, k ≥ 38) kept clear of the live box by the takeoff stands: southbound
z ≤ 13.79 behind the arm box_max ≤ 12.0, northbound z ≥ 9.52 behind the arm box_min ≥ 12.7.
Implemented as sweep_jump_beam614 + the three crossings re-wired (the keyboard's southbound,
the walk-back's nav final node 10.6 → 9.0 and the northbound, the ring's entry) in
walkreal_0700000a_live.rs; the merge-collision literals re-fixed (walkreal_07000001_live.rs
Toucher.seg_only, exit_aim_hint_live.rs Trigger.planes). Verified IN-SIM: the #614 crossings
clean — "[0a] outbound past #614: fired []", the ring's sweep-jump clean, ch 0x22 never stamped;
the compile check of the three touched binaries green. The four tests still fail: the failure
MOVED — 0x24 (#621's STATIC band, motion false [L probe2]) now stamps between the #614 crossing
and the ring. Two named candidates: (a) a drone contact — the tick-phase cascade (the arm watch
changes the route's tick count) moved a guard into #621's ankle band (y 0.128..0.392, a guard's
segment reaches pos−1.0); (b) the player's own #621 arc landing inside the band floor's
exclusion (z −34.4 vs the floor −34.138: dz 0.26 < 0.55) — but the same geometry was clean
before, so (a) is likelier. Next lever: watch which toucher writes 0x24 around the #621 legs
(the band legs: walkreal_0700000a "the column north of beam #621" / "jump beam #621
southbound/northbound"), re-time that one leg; the #614 sweep-jump needs no re-tuning.
