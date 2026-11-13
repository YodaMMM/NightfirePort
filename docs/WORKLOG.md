
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
