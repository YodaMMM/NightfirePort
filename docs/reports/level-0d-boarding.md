# level-0d-boarding — report

Task: settle the last placement in `walkreal_0700000d_live::walk_arrival_to_the_c5_door` — the
boarding of SP #115's gantry (the ride west and the drop into the yard are real input now) —
from the data [D the gantry's script-entity collision per tick of the 3000-tick shuttle; the
roof/catwalk edges it passes; M Control_InheritVelocity 0x133e88 / Player_FeetOnPoint
0x194188], walk it on real input and delete the write; if only a live run can settle it, pin
the exact question with a sister. Branch `s/level-0d-boarding` from origin/master (81da6c47,
batch-1031l — master did not move during the task; the branch is master + test-file changes
only, no source changes).

## Result: the boarding is NOT settle-able from the port's data — every route the statics and
## the script offer is now closed with evidence, and the exact question is pinned with new
## sister asserts. The one pos write remains the documented stand-in ([S] unverified); the
## ride and the drop after it are the game's own rules (real input).

## 1. What this task added to the closure (all measured this window, `probe_crane21.rs`)

- **The canopy closure (the "roof/catwalk edges it passes" half of the hint).** The 18.6
  canopy roof spans the gantry's WHOLE swept footprint: probe21's roof map (x 130..190 ×
  z −270..−305, columns from y 22 down) reads a roof surface first on every column — the flat
  18.6 canopy, with the 20.2 rooftop structures (x 158..190, z −270..−291, with their own
  slot at x 175..179 opening down onto the canopy) over its east part. The canopy's edges are
  clear of the footprint: the north edge is z ≈ −267 (the link rows: 18.6 at z −267, the
  12.1 walkway / open at z −266), the west edge x ≈ 130 — the footprint is x 133.4..181.4 ×
  z −270.1..−304.1. So no roof edge and no hole ever passes over the ride: no walk-off or
  fall from above can land on the gantry. The yard catwalk hypothesis is dead too: the
  catwalk band scan (z −282..−293 × x 133..194) found only the yard (3.98), the parked gantry
  (14.32/13.47) and TWO runway pillars — columns at x 154 and x 169 topping at 16.07 (1.75+
  above the gantry top, portal openings for the shuttle, themselves unreachable — no ladder,
  no steps, 12 m above the yard).
- **The script closure (the boarding is not scripted).** 0700000d.bin's script 0x060009EF
  decoded [D the ops; nf_formats Nis::parse]: nine streams — stream 0 End(3000); stream 1 the
  gantry entity (glist 0x0200_0a80, type 31) keyed every 10 ticks over the 3000; stream 2 the
  riding trigger (glist 0x0200_0a81, type 0xEA, params [103, 8]); stream 3 Event kind 3
  params [0]; streams 7/8 the crane-motor sounds. NO event 4 (`Player_Disable`) and NO event
  6 (`Player_Enable`) anywhere [M Script_EventHandler 0x1d94c0 cases 4/6 — the only two ops
  that move the player; its event 3 with p0 = 0 is the negative-delay hold
  (`SCRIPTINFO+0x9C = −1`, already modelled in nis.rs)]. The game does not script the player
  onto the gantry.
- **The aboard detector (the game expects a physical boarding).** The riding 0x0200_0a81 is
  a 2-tri FLAT plane at local +2.95 (absolute y 14.28), x ±2.44, z ±16.80 — a 0xEA trigger
  with out event 103 (= ch 0x67) and mask 8 (= player walk-in) [D stream 2 params; the shape
  bounds via `sim.world.shape`]. Its plane sits at gantry-deck height: it can only fire for
  a body already at deck level, so the game's own data says the player boards physically —
  and level 0d has a SECOND copy of the same detector as placement #116
  `ShippingCrane_TouchPlate` (kind 0xEA, hash 0x0200_0a81, translation (0,0,0) — following
  the mover, out ch 0) [D the placements dump]. No trigger placement volume exists at
  gantry height anywhere along the path (the only 0xEA/0xEB volumes near the yard are the
  ST-3 plate #378 at (138, 8.5, −308.9) and the display trench #264 — both at floor level).
- **The per-tick sweep (the "script-entity collision per tick" half of the hint).**
  probe21_sweep armed SP #115 (the real E on #460 from the bay) and sampled the gantry's
  collision every 8 ticks for the whole shuttle: `segment_hit` tracks the moving shape live
  (the 13.47 girder-top reading travels with the origin through x 139 → 141 → … → 175; the
  14.32 west-deck reading at local −2.54 confirmed at ox 141.5), and the top under the canopy
  band never leaves 13.47..14.32. The live-collision model the ride asserts already depend on
  is verified per tick.

Together with level-0d-beam-boarding's closure (the east ledge 13.76 / pad 14.7 chain 2.6 m
from the gantry's max east face 181.4 with the gantry top ABOVE the ledge top and the jump
vertical-only [M 0x1916b0]; the bay floor 4 m under the ride floor; the ladder census), every
route the port's data offers is closed. The exact question goes to the live check (§4).

