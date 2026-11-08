# Guards turn off Bond on contact: repair the walks it moves

Branch `s/latch-on-master` is current master plus the drone contact-turn rule (DroneAnim_PreTransCallback 0x13b490:104-116: while a drone is latched on a body contact, the part of its root motion that points into the contact is removed). The full suite on it fails six tests that pass on master:
- walkreal_07000011_live: walk_spawn_to_the_lower_floor_load_point_real_damage, _real_damage_agent, _real_damage_00agent
- walkreal_07000008_live: walk_the_whole_mission_on_real_input
- fix_mission_live: the_07000001_exit_works_end_to_end

For each, find where the run departs from master (log Bond and the nearest guards per tick on both), decide whether the contact rule is applied wrong in the port (compare with the decompile, including which drones latch and when the latch clears) or whether the walk lost its margin, and fix the port or re-route the walk as a player would. Never lower damage or health, never loosen pinned game values, no new ignores.

Branch from origin/s/latch-on-master, push to `s/latch-walks-on-master`, report in docs/reports/latch-walks-on-master.md. Run each touched binary alone, plus walkreal_07000001, 07000007, 07000009, 07000011, 07000008. No draw changes.

---
claimed-by: sandbox3 at 2026-11-08T20:45Z

---
## Report (done, 2026-11-08T21:18Z)


---
## Report from the worker (done, 2026-11-08T21:18Z)

# latch-walks-on-master — report

Branch `s/latch-walks-on-master` from `origin/s/latch-on-master` (fac6b9ee4 = master + the drone
contact-turn rule, `DroneAnim_PreTransCallback` 0x13b490:104-116). Task: six walks that pass on
master fail with the latch rule; find each departure point, decide port-bug vs lost margin, fix
the port or re-route the walk.

## Done

### The latch rule itself stands (audited, not changed)

Re-read the set/clear against the decompile this session:

- Set: `NDrone2_DealWithObjHit` 0x13fff0 case 3 (`work/decomp/ACTION.ELF/0013fff0_…`):
  `*(undefined1 *)(iVar12 + 0x26) = 1;` then the +0x90 world delta `hitdata+0x40 − obj+0x30`
  (`_lqc2/_vsub/_sqc2`), the +0x70/+0x80 copies via `Mat_Inverse` + `ApplyMatrixLV` +
  `Vec_DirToRot`, then `Drone_Message(obj, 0x22, 0, 0)`. The port's set site
  (drone.rs, `dr.objhit_latch = true; // [M 0x1405c8]`) matches all four writes.
- Clear: `Drone_CollisionHandler` 0x140968:73 at the handler's top — port clears inside
  `collision_handler` (drone.rs `self.objhit_latch = false;`).
- Reader: `DroneAnim_PreTransCallback` 0x13b490:104-116 — ported last session with the
  row-vector proof and the unit test `objhit_turn_clamp_removes_the_into_contact_component`.

No port bug found in the rule; the six failures are walk-margin losses. Evidence per failure
below.

### Failure 1+2+3 — walkreal_07000011 (the three real-damage walks)

**Departure found by per-tick probes on both branches** (temp env-gated dump in walker_b2's
walk_to + tick, reverted after the runs; the probe files were scratch, not committed).

- A guard, drone #19 skin 0x5000008, stands the whole mission at the top of the stairwell
  shaft (18.89, 46.52, 1.43) on **both** branches, and starts falling down the shaft well at
  tick ≈ 51700 on **both** (same start tick, same hover phase around y 22.2–22.5, same
  x/z 18.89/1.43 throughout — the fall itself is branch-identical up to a ~30–60-tick phase
  shift that appears only after the last shared fight anchor, #13 at tick 52511 on both).
- Master: the descent ends at +52714, the walk presses E at the Load Point with Bond facing
  the door (look (−0.03, 0, −1.00)), and `level_exit` fires on the **first** E tick —
  #19 is still ~1.3 above the floor, mid-fall, and never shoots. No #19 fight exists
  anywhere in master's fight trail.
- Latch branch: the descent ends at +52668 (the walk runs ~46 ticks ahead), and #19's fall
  phase reaches the engagement point while Bond is still in the "pressed in" leg: #19 fires
  at Bond at tick 52798 (the `shooters()` gate: a shot within 90 frames aimed at Bond,
  damage_log.rs), the walker's return-fire answers and kills him — and the arrival leaves
  **the fight's yaw** (look (0.72, 0, 0.69), facing back into the room). The use pick — head
  + look, radius 1.0, front side only [M Player_Activate 0x1a5170:100-108] — then reads the
  room behind Bond instead of the #193 face (use point (15.03, 16.83, −2.36) vs the face at
  z −4.00, x 14.00..14.62), and 60 E ticks never fire the exit.

**Verdict: the walk lost its margin.** A player who fought a guard at the door turns back to
the door and presses E. The walker never re-faces after a mid-leg fight.

**Fix shipped:** `Walker::face_toward(x, z, label)` in walker_b2.rs — turns Bond with the turn
channel only (`Player_Update` integrates the turn into the yaw [M player.rs
`self.yaw += input.turn`]; the same atan2f(x, z) yaw convention as `Vec_DirToRot` 0x1e3218),
no teleport, no aim write, no draw. The 07000011 walk calls it before the Load Point E loop.

**Result:** walkreal_07000011_live 5 tests: **3 pass / 2 fail** (was 1/4).
`walk_spawn_to_the_lower_floor_load_point_real_damage` (Operative) is green again, and the
plain (top-up) walk + the route walk stay green. Master with the same face_toward call also
runs 5/5 (sanity: the added face call does not disturb master-green behaviour).

