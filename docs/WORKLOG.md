
## real-health-00agent-01-hall
Started 2026-11-11 00:51 UTC. Branch s/real-health-00agent-01-hall cut from origin/master 784ba2d99.
Task: un-ignore walkreal_07000001_live::
walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_00agent — Bond reaches the
AND(0x11,0x12,0x13) hall gate at 85.6 hp but the door phase is ammo-exhausted (the pre-hall
stock already consumed the off-path boxes). Plan the ammo: restock on the route the level
offers [D the pickup placements] before the hall, take the three named guards with head shots
from the aperture. Real input only, no top-ups, never lower damage or health; helpers gated to
07000001 and difficulty >= 3. Run walkreal_07000001_live and interactables_07000001_live.

Work log:
- (00:51) branch created from origin/master 784ba2d99, WORKLOG entry, checkpoint pushed.
  Next: read the ignore message and the walker/damage helpers, then the route + pickups [D].
- (00:55-01:05) reproduced the d3 walk on the new base (784ba2d99): the stream moved again —
  Bond died at tick 6709 to the hall captain's volley after the unconditional stock's corridor
  legs took 51.0 (guards 13/0/8). Read the full ignore + the drain-4 detour machinery.
- (01:05-02:00) five d3-gated levers landed (the hoard stock, the pre-clear diet,
  d3_switch_to_deepest_gun, the reload arg in the inline loops, the crouch + the hold-line
  fight; the threshold stand retired; the flank refused for across guards). 15 instrumented
  runs ([L] traces in the report): the walk now reaches the door phase armed and unhurt there,
  but the deep captain's fire lane out-trades the hold line (his hp 24→16, Bond 51→0).
- (02:05-02:15) two un-gated edits (the lure aim, the reload handling) shifted the d1/d2
  streams — the d1 plate walk died; gated both to d3 and re-verified the base parity [L the
  stash A/B]: the whole walkreal_07000001_live binary 7 passed 1 ignored (the d3 ledger),
  interactables_07000001_live 10/10.
- (02:15-02:26) the ledger pushed into the ignore (the REMAINING WALL + the next lever: the
  west-corridor pre-clear before the meet walk); the fast suite died early on the sandbox
  memory (0-byte suite.out) — the explicit batteries are the coverage; the report written and
  the branch pushed.

## real-health-00agent-01-hall-2
Started 2026-11-11 07:12 UTC. Branch s/real-health-00agent-01-hall-2 cut from origin/s/real-health-00agent-01-hall (4f76e3fb90) and merged with origin/master (already up to date).
Task: continue the 00 Agent hall walk from the prior ledger's next lever — the west-corridor
pre-clear before the meet walk. Helpers stay gated to 07000001 + difficulty; real input only,
no top-ups, never lower damage or health. Remove the ignore when green; else push ledger+lever.
- (07:12) setup: clone, branch, WORKLOG entry, checkpoint pushed. Next: read the prior report
  and the 00 Agent ignore message; downloads (work-slim, ISO) next.

## real-health-00agent-01-hall-2
Started 2026-11-11 07:12 UTC. Branch s/real-health-00agent-01-hall-2 cut from origin/s/real-health-00agent-01-hall (4f76e3fb90) and merged with origin/master (already up to date).
Task: continue the 00 Agent hall walk from the prior ledger's next lever — the west-corridor
pre-clear before the meet walk. Helpers stay gated to 07000001 + difficulty; real input only,
no top-ups, never lower damage or health. Remove the ignore when green; else push ledger+lever.
- (07:12) setup: clone, branch, WORKLOG entry, checkpoint pushed. Next: read the prior report
  and the 00 Agent ignore message; downloads (work-slim, ISO) next.
- (08:28-08:40) sandbox kept the repo; fixed the ALSA dev-symlink for the linker (the extracted
  libasound.so pointed at a missing .2.0.0 — relinked to the system libasound.so.2, RUSTFLAGS
  -L the prefix). Probe dump [L]: the d3 roster's live guards after the intro are exactly the
  ledger's 13/0/8 (#0 0x5000036 at (-7.8,-5.9), #8/#13 0x5000039 SMG at (-10.1,-0.3)/(-1.0,-0.2)).
- (08:40-08:50) repro on the branch base: the base d3 walk died at tick 13546 at the door phase,
  49 hp left — the ledger's numbers exactly (#13 25.2 in 14 + #0 24.0 in 2 + #8 1.8 [L the
  repro trace]).
