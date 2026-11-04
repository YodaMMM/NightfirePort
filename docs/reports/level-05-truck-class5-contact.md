# level-05-truck-class5-contact — the 05 truck ride's own contact

Trigger-audit item #11 asked for the 05 truck ride's "class-5 contact" with the park plate
(ch 0x70). The decompile settles the chain differently, and the port's ride now matches it.

## What the decompile says

**The ride's plates are MASK-8 player triggers, not class-5.** The truck script 0x06000A22
creates them as type-0xEA/0xEB Entity triggers with params (out, 8, 0…): the park plate ch 0x70
(stream 6, entity 0x0200087F), the bed touch ch 0x71 (stream 7, 0x02000AC8, parented to the
truck) and the hand-off ch 0x5C (stream 8, 0x02000480) [D the decode,
docs/formats/nis-script.md §06000A22 + a raw placement param dump: #607-style params p0=out,
p1=8]. The wrapper mapping is disasm-verified: `Trigger_Touch` 0x1b2aa8 / `Trigger_TouchOnce`
0x1b2b88 write the type into level_tag slot 0, move the ORIGINAL slot 0 (the Entity param[0] =
the out) into slot 1 and the ORIGINAL slot 1 (the Entity param[1] = 8) into slot 9, and
`Trigger_Create` 0x1b1720:80-84 reads the mask from ext+0x36 = level_tag param 9, defaulting a
zero to 2. So the plates' effective mask = 8 (player), and `Trigger_Hit`'s mask-8 arm returns
exactly the class-3 contact bit [M 0x1b1fc8:56-58 `if (uVar1 == 8) return (uVar8 >> 3) & 1`].

**The truck's own contact credits nothing.** `Trigger_Update` passes the TRIGGER's own object to
`Trigger_Hit` [M Trigger_Calc 0x1b1900:55-57 → 0x1b1a78:71], whose contact walk classifies each
contact owner's class byte [M 0x1b1fc8:27-47: 3→8, 2→0x10, 5→2, 0x4a→0x10]. The truck is a
script player: `SP_CreateScriptPlayer` stamps obj+0xFF = 0x1b [M 0x1db9a8:71]. 0x1b is in no
classification arm, and the decompile-wide class census agrees — only `Bullet_init` 0x12c5a0:87
writes 0xff = 5 (Car_Create 0x1305b0 = 0x28, Sub_Create 0x1ad6c0 = 0x4a). So even where
`Collide_Update` 0x1ea398:160-215 mirrors the truck's contact onto the plate's obj+0xD0 list, it
lights nothing. The port's player-toucher-only model is faithful for the whole chain —
item #11's "class-5" reading is disproven for it (docs/atlas/trigger-port-check.md item 11 is
annotated). The port's mask&2 (class-5) arm stays modelled for the case the glass pass built it
for: `BulletContact` (bullets, whose obj+0xff = 5 [M Bullet_init 0x12c5a0:87]).

