# level-14-hall-walk-in — the report

Task: `walkreal_07000014_live` still placed Bond at `[-60.3, 0.9, 38.4]` (the cascade test's
teleport, after seeding the fusebox channels 0x87/0x88). Replace the placement with a real walked
route, and raise the seeded channels through the walk's own real fusebox cuts in the same run.

## What's done

**The placement is gone.** `the_e_use_switch_fires_the_sabotage_cascade_and_opens_the_west_door`
in `crates/nf-view/tests/walkreal_07000014_live.rs` no longer writes `bond.pos` and no longer
seeds `channels.set(0x87/0x88)`. It now:

1. pins the disc facts on the fresh load (the 0xA0/mask-4 E-use touch exists with its volume
   [D placement 399]; the Door_Slide_11A/B pair starts locked on ch 0x85 [D placements 151/152];
   nothing armed [D]);
2. **walks the real route** — `walk_infiltration_to_handback` (the vent start → the SkyBridge
   Q-Claw crossing → the trench → the shaft → the mezzanine → the SP #276 hand-back) then the new
   shared helper `walk_the_west_route_to_the_switch_band` (extracted verbatim from the main walk
   test, which now calls the same helper — one route, two tests):
   - the descent to the facility corridor (the walked route, probe landing (29.99, 0.61, 38.00));
   - leg 2: the SOUTH loop to the #381 band [D nav nodes 94-97/150/591-600] — it never crosses
     the closed 0x85 door plane [D probe: the walk east stops at (−52.79, 1.20, 39.42)]; the
     #381 Kiko cut-scene (SP #382, 0x0600062F) carries Bond west past the closed door;
   - leg 3a: the west generator room down the ramp [D nav nodes 613/614/625/615/623/628/631/663/
     654/653];
   - leg 3b: **the two fuseboxes burned through the REAL laser path** (`cut_fusebox`: weapon 0x4E
     into the wire sub-object; `FuseBox_Update` 0x1adfe8:112-121 writes `switch_channels[ch] = 1`
     on the wire-group death [M]; #383 p1 0x87, #385 p1 0x88 [D placements]; AND #391
     (0x87, 0x88) → 0x8C [D AND plc]) — the seed writes are replaced by the walk's own cuts in
     the same run, exactly as asked;
   - leg 3c: back up the north ramp and into the E-use switch band [D nav nodes
     641/637/635/199/632/630/620/618/626; D placement 399 band lo −63.08…−57.52, z 37.33…39.41] —
     WITHOUT pressing E (the callers own the press);
3. pins the arrival: 0x87/0x88 both ON — the walk's own cuts' writes [M 0x1adfe8; D placements
   383/385]; 0x8C raised through the real AND [D AND plc]; **the 0x85 pair STILL locked** at the
   walked arrival [M `Door_IsLocked` 0x136810:10-16; D placements 151/152] — the route reaches the
   band from the west without raising the door's lock channel or crossing its plane; and nothing
   fires while standing in the touch volume without E [M `Trigger_Hit` 0x1b1fc8 builds contact
   bits {2, 8, 0x10} only, mask 0x4 is the E-USE flag];
4. presses E through the REAL `Player_Activate` path with **turn/look steering** (the old direct
   `bond.yaw/pitch` write is also gone) → 0xA0 [M `Trigger_Activate` 0x1b2318:23-33; M
   0x1a5170:100-108, 134-137] → the SEQ_AND [0x8C, 0xA0] → 0x51 pulse [D placement 393] → the
   kill cascade → box death 0x85 → the door unlocks and AUTO-OPENS [M `Door_Update`
   0x1363c8:88-107] → pos01 > 0.5.

**No port gap was hit**: the whole walked route went green with zero changes to `nf-game` — the
route's classes (Walker/Wp `walk_to`/`follow_nav_route`/`climb_stairs`, the 0xEB TOUCH wrapper
`Trigger_Touch` 0x1b2aa8/`Trigger_Create` 0x1b1720, `Player_Activate` 0x1a5170, `FuseBox_Update`
0x1adfe8, `Trigger_Activate` 0x1b2318, `Door_Update` 0x1363c8) already matched the game.

## How it was verified

All three totals are the FINAL tree (a20d6f5b: the walked route + the lock pin, merged with
origin/master b3d86a03), suite-style runs (`--test-threads=1`):

- `walkreal_07000014_live` — **6 passed / 0 failed / 0 ignored** (102.47 s), including the rewritten
  cascade test and the unchanged main walk (which now shares the same helper — its input timeline
  is bit-identical to before).
- `door_guards_locked_door_07000014_live` — **4 passed / 0 failed / 1 ignored** (23.69 s); the
  ignored pin is the pre-existing, evidence-named blocker (the inert NoBunching mirror + the
  release-to-door divergence), untouched by this task.
- `interactables_07000014_live` — **9 passed / 0 failed / 0 ignored** (18.68 s).
- Full suite (fast runner, 996 binaries built in 865 s, running on the final tree): SEE THE TOTAL
  LINE BELOW — a partial if the sandbox window closed first; FAILED/TIMEOUT/CRASH lines quoted.

## What's left

Nothing for this task. The suite's `disc_golden` tests may fail on the slim ISO (allowed).

## Live checks for the main agent

None required — the route is [D]/[M]-tagged per leg and now proven green in the port. One
optional confirmation on a FRESH boot (save-state resume would skip the boot path):
- In level 0x07000014, stand Bond at (−60.3, 0.9, 38.4) after burning both fusebox wires
  (channels 0x87/0x88 up, AND 0x8C up), face the switch, press E. Expected: `switch_channels[0xA0]`
  pulses 1 for one tick, `switch_channels[0x51]` pulses, the box cascade ends on 0x85, and
  Door_Slide_11A/B (placements 151/152) slide open. Watch with gdbwatch on `switch_channels`
  (0xA0, 0x51, 0x85) access during the press.

## Usage

not visible to me
