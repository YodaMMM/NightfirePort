
### level-0a-rain-indoors (started 05:43 UTC, 2026-10-25)

Task: 0700000a — rain streaks fall indoors, in the dark office lounge at Bond
(-37.50, 1.11, -16.95, yaw 2.827, pitch -0.092), frame 2719. Rain belongs outside the windows
only. (1) Find how the game confines its rain [M — the weather/rain emitter's creator and
update, its volume/box or room gate, any indoor/outdoor flag on the room or the camera's room,
the particle cull] and why the port draws it in this room. (2) Fix it [M]; a test that no rain
particle draws with the camera in this room, and a sister that rain still draws outside (0b's
rain stays as it is — the user already checked 0b's rain as right). Headless render of the spot
named in the report.

Plan: branch s/level-0a-rain-indoors from origin/master 38f1bc93; reuse the sandbox's existing
work/ + ISO + nf-queue; fast suite runner started early; census the rain emitter + particle cull
decompiles [M] and the 0a weather setup [D]; fix + tests; report + finish.


- (05:43-06:11) Setup reused (work/ + ISO + nf-queue already present); branch pushed; fast suite
  runner up (needed the alsa pkgconfig prefix + cargo PATH; 792 binaries built in 1489 s).
- (06:05-06:50) [M] rain census: boxes are the only confiner — RainBox_Create 0x18d840 (0x32
  placements), RainBox_Clip 0x18d0d0 (cube ±15, x/z-emptiness), AddDrop 0x18c278 (footprint gate,
  kill = max(box floor, eye−7.5)), UpdateDrops 0x18c580 (early-out, RandTable2 = vel·FRM +
  wind_quad·FRM). DrawDrops 0x18cb90 draws per drop (object pipeline + a darker camera-facing
  pass on 09-0b); Game_Draw 0x1e6f50 calls it per active viewer — no room gate anywhere.
- (06:35-06:55) [D] 0a census (probe_0a_rain/probe_0a_rain2): Env #530 param0=1 (rain), 7 rainboxes
  #791-#797; office cel x −42.44..−10.72 z −38.77..−13.82; the office spot is 4.77 m east of
  rainbox5's east face −42.27. Port's box/clip/gate/kill arithmetic all matched [M] — but the
  step missed InitDrops' DAT_003194f4 = −0.5 for rain [M 0x18bf68]: rain fell at snow speed
  (0.025-0.1/tick), drifted up to ~6 m (pre-fix probe: drop bbox x hi −37.63, inside the office).
- (06:55-07:10) Fix: Drops.fall (DAT_003194f4) folded into the step y [M 0x18bf68/0x18c580].
  Tests: nf-game env unit 6/6 (rain speed + 1.0 m drift bound pinned; snow sister keeps the slow
  fall); nf-view fix_0a_rain_indoors_live 2/2 (no drops x > −41.0 at the reported office state
  through the real load + step_bond; rainbox5/rainbox1 strips stay populated).
- (07:10-07:25) Report + finish. Suite baseline (start tree): 3 disc_golden + identity_disc +
  probe_render_pylon_only FAILED — master's, pre-fix; touched crates run green directly.


## level-0a-rain-indoors-2

Branch `s/level-0a-rain-indoors-2` (origin/master 38f1bc93 merged with
origin/s/level-0a-rain-indoors f7246aaf). Window 08:06-10:05 UTC.

Plan: land the rain fall speed (-0.5 step, already on the rain branch) WITHOUT shifting
other levels' placements/AI. The merged tree turns 7 live tests red; the suspect is the
Drops path consuming the shared random stream differently (faster fall -> more AddDrop
calls -> different Rand() draw count/order -> every later draw shifts). Census the drops
decompiles [M 0x18bf68/0x18c580/0x18c278] for their exact generator + draw order, make the
port draw identically, get all 7 + fix_0a_rain_indoors_live green, full suite on the final
tree merged with origin/master.

Progress:
- (08:08) Setup: repo cloned fresh (sandbox reset), branch created + pushed; work-slim/ISO
  downloads starting.
- (08:45) [M census] The shared generator confirmed: Rand_Rand 0x1e3738 / Rand_Random 0x1e36f0 /
  Rand_FRand 0x1e3780 all step the same SEED_X/SEED_Y (0x30d0a0/4) — drops and AI share one
  stream. Per-frame rain consumers [M]: Env_Update 0x17bc88 draws Rand(20000) unconditionally
  under glb_world, then UpdateDrops; rain draws NO wind draws (0x18c580:139 kind!=1 gate) and
  3 FRands per AddDrop (0x18c278:29-40), capped Added<0x15/frame in slot order (0x18c9fc slti
  disasm). **The port's real bug [M 0x18c580 rain branch]: rain steps by the UNIFORM wind quad
  (`_lqc2(_WindVector2)` + `_vadd`), NOT the per-slot RandTable2 (read only by the snow/rising
  scratchpad walk) — the port applied `step[i & 31]` to rain too, giving every drop a fake
  ±0.02 slant and a vel.y-varying fall (0.525-0.6) instead of the game's exact straight 0.5.
  Different death times -> different respawn counts -> the shared stream shifts on every rain
  level -> the 7 pins break.** [L ee_start snapshots]: CurrentMax is a u16 (0x30d784): 01=0x40,
  0a=0x80, 05=0x1000 — the port's unsigned split is right; InitBodge=1 in the 01/0a dumps is a
  stale save-state field (InitDrops disasm 0x18c1f0-0x18c220 sets it only for 05/06, as the C).
- (08:50) Fix: rain step = uniform [wind0, fall, wind1]; snow/rising keep the per-slot table.
  Unit tests re-pinned: rain y step EXACTLY 0.5, x/z constant (no slant exists). env:: 6/6.

## level-0a-rain-indoors-3

Task: the last two reds from the rain fix (branch starts from s/level-0a-rain-indoors-2 merged with origin/master):
interactables_0700000a_live::armour_pickup_taken_by_walking_over_it and
escort_follow_07000001_live::escort_at_walking_speed_keeps_the_games_distance_and_the_exit_opens.
Find what the shifted random stream changes on each path (which draw, which drone/door/channel decision),
whether the expectation rests on a port-only [S] value; where the walk needs one lucky draw, play it with real
input; where the port's per-frame draw ORDER differs from the game (Env_Update Rand(20000), AI draws, order
within a frame [M 0x17bc88/0x1e36f0..0x1e3780]), fix the order. Then both green with rain at game speed, the
5 recovered tests still green, full suite on the final tree merged with origin/master.