**The standing case is settled by the decompile — the plate fires standing.** The plate fires
from Bond's OWN class-3 contact: the collide-pass HITTEST segment [M `Player_Collision`
0x194df0's default arm — A = pos+0.275, B = pos+0.55−cc, r 0.55, mode 0x800] refined by
`Intersect_CylGeom` 0x109da0 (the clamp counts the segment-end region against the triangle,
docs/systems/player.md) and mirrored into the plate's list [M 0x1ea398:160-215]. Standing on the
bed (feet 37.28 [D block 0x2E of 0x020003be], pos−floor 1.086 [L the trace's standing offset]),
B = pos−0.4847 sits 0.163 above the plate at 37.7418 — well inside r 0.55, in EITHER stance (the
crouched arm's B = pos−0.1 sits 0.045 above it). No live read is needed to settle the question;
the numbers above are [M]/[D]/[L].

## The fix

`drone.rs`'s standing toucher arm: the bottom end is now the game's own segment B
(`Player_Collision`'s stamped default arm — `player.body`'s B), giving the toucher the game's
deep reach (its lowest point = B−r = pos−cc). The calibrated box top is kept (pos+1.26 — the two
top-side fires, the ch-0x5C hand-off plate and 07000011's #235, come through the port's PARSED
glist extents which ride above the game's actual reach — the documented ref-gate/extent
divergence). The deep bottom is gated to the MOVING CARRIER: `Sim.riding` is set from
`ground_motion`'s Some [M `Player_FeetOnPoint` → `Control_InheritVelocity` 0x133e88's ride;
world.rs answers Some only when the first surface under Bond belongs to a moving object] and
`drone::Player.riding` carries it to the toucher (a new field — every literal updated).

Why gated: the same deep reach on the FLOOR fires the port's MISPARSED low beams. Measured on
this branch: 0a's #614 parsed band (y 0.25–0.53) sits ~0.9 UNDER the west corridor's floor
(≈1.2) — the faithful deep reach fired ch 0x22 through the floor on the walked ring
(NF_CHWATCH: the 0x22 stamp at frame 4199, in `triggers::calc`), while the game's own jump
clears the line [L 2026-11-01: 0.9–1.3 takeoff clears, a walk fires]. The parse divergence is
the same lever that keeps the box top; fixing the glist ref-gate/extent extents is the next
step that would let the deep reach go unconditional (the toucher comment carries the plan).

## The pins (all real input)

- `fix_truck_live::truck_ride_parks_bond_at_the_game_height_and_ends_the_level` — the crouched
  ride vs the trace (unchanged, green).
- `fix_truck_live::standing_ride_still_fires_the_parking_plate` — FLIPPED to the decompile-backed
  law: the standing ride fires ch 0x70 and the chain ends the level into 07000006; pins the
  segment-reach arithmetic (B−plate = 0.163 < 0.55).
- `mayhew_handover_live::the_standing_ride_still_parks_and_ends_the_level` — NEW, the real-input
  standing ride: the walk from the cold start to the gatehouse edge, the hand-over window, then
  NO crouch input ever — ch 0x71 parents Bond, the standing ride carries him west, ch 0x70
  fires, the level exits into 07000006. A rider who stays standing is NOT stuck.
- `mayhew_handover_live::the_walk_to_the_gatehouse_fires_the_handover_window_and_rides_out` —
  the crouched real-input ride (unchanged, green).

## Verification

Per binary (release, --test-threads=1, NF_ISO on the slim ISO):
- fix_truck_live 2/0; mayhew_handover_live 3/0; walkreal_07000005_live 14/0;
  gate_guard_truck_live 2/0; interactables_07000005_live 6/0.
- walkreal_0700000a_live 12/0/3 (the three ignores are master's own: the low-beam port-gap
  note, the corner-crossing mechanism, the west-hall glide height) — the deep reach's floor
  regression was caught and gated here (see above).
- nf-game lib 657/0/5; nf-formats 92/0; nf-audio 23/0.
- npc_doors_live 1/0/1; fix_guard_misc_live 5/0.
- Every walkreal whose level can put Bond on a MOVING carrier (the gate's active set) or carry
  the plate chains: walkreal_07000005_live 14/0, _0700000a_live 12/0/3, _07000008_live 6/0/1
  (the gondola ride), _07000002_live 3/0/1 (the lifts), _0700000b_live 6/0, _0700000c_live 7/0,
  _0700000c_barrelrow_live 1/0, _0700000d_live 3/0, _07000001_live 6/0, _07000003_live 8/0,
  _07000004_live 6/0/1, _07000006_live 10/0, _07000007_live 6/0/1, _07000009_live 8/0,
  _07000011_live 3/0/1, _07000012_live 3/0, _07000013_live 3/0, _07000014_live 9/0/1 — 0 failed.
- The full suite did not fit: the sandbox's disk filled twice during the run (a duplicate ISO
  copy and the target tree; the second fill killed the cargo process mid-build, background jobs
  are reaped between tool calls) and the 2-hour window closed before a clean rerun. The
  remaining walkreal binaries not listed above (_0700000c_eastroute, _07000011_route, _15, _16,
  _1b, _41, _4a) exercise levels with no moving carrier under the walk — the toucher's standing
  arm is byte-identical to master there (the deep reach rides only on `riding`) — plus the
  ~30 nf-game test binaries whose only diff is the inert `riding: false` literal.

## What's left

- The glist ref-gate/extent divergence (the parsed trigger cels' extents riding off the game's
  actual placement — 0a's #614 under the floor, 07000011's #235 over pos, the 0c ch-27 bay
  volume) is the documented next lever; with it parsed right, the standing toucher's deep reach
  can go unconditional and the box top retires.
- The general deep reach's floor-plate band (0.6–cc below the reference — e.g. 07000002's bunker
  plate 0x80) stays unfired until that parse fix; unchanged from master on the floor.

## Live checks for the main agent

None required by this task's verdict — the decompile + disc data settled the standing case (the
plate is mask 8 [D], the segment reach is [M]). Optional confirmation, if a ride window is ever
played standing in PCSX2: watch the ch-0x70 write (`switch_channels_time` at 0x2a3e2c+0x70·4
style, or gdbwatch on `switch_channels[0x70]` = 0x2a3e34's byte) during a standing ride —
expect the stamp on the park tick, matching the port's pin.

## Usage

`not visible to me` — this sandbox exposes no token, credit or context-window figures for the
session (no such counters were shown in any tool output or environment surface during the run).
