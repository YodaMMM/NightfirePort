# level-08-00agent-hind — the bottom-terminal Hind fight's flight law

Branch `s/level-08-00agent-hind`, cut from `origin/s/level-08-00agent-doors` 26c30220 and merged
origin/master cee0f0df (a fast-forward — master already carried the doors work and strips
WORKLOG as a process file). **Status: the flight's GLASS PASS is ported and verified; the 00agent
walk stays ignored one stage later — the bottom-terminal pane shoot-out kills nothing (the
orphaned-pane lead) and the first rocket self-blasts.** The next session continues from the
branch; the ignore message carries the state.

## What was done

### 1. The blocker root-caused: the flight's glass pass is a real port gap

The doors session's lead ("the carriage's collide-glist glass") resolved to a different, precise
law, from the decompile and disc dumps:

- The bullet's FLIGHT query is armed every flying tick [M `Bullet_update` 0x12e7b0, disasm
  0x12f030-0x12f058]: kind 0x101/0x201 at HITTEST+0xe4/$s4+0xe4, the pick mask at +0x98 ZEROED
  (`sh $zero,0xe8($s4)`), and **+0x9a = `(record+0x6C & 0x800 == 0) << 5`** — 0x20 for every gun
  whose record's +0x6C bit 0x800 is clear. The HITTEST base is obj+0xD4 [M `Collide_Update`
  0x1ea398:78-84].
- The contact walk [M `Collide_Update` 0x1ea398:216-229] computes
  `pass = (HITDATA+0x51 bit 1: the obj+0xF0 0x40 glist flag) | (surface byte bit 0x40)`, rolls
  the 0x80 arm, and **when HITTEST+0x9a carries 0x20, a surface byte 0x10 (glass) contact is
  ALWAYS a pass** (`bVar3 = 1` unless the byte differs) — the round continues past the glass
  while the mirrored contact still lands on the glass object's obj+0xD0 chain [M :160-233] and
  `Break_Update` drains it.
- [L weapon_data (ACTION.ELF)]: the guns 0x02/0x03/0x05/0x06 (+0x6c = 0x00040050) and the AT-420
  0x2c (0x00012204) all have bit 0x800 CLEAR; 0x01 (fists) and 0x45 (laser) have it SET.
- [L coll-obj 07000008 map1]: the 16 ride panes (OBJECT_Glass1a/6a 0x02000477/78, DOOR_Glass_1/4
  0x0200037f/0x02000479, Glass10 0x02000382 — the ride script 0x0600004C streams 7-18/23-26,
  params [2,1,0]) carry collision ALL mat 0x10. OBJECT_Carriage (0x0200038a) is flags 0x1000,
  250 tris ALL mat 0x06, bounds (−0.42..0.66, −0.31..1.07, −1.98..1.74) — the hanger box ABOVE
  the cabin (the carriage origin rides 4.5 above the cabin floor), NOT the walls Bond sees; the
  cabin box around Bond is other geometry. Object_Floorgrate (0x02000413) is flags 0x1001.