Progress:
- (10:04) Setup: sandbox warm from the urn-break session; branch s/level-0a-rain-indoors-3 created from
  origin/s/level-0a-rain-indoors-2, origin/master merged clean; pushing before work starts.
- (10:44) Both reds green, pushed. Mechanisms:
  - armour: the sim's `level_exit Some(0x0700000A)` (the CURRENT level) is NOT the east-side
    LOAD_LEVEL #59 (next 0x0700000B [D]) and no script event 0x12 fires (NF_EXIT_DBG dump) — it is
    the mission's LevelEnd [M Mission case 4 → ResetMap_LevelToLoad(LevelToEndTo) 0x182c60:73].
    The graph's route to the armour turns north off the z≈5.5 corridor at its node 8; the steering
    wedges there (+1017 ticks) and parks Bond inside the west office's ch-0x15 TOUCH plates
    (#11/#22, x −24.27..−19.96, z 8.89..14.45 [D trigger dump]) — a long stand satisfies the
    mission's end condition. The plates' brief crossings are harmless (out 0x15 feeds the
    door-unlock muxes [D docs]; walkreal-0a's west_doorway_repro walks straight over them, green).
    Fix: the walk takes the AI ring's west-office legs (walkreal-0a's proven anchors), wall-clear
    at body height [probe leg rays]. armour test green 2.12 s.
  - escort: the contact's stream-shifted park at (3.4,6.3,6.6) put Bond's close-through-the-nav
    into the gallery doorway wedge (4-5.5, 11.5) — the slide door's open panel + the wall band at
    z≈11.05, re-closed every 360 quiet ticks by the auto-close [M 0x13660c-0x136700]. Fix
    (player's stick moves, no assert touched): the lateral jog in the stall recovery + walking the
    doorway frame's OPEN east half. escort test green 25.59 s (9533 distance checks, level_exit
    07000002 at tick 53311).
  - No port draw-ORDER divergence found beyond the rain-2 report's alignment [M 0x17bc88 /
    0x18c278 / 0x18c580]; both reds were walk choreography resting on one lucky stream decision
    each. The [S] per-test seed (05-snapshot XOR mix) remains the documented port choice; the
    per-level game-seed rebaseline stays next-session.
- (10:46) Full suite started on the final merged tree (daemon). Recovered binaries spot-run green:
  sight_frustum 2/2, fix_pickups_ammo 4/4, escort_hide_findrun 3/3 (three more pending the suite:
  direct runs OOM-killed under the build's memory pressure).
- (11:37) Cutoff state: suite 232/798 binaries, no total line (projects ~12:46 — past expiry).
  FAILED: bathhouse_shout_mixer (07000002 shout voice window), interactables_07000041 (glass
  pan fire window) — both the rain-2 merge's stream-shift class, first seen on this merged
  tree (that merge's own suite never completed); probe_pylon_render (GPU env red). CRASH ×7
  OOM (net_*/mp_*/walkreal_probe13). The two task binaries pass inside the suite run too.
  Full detail in docs/reports/level-0a-rain-indoors-3.md (Suite appendix).