## 2. What the test now pins (`walkreal_0700000d_live::walk_arrival_to_the_c5_door`)

The boarding-question block gained three sister assert groups (all [D]-backed, all failing on
the wrong result):

1. the canopy closure — six columns over the swept footprint must read a roof surface
   (17.5..21.0) with no hole down to 15.0;
2. the script sister — `boarding_script_ops()` parses 0700000d.bin from the disc, decodes
   script 0x060009EF and asserts (a) no player hand-off event 4/6, (b) stream 2's entity is
   the 0xEA trigger 0x0200_0a81 with params [103, 8];
3. the detector's geometry — the riding shape is exactly the 2-tri flat plane at +2.95,
   x ±2.44, z ±16.80.

The pre-existing east-side closure asserts (ledge/pad/gap) and the east-extreme assert are
unchanged. The ONE pos write stands, marked [S] unverified in the test; everything after it
(the carry ride to the west apex, the jump-drift drop through the open slot, the yard legs,
the photograph, the C5 kill, the mission hand-off to 0x730000d) is real input.

No port source changed: the closure is a data finding, not a code gap.

## 3. Verification (what was run, every total)

- `walkreal_0700000d_live`: **1 passed / 0 failed / 0 ignored (37.3 s)** with the new sisters.
- `interactables_0700000d_live`: **11 passed / 0 failed / 0 ignored (29.8 s)**.
- `nf-game` (lib + all integration binaries, `-j2 --test-threads=1`): ran clean through the
  alphabetical binaries in the window (ai_live 37/0, anim_events_live 4/0/1 …); the full
  per-binary tally for the crate is inside the chunked-suite state below (the suite runs the
  same binaries; nf-game's slice of the 307 recorded binaries passed 0-failed except the two
  TIMEOUTs noted there).
- **The full suite (tools/test/chunk_suite.py, the fast runner, resumable state
  `/home/z/suite_0d_state.json`): 307/997 binaries recorded — 949 passed, 2 failed, 79
  ignored.** The two "failed" are TIMEOUTs (rc 124) at the runner's 100 s per-binary limit,
  both the sandbox's known class, not code failures: `net_soak_faults_live` and
  `levels_sweep_live` (the levels sweep loads every level on the disc — it needs minutes).
  The remaining 690 binaries did not BUILD in this sandbox: the workspace's full test-binary
  set needs more disk than the sandbox has free (the build filled the disk at 95%; deps
  alone is 4.7 G). The state file resumes the run on a bigger sandbox. The named binaries
  above were run by hand on the final tree and are green.
- `cargo test --offline --release -p nf-view --test walkreal_0700000d_live /
  --test interactables_0700000d_live` and the chunked runner were the only test invocations;
  `probe_crane21.rs` (three `--ignored` probes) is committed as the evidence generator.

## 4. Live checks for the main agent

1. **THE BOARDING QUESTION (the exact one).** Boot 0700000d FRESH (a save-state resume skips
   the boot path). Walk Bond to the bay and E the touchplate #460 (189.02, 9.72, −288.92)
   [D] to arm SP #115 (script 0x060009EF, 3000 ticks [D]). With gdbwatch, watch the player
   object's position writes (and `Player_FeetOnPoint` 0x194188's ground reads) for the whole
   shuttle while Bond stands first on the east ledge (184, ~14.8, −272) then on the pad
   (188, ~15.7, −272). QUESTION: from where does the real game's player physically reach the
   gantry top (walkable 13.42–14.32, swept x 137.54↔177.22 at z −270.1..−304.1)? The port's
   data closes every route (§1) — the likely answers to check: (a) a collision surface the
   port skips (a glist with flag bit 0x10 "not drawn" that still collides in the game —
   diff the port's built collision tri count in the boarding region against the disc's
   glist tri count), or (b) the riding 0x0200_0a81 plane being SOLID in the game (the port
   treats 0xEA as trigger-only) — if the game collides the trolley plane, the player walks
   onto IT from the pad (the plane's east edge reaches x 179.66 at the east extreme, still
   4.3 m from the ledge — so (a) first).
2. **The aboard detector.** With fieldprobe, place Bond on the west girder at the west park
   (135.05, 14.55, −280) and read the trolley trigger's state (the 0xEA entity of script
   0x060009EF) plus channel 0x67 for ~60 ticks: expect the walk-in to latch (mask 8) and
   ch 0x67 to rise — confirming the detector fires from a gantry-deck body and pinning the
   ride's "aboard" state the port could assert later.

## 5. What's left

- The boarding itself needs the live answer (§4.1); then the stand-in write can be deleted
  and the walkreal boarding leg becomes real input (the walk-off/fall onto whatever surface
  the game offers).
- The suite's remaining 690 binaries (disk-bound here; the state file resumes).
- ch 0x67 (the detector's out) has no reader on this level [D the script/placement scans] —
  if the live run shows the game using it (e.g. the crane buttons' state, the MusicTrigger),
  note it for the docs' channel table.

## Usage

not visible to me (no token/credit/context-window figures are exposed in this environment).