- (08:50-09:35) the pre-clear through 4 measured iterations: run 1 (return-fire holds in the
  open, 65.4), run 2 (nav stopped 1 u short of the stance, 34.2 to #13), run 3 (bullet-lane
  grid lie — sight rays don't stop bullets; the #8 fight took 50.4 from the open stance), run 4
  (per-guard stances: #0/#13 from the west band, #8 from the spawn-area south where the base
  repro's whole walk took ONE #8 round) — 45.0 total, #8 free, and ZERO damage from the
  corridor to the door phase (55.0 hp at the door vs the base's 49.0).
- (09:35-10:05) the hall phase: run 5 died to the return-fire flank INTO the hall — added the
  opt-in Walker::flank_hold_z (a guard across the aperture never takes a flank nav; None by
  default, the d3 hall phase sets 11.2, cleared after the AND). Runs 6-8: the AND crowd won't
  die — the sustained lures ran with full pools and drew NO crossings (alive-across ×3 every
  run) while the crouch fights burned the pool on body taps; d3 now goes lures-first (the
  crouch fight stays d1/d2). The walk survives to the door phase at 55.0 and the named loop
  runs dry — the ledger + the next lever (the door-phase kill path) pushed into the ignore.
- (10:10-10:20) whole walkreal_07000001_live: 7 passed 1 failed (the d3 ledger only; d1/d2
  byte-identical); interactables_07000001_live 10/10. Report written; the fast suite did not
  fit (the sandbox OOM-kills the 4-crate build; 4 GB) — the explicit batteries are the
  coverage, re-run the suite before merging.
Started 2026-11-13 03:12 UTC. Branch s/real-health-00agent-01-hall-3 cut from origin/s/real-health-00agent-01-hall-2 (ec54db62c) and merged with origin/master (merge ca17d3e5b; one conflict in tests/common/walker.rs — hall-2's flank_hold_z field kept alongside master's record_inputs/record_states replay machinery).
Task: the door-phase kill path — the hall-2 ledger's next lever. Either work out which weapon/timing draws the guards across the aperture ([M] DroneFunc_SetNoiseHeard 0x149920, HandleSoundAlerts 0x148a30) and lure with that, or kill the crowd through the aperture with crouched taps along the flight lane, ammo metered. Helpers gated to 07000001 + difficulty 3. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; else push the ledger + next lever.
- (03:12) sandbox kept the repo + work/ + ISO; branch cut, master merged, WORKLOG pushed.
- (03:12-04:00) the door-phase kill path, both arms measured: the LURE arm is dead by the AI
  gate — the crowd is sighted-attacking and +0x4F8 0x4000000 never clears on 07000001 (only
  the 09/0A tower-impact path clears it [M tower_impact_stamp]), so DroneVision_AlertSound
  0x177800 returns 0 for msg SOUND [M enemy_alerts]; the diag run: P2K 0x403 (w65/o40 [L
  SFXOutputData]) 56 taps, SMG 0x12→0x3 (w50/o30) 55, fists 56 — alive-across ×3. The TAP arm
  is dead from the deep line: 0 of 56/55/56 taps landed, the flight lane walled on every
  trigger tick; the lane grid (64 combos, both eye heights [L]) walls the whole middle band
  (x 6-7.5) and the clear west/east bands wedge the nav 1.6-1.7 u short. The crouched peek
  drew no advance in 24 windows (the crouch hides Bond).
- (04:00-04:30) the mechanic that WORKS [L the stance-fight run window 2]: a STANDING walk
  through the door lane draws the crowd's advance — guard 6 died to the walk's own return
  fire at 16518 for 3.6 — but the nav leg lingered and the captain's 5-pellet 36.0 volley
  killed Bond at 16593 (the 75-tick lethal linger). The WIP peek dance + the lane grid + the
  diag counters are on the branch, d3-gated; d1/d2 byte-identical. Ledger + the next lever
  (the advance dance with the hard exchange budget) pushed into the ignore message.
- (04:30-04:36) whole walkreal_07000001_live: 7 passed / 1 failed (the d3 ledger only [L
  99.4s]); interactables_07000001_live 10/0 (29.4s). The fast suite dies between tool calls
  (hall-2's documented artifact) — the batteries are the coverage. Report written.
- (04:05-04:40) the advance dance built and pushed (9b09f06ab): the capped-nav opt-ins
  (Walker::nav_tick_cap, Walker::leg_abort_on_wound, DamageLog::abort_on_wound — None-
  defaulted, pure reads, no RNG draw) + the dance legs. THE MECHANIC CONFIRMED [L dance5]:
  the nav leg through the door OPENS it (7.9,5.3,11.1 state 0x2 at leg 0) and guard 6
  advanced and died to the walk's own return fire at tick 7217 with 55.0 -> 55.0 — the kill
  is FREE when the door is open. The remaining bug: the abort misses the death window (the
  captain's 6-pellet frame 7301 lands 40.6 inside one tick; the die panic fires inside the
  tick before any loop-head abort) — next round wires the abort into the note path or gives
  the dance a no-die-panic window. Battery with the WIP files: walkreal_07000001_live whole
  7 passed / 1 failed (the d3 ledger only [L 88.9s]); interactables_07000001_live 10/0
  (29.9s) — the d1/d2 streams byte-identical through the opt-ins.
- [00:56-? UTC] real-health-00agent-01-hall-4: branch off origin/s/real-health-00agent-01-hall-3 (0a05db489), WORKLOG continued. Task: land the advance-dance lever with the hard exchange budget on the hall door phase, measure it on its own real run, then carry the d3 walk through the AND gate and the rest of the route (post-AND untouched, green on the older streams). Sandbox note: repo persisted in my-project; ISO re-download needed; toolchain reinstalled.
- [00:56-02:15 UTC] real-health-00agent-01-hall-4: the advance-dance lever LANDED on the door phase. The lever set (all d3-gated, None-defaulted): (1) `Walker::leg_abort_on_kill` — the leg ends the first loop head after a listed guard DIES (alive-baselined per leg; the run2 trace showed the un-baselined abort stalling every later leg on g6's corpse) — the 7217 free kill no longer rides into the captain's 84-tick-delayed 40.6 frame [L the dance5 run]; (2) the HARD EXCHANGE BUDGET — never press into the lane from below 45.0 = the captain's worst measured frame 40.6 (the 6-pellet 7301 burst, lands INSIDE one tick before any loop-head abort) + the advancer's volley 3.6 — the old <=15 gate rode exactly that death; (3) the determined pass OFF inside the legs — its shoot_bounded NAV ignores flank_hold_z and walked Bond deep into the hall (the run3 death at 11644: the pass flanked Bond to z 19.4, cost 19.2, the retreat ate #7's frames); (4) the retreats run CROUCHED (`Walker::leg_crouch`) — the peek runs measured crouched Bond in the lane taking 0 fire across 24 windows, and the run1 death showed the home walk's own return fire HOLDS Bond in the lane when the shooter shows no exposed part (he died at the leg-end position, never moving); (5) the d3 dry resupply goes detour-only — the press legs put the hall interior on the walked-path trail, so resupply()'s nearest-on-trail rule picked the hall's own deep box (3.6,17.8) whose lane is owned by #5/#14 (the run8/run10 deaths); (6) the escort loop breaks on a posted exit (the run13 stream: the exit posts mid-resupply, every follow leg no-ops at +0 ticks, the 24 stall rounds expired into the panic). Measured [L runs 1-15]: g6 free kill at 7217 (55.0->55.0), #7 free kill at 9525 (47.8), g11 down to the press finish (a fixed shallow press to z 13.2 + ONE wound-aborted shoot; the deep press at the guard's own z 18.3 target ate the melee + #5's west-lane pellets). THE HALL AND FIRED at 13831; the walk reached the escort at 21.4 hp (12 hits for 78.6). The NEW WALL: the escorted contact (#161, death channel 0x63) died to a PLAYER round in the post-AND resupply's west-corridor crossfire — DroneFunc_OnInitDeath reason 1 -> 0x4000034 'Killed Civilians' -> the fail-restart exit posted at 14551 [M ally.rs on_init_death] — despite the stand fire_shield (Some((ci,true)) since level-06-rook-door-3). The ledger + the next lever (instrument the contact's death: NF_DRONE_WATCH ci + NF_DMG_TRACE; find the unshielded fire path or widen the 0.9+0.05 ds cone; then the west route, the slab ambush, the plate AND-touch) are in the d3 test's ignore message. Battery: walkreal_07000001_live whole 7 passed / 0 failed / 1 ignored (84.6 s); interactables_07000001_live 10 passed / 0 failed (28.9 s). The fast suite was started in the background (suite.py, PID alive) but killed at the lever round: the cargo lock serializes against the lever builds and this 9.9 G disk forked the nf-view fingerprints to ENOSPC twice on prior rounds (the 1b-5 note) — the two whole binaries above are the coverage.