**The port fix** (all [M]-cited in the code): `segment_flight`/`segment_flight_scoped` in
`crates/nf-game/src/world.rs` — the flight traces drop surface-byte-0x10 hits when the round's
record arms the pass; `Bullet` carries `glass_pass` from the record at every spawn; the
player-bullet flight's three world traces (`tw`, `tstop`, the copter shield) in
`crates/nf-game/src/drone.rs` `update_player_bullets` read it; the 00agent test's d3 slit/corridor
gates read the flight query (d3-gated — the sisters' timelines stay byte-exact).

### 2. The guard-side flight keeps its documented deviation — measured, not assumed

Arming the pass on the guard spawn sites too turned `walk_the_whole_mission_on_real_input_real_damage_agent`
(the d2 Agent walk, previously green) **red at tick 25178**: #23/#19's volleys pass the top-station
glass the platform-mouth stand hides behind — the real-health route relied on the port's old
wrong behavior. The four guard spawn sites keep `glass_pass: false` with the analysis inline;
arming the guard side belongs to the route work that re-covers those stands.

### 3. The 00agent walk was run un-ignored once (the new blocker, measured)

- The pane shoot-out taps 9 rounds (clip 30 → 21) and kills **nothing** — `dead rows []`, every
  pane reads OPEN on the aim ray. The bottom-terminal panes are rows in `breaks.list` but their
  geometry never blocks: the SCRIPT-pane rows (the ride's 16) are dropped when their script
  objects end (`retain_script_panes`), and the PLACED-pane collide arm (`script_parts`'s
  `_pane_parts`, the 0x400_0000_0000 ids) is still OFF ("NOT ridden yet [S]") — so neither source
  puts quads in the collide set at the terminal, and the box-contact kill did not fire either.
- With the rays open, rocket 1 launched (t138, dist 20.9) and the walk died the real-damage death
  shortly after (the self-blast chain the 12 u corridor gate was built to refuse — the gate now
  passes the pane planes but the terminal's wall band beyond it still catches the guided chase).

## How it was verified (the binaries that ran, with their pass counts)

- `nf-game --lib`: **656 passed, 0 failed, 5 ignored** (the glass change's unit surface).
- `walkreal_07000008_live`: **5 passed, 0 failed, 1 ignored** — after the guard-side revert: the
  Operative real-damage walk, the d2 Agent real-damage walk (re-verified green), the topped-up
  walk, and the two ally tests; the 00agent sister carries the updated ignore.
- The other `walkreal_*_live` binaries, `npc_doors_live` and `fix_guard_misc_live` did NOT fit
  this session's window (the sandbox expires ~02:58 UTC); they MUST run before this branch
  merges. The guard-side flight is byte-identical to master (the only sim-side behavior change
  is the player-bullet flight's glass pass and the d3-gated test gates), so the risk is the
  player-fire paths the 08 walk covers.

## What is left (the next session's leads, in order)

1. **Re-source the bottom-terminal panes**: are they placed 0x20 placements (their collide arm
   is the disabled `_pane_parts` — proving that pairing is the doors session's old lead) or
   script panes whose stream re-creates them at the terminal? The shoot-out needs EITHER the
   placed-pane collide arm proven (the [S] shape-key/matrix pairing) or the shoot-out re-targeted.
2. **The guard-side glass pass + route work**: arm `glass_pass` from the record at the four
   guard spawn sites and re-cover the stands that used glass as cover (the 08 d2 platform mouth
   died at 25178 through the top-station glass).
3. **The corridor gate's reach**: with the panes passing, the 12 u gate reads clear through the
   window planes but the first rocket still self-blasted — the gate may need the flight-time
   chase-path term (the machine's bearing behind the terminal's wall band) rather than a longer
   ray.
4. Then the done criterion: the 00agent walk green un-ignored, every `walkreal_*_live`, the full
   suite total line.

## Live checks for the main agent

1. **The glass pass at the bottom terminal**: in PCSX2, stand Bond inside the bottom-terminal
   cabin and fire the AT-420 at a LIVE window — expected: the rocket flies THROUGH the glass
   (the window shatters from the mirrored contact) and continues out; it does NOT detonate on
   the pane plane. Address evidence: the flight's HITTEST+0x9a reads 0x20 (watch `obj+0xD4+0xEA`
   = `s4+0xEA` during a flight) and the contact walk passes the mat-0x10 node [M 0x1ea398:216-229].
2. **The AT-420 record word**: read `weapon_data[0x2c]+0x6C` live (0x2bf150 + 0x2c·0x10C) —
   expected 0x00012204 (bit 0x800 clear) [L the ACTION.ELF dump].
3. **The Hind's hp at 00 Agent** (carried over from the doors report): expected 598 = 2.0 × 299
   [M Copter_Create 0x17d340's DiffMod].

## Usage

not visible to me — no token, credit, cost or context-window figures are exposed anywhere in
this environment (no status bar, no session summary, no API dashboard access).