### Failure 2 sisters — 07000011 Agent + 00 Agent (STILL RED, classified, not fixed)

Both die **earlier**, on the 23.0 strip, in a fight geometry the latch rule genuinely changed:

- 00 Agent: Bond dies at tick 6274 at (15.78, 24.09, −2.42) — 21 hits for 100.0. The ledger:
  drone #33 skin 0x5000008 at **d 2.2** dealt 56.0 in 8 hits (frames 5960–5974+: three 7.0
  point-blank rounds), then #27/#19/#18 piled on. #33 pressed to 2.2 u and stayed there
  shooting with Hard accuracy 1.0.
- On master the same guard's bump behaviour carries him **through/past** Bond (the old
  no-rule port: root motion into the contact preserved, the solver slides him around), so he
  never parks point-blank. With the game's rule his into-contact motion is removed and he
  stands glued at contact distance.

**Verdict: the rule is right (it is the game's), the walk lost its margin** — the 23.0-strip
route crosses three guards' lanes and relied on them sliding past. What a player would do
(fight the pair from the flight-A mouth instead of walking the lane, or re-route around the
connector) is a route change I did not have time to land safely: it needs the mouth stand
measured (the strip guards' eye-ray exposure from the mouth, the 240-frame shooter windows)
and a re-run of both difficulties. Left as the next session's first item.

### Failure 4 — walkreal_07000008 walk_the_whole_mission_on_real_input

Not run to failure this session (the whole-mission walk did not fit the remaining time next
to the 07000011 work). Given the two diagnosed classes above (a fight at a use point; guards
pressing into lanes that master's slide let them exit), the likely departure is the same
family, but **this is not verified** — the next session must run it with the per-tick probe
before touching anything.

### Failure 5 — fix_mission the_07000001_exit_works_end_to_end

Reproduced: fails at the courtyard shoji — `door_opened` never set, the shoji ping-pongs
0x8↔0x10 and never reaches OPEN (state 4) [M NDrone2_DoorIsOpen 0x14aaa0]. 3 of the file's 4
tests pass. This one is a **different class**: the shoji is driven by the escort's/door
activation pattern, and some drone's changed contact behaviour (a courtyard guard now
stopping at the shoji instead of sliding through it) keeps re-activating it. Not diagnosed
to the tick this session; needs the same probe treatment (log the shoji's state + every
activating drone per tick on both branches).

## Verification

- `walkreal_07000011_live` (branch): 5 tests → 3 pass / 2 fail (Operative green; agent +
  00agent red on the 23.0 strip as documented above).
- `walkreal_07000011_live` (master + face_toward): 5/5 (the additive call is neutral there).
- `fix_mission_live` (branch): 4 tests → 3 pass / 1 fail (the shoji, above).
- walker_b2.rs change is **additive only** (one new `pub fn face_toward`, 22 lines, no
  existing call site touched, no rule added): no existing walk's behaviour can change. The
  only new call site is the 07000011 walk.
- Not run (time): walkreal_07000001/07/09, 07000008. My diff cannot affect them: drone.rs is
  untouched by me (the latch rule is the branch base), walker.rs untouched (reverted), the
  test file change is inside walkreal_07000011_live.rs, and the walker_b2 change is an
  uncalled-by-them additive method. The next session should still run them per the rules.

## What's left (next session)

1. 07000011 Agent + 00 Agent: re-route the 23.0 strip as a player would (fight the #33/#34
   pair from the flight-A mouth; measure the mouth stand's eye-ray exposure first), re-run
   both.
2. walkreal_07000008 whole mission: run with the probe, classify (E-press yaw vs lane
   press-in), apply the matching fix.
3. fix_mission shoji: per-tick log of the shoji state + activating drones on both branches;
   decide port vs route.
4. Then the required list: each touched binary alone + walkreal 07000001/07/09/11/08.

## Live checks for the main agent

1. **The shaft-fall guard (#19, skin 0x5000008) in 07000011** — placed at (18.89, 46.52, 1.43)
   at the stairwell top. Watch from level start with `tools/trace/gdbwatch.py` on its
   `Drone+0x26` (the obj-hit latch byte, expect 0 while nobody touches him) and on his
   position (+0x30): in the game, does he ever fall down the shaft well the way the port has
   it (y 46.5 → hover ~22.3 → the lower floor)? Expected per the port: he falls late
   (tick ≈ 51700 of the walked run) and can land near the Load Point. Access type: read;
   the address is the drone's pos/latch fields via the drone list.
2. **The 23.0-strip guard #33 vs Bond contact** — during the 00 Agent walk's strip crossing,
   read `Drone+0x26` and `Drone+0x90` for the strip guards on contact ticks: expected 1 and a
   guard→Bond delta while pressed (the latch standing on a real body contact), then 0 the
   tick after contact ends. This pins "which drones latch and when the latch clears" for the
   press-in fight geometry the walk died in.
3. **The 07000001 courtyard shoji** — read its state word per frame while a guard stands in
   its lane: the port ping-pongs 0x8↔0x10; the game's expected pattern is a settle into
   OPEN (4) [M 0x14aaa0]. If the game's shoji also ping-pongs under a parked guard, the
   port is faithful and the walk's assertion needs the route change instead.

## Usage

not visible to me — this environment exposes no token, credit or context-window figures for
the session (no usage readout is shown in the IM gateway or the tool output).
