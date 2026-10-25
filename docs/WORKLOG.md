
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
