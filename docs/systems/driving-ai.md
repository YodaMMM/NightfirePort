# The driving missions' AI cars — road network, path following, speed, spawn

Status: **§12 (2026-11-21) adds the road network logic: curves, the chase/follow choosers, the lights and the yield, traffic spawn and respawn; §12.5 the nearest-road join, the chasers' lane pick and the car box.** **Reviewed against the decompile 2026-11-20 — §9 is the per-function table and
supersedes §3.1, §5 and §8 wherever they differ** (first port 2026-10-15). The port lives in
supersedes §3.1, §5 and §8 wherever they differ; §10 (2026-11-21) adds the hostile AI and the
AI vehicle lifecycle and supersedes §3.3 and §4** (first port 2026-10-15). The port lives in
supersedes §3.1, §5 and §8 wherever they differ; §10 (2026-11-21) adds the hostile AI and the
AI vehicle lifecycle and supersedes §3.3 and §4; §11 (2026-11-21) adds the per-pattern steering**
(first port 2026-10-15). The port lives in
`crates/nf-game/src/driving/` (`road.rs` the RNgp/Wmap-side data model + `ai.rs` the AI car step) with the
RNgp container decode on top of `nf_formats::crp`. Evidence tags: [M] code (function + address,
DRIVING.ELF — EE address = file offset + 0x107080), [D] disc bytes, [L] EE snapshot, [S] suggested.
Everything below was read out of `work/decomp/DRIVING.ELF/`; nothing from memory. The
container/carrier facts (CARP records, RNgp as the road-network article) come from
docs/formats/crp.md.

## 1. The road network: the `RNgp` article and its five arrays [M]

The track `.crp`'s `RNgp` article ("<<RoadNetwork>>" per the namer FUN_002dc1b0 0x2dc1b0,
docs/formats/crp.md §3) is read by **`FUN_00220c28` 0x220c28** (1,744 B, "Init RoadNetwork", called
lazily through FUN_00220c18/DAT_0033a5f8 — also in the mission-setup list as "Init AI Controllers"
FUN_0013e688's neighbour FUN_00220c28, docs/atlas/driving-elf.md §1.4):

1. `FUN_002d51f0(track->0x14 /* the crp */, 0x524e6770 /* "RNgp" */)` locates the article [M 0x220c28:71].
2. Inside the RNgp window it reads **`RNhd`** (header; payload = five u16 counts) then the indexed
   short-tag records **`rn  `** (nodes), **`rs  `** (segments), **`ri  `**, **`rr  `** (routes),
   **`rj  `** [M 0x220c28:75-201]. The counts map 1:1 to the tags in the order rn/rs/ri/rr/rj
   ([M 0x220c28:82-86] reads them in that order).
3. Each array is stored as a malloc'd table of **pointers** at DAT_0033a5fc (rn), DAT_0033a600 (rs),
   DAT_0033a604 (ri), DAT_0033a608 (rj), DAT_0033a60c (rr) [M 0x220c28:91-96]. Every consumer indexes
   them as `*(int*)(idx*4 + DAT_0033a5xx)` — e.g. FUN_002247b0 0x2247b0:125,173, FUN_0012e808
   0x12e808:165 — so the port keeps `Vec<*const Rec>`-shaped `Vec<&Rec>` tables.

### 1.1 `rn  ` — road node [M]

| offset | type | meaning | proof |
|---|---|---|---|
| +0x00..+0x0c | 3× f32 | position x,y,z | FUN_00225c48 0x225c48:179-186 reads the triple; FUN_00126c98 0x126c98:178 copies 12 B |
| +0x0e | u8 | link count (outgoing segment count) | FUN_002247b0 0x2247b0:99,139-150 (`(char)(uVar4+0xe)`, loop bound) |
| +0x10 | u16[] | the linked segment indices | FUN_002270e8 0x2270e8:12-22; FUN_002247b0 0x2247b0:153,173 (`*(ushort*)(iVar16 + iVar7*2 + 0x10)`) |

### 1.2 `rs  ` — road segment (the AI spline piece) [M]

A segment joins node A to node B; a car drives it A→B or B→A. Measured fields:

| offset | type | meaning | proof |
|---|---|---|---|
| +0x0c | u16 | node A index | FUN_002215f8 0x2215f8:7-8 |
| +0x0e | u16 | node B index | same |
| +0x00 | 3× f32 | the Bezier control point beside node A (flags&0x4000): FUN_00222d40 passes the record itself as P1 of FUN_001a8380 (§12) | [M 0x222d40, 0x226538] |
| +0x10 | 3× f32 | the Bezier control point beside node B: P2 (§12; the old "P2 = rs+0x10, quadratic" reading is superseded) | [M 0x222d40:54, 0x226538] |
| +0x1c / +0x1e | s16 | the junction (`ri`) at node A / node B, −1 none | [M 0x223a30, 0x1377c8; D] |
| +0x2c | f32 | the reference chord the lane control points scale against (= \|A−B\| on 33 of Paris's 105 curves, longer on the 0x4641 junction connectors) | [M 0x226538; D] |
| +0x3c | s16 | route (rr) index | FUN_00126c98 0x126c98:137; FUN_0013d3f0 0x13d3f0:49 |
| +0x3e | s16 | the segment's own id | compared as an id in FUN_002247b0 0x2247b0:105, FUN_002270e8 0x2270e8:16 |
| +0x40..+0x4c | 3× f32 | unit direction A→B | FUN_002216b0 0x2216b0:8-10 |
| +0x4c | f32 | **speed limit** — the AI's per-segment target speed | FUN_0012e808 0x12e808:165 (`*(float *)(*(int *)(nav->0x3e * 4 + DAT_0033a600) + 0x4c)`) |
| +0x50 | u16 | flags | 0x1 blocked-variant [M 0x2270e8:17 `(f^1)&1`]; 0x2 [M 0x1377c8:117]; 0x8 type-restricted [M 0x225490:129]; 0x10/0x800 road class → lane table [M 0x221e98]; 0x100 skip + trigger id at rs+0x59 [M 0x225490:127, 0x126c98:148]; 0x200/0x400 turn restrictions [M 0x2247b0:115-118]; 0x4000 curved [M 0x222d40/0x222468]; 0x8000 skip-type5 [M 0x225490:140] |
| +0x52 | i8 | lanes, negative side | FUN_00221f20 0x221f20:28-29 |
| +0x53 | i8 | lanes, positive side | same |
| +0x59 | u8 | trigger id (with flags&0x100) | 0x126c98:148 |
| +0x5a | i8 | car-group id (spawn filter) | FUN_00225490 0x225490:133 |
| +0x60 | u64 | runtime visit stamp (nearest-query dedup) | 0x225490:121-126 |

### 1.3 `rr  ` — route [M]

`+0x2e`/`+0x2f` u8 occupancy counters, incremented/decremented as cars enter/leave
([M 0x220c28:201-202 init, FUN_002213d8 0x2213d8:25-26, 0x126c98:134/142]). The traffic spawner weighs
routes by occupancy (FUN_0013d3f0 0x13d3f0). Rest of the record: unproven.

### 1.4 `ri  ` junctions, `rj  ` [M + D]

`ri` (48 B on every track [D]) is the junction record: +0x00 3× f32 position (the light check
measures from it, FUN_0013efc8), +0x0c u16 own index, +0x0e u8 neighbour count with s16
neighbour junctions at +0x10 (at most 4 on the disc), +0x18 u8 road count with s16 road pieces at
+0x1a (FUN_00223a30, FUN_001377c8) [M; D every listed road points back through rs+0x1c/+0x1e].
`rj` has no reader in the decompile; the port keeps its raw payloads. §12 has the users.

## 2. How a car follows the road [M]

The car's path state lives at car+0x1d0 (first arg of FUN_00225c48/002257c0/002247b0):

| offset | type | meaning |
|---|---|---|
| +0x00 | u32 | kind: 1/2/4 — 4 = traffic [M 0x225c48] |
| +0x04 / +0x20 | 3× f32 | previous / current position |
| +0x30 | 3× f32 | direction |
| +0x3c | u8 | end: 0 = driving toward A, 1 = toward B [M 0x2257c0 join: `dot(dir, rs->0x40)` sign picks the end] |
| +0x3e | s16 | current segment index |
| +0x50 / +0x60 | 3× f32 | node A / node B position cache |
| +0x70 / +0x80 | 3× f32 | curve control (from rs+0x10 when flags&0x4000) [M 0x2257c0:174] |
| +0x90 | ptr | curve object (0x4191e0 template) |
| +0x95 | i8 | lane |
| +0x98/+0x9c/+0xa0 | f32 | progress along the segment |

The per-frame position: straight segments evaluate `nodeA + (nodeB − nodeA)·t`
(**FUN_00222468 0x222468:12-20**, VU `_vsub/_vmulbc/_vadd`), curved segments go through
**FUN_00222d40 0x222d40** → the 4-point path **FUN_001a8380 0x1a8380** + **FUN_001a85d0 0x1a85d0**
into the curve object at 0x4191e0 (lazy-init once, DAT_0033a958 [M 0x222d40:49-53]).
Joining a segment: **FUN_00225c48 0x225c48** (2,288 B) — `dot(car dir, rs->0x40 dir)` picks the
direction (end A/B), the lane offset comes from **FUN_00221f20 0x221f20** (lane-offset table at
DAT_00427000, 6 entries built by FUN_00220c28:24-61, selected from rs flags by FUN_00221e98,
clamped by rs+0x52/+0x53) [M].
At the segment end: **FUN_002247b0 0x2247b0** (1,616 B) picks the next segment — walks the exit
node's u16 list (rn+0x10, count rn+0x0e), filters rs flags 0x2/0x80(trigger)/0x200/0x400, XORs the
candidate's node indices against the entry node to detect direction reversal, and returns the next
segment + lane [M 0x2247b0:99-255].
The nearest-segment query (used to find the segment under a world position) is
**FUN_00225490 0x225490** (816 B): world→cell through the CDat `CGrd` grid (DAT_0033a5c8; cell
payload `cn  ` +0x7 count, +0xe u16 offset into a s16[] of segment indices — the road net reuses the
collision grid [M 0x225490:104-120, FUN_0021c1f8 0x21c1f8:48-50]), dedup via rs+0x60 stamp vs
DAT_0033a610, closest point via FUN_00222968→FUN_00222a30 (returns t and position), curve distance
via FUN_002223b0 when flags&0x4000 [M].

## 3. The AI car object and its per-frame step [M]

`AIVehicle` (RTTI name [D]; 0x2a0 B, ctor FUN_0012dd80 → FUN_0013de68 + FUN_0012ded8):

| offset | type | meaning | proof |
|---|---|---|---|
| +0x04 | ptr | the actor (vtbl at actor+0x68: +0xd4 SetSteer f32, +0xf4 SetThrottle f32, +0xfc SetBrake f32, +0x104 SetHandbrake i32, +0x9c mark-updated) | FUN_00133ab0 0x133ab0:136-143 call sites |
| +0x54 | 0xc0 B | WRoadNav "current" (road-node/segment id at nav+0x3e; nav+0x90 holds a 0x70 spline) | 0x133ab0; nav+0x3e read at 0x12e808:165 |
| +0x58/+0x5c | u32 | enable flags (Update guard) | FUN_0012e350 0x12e350 |
| +0x60 | u32 | slot id (pool slot from FUN_0013e688) | 0x13e688 |
| +0x70 | u32 | state 1..5 (1 standby, 2 ?, 3 teleport-to-marker, 4 traffic, 5 chase) | FUN_0012e350 switch; FUN_00131620 = the state-3 teleport |
| +0x78 | u32 | drive pattern 1..5 | 0x12e808 switch on +0x84's sibling |
| +0x84 | u32 | profile (1, 0xb..0xf; 9/10 = fire modes) | 0x12e808:56 (`iVar2 = *(int *)(param_1 + 0x84)`) |
| +0x8c | f32 | **target speed** (written by FUN_0012e808, read by FUN_00130508→FUN_00133ab0) | 0x12e808 stores; 0x133ab0:100 |
| +0x134 / +0x138 | f32 | speed multiplier (init 1.0) / max-speed cap | 0x12ded8 init |
| +0x164 | s16 | spawn-delay countdown | 0x131018 |
| +0x19c | u32 | forced-stop flag (≠0 → throttle 0, brake 0, handbrake 1) | 0x133ab0:144-179 |
| +0x1a4 | ptr | chase target (an AIEl element) | 0x131770 |
| +0x226/+0x22c | s16 | gun cooldowns; +0x228 f32 fire rate | 0x12ded8/0x130050 |

Per-frame: the world scheduler (FUN_00168280 0x168280, docs/atlas/driving-elf.md §1.2) calls the
vtable+0x1c Update = **FUN_0012e350 0x12e350** (280 B): guards +0x58/+0x5c, honors the +0x1a0 reset,
then switches on state +0x70:

- **1 standby**: FUN_00131018 (activate the sim object when the spawn delay +0x164 counts out).
- **2**: FUN_00133850 + FUN_00131080 (1,160 B; re-targets on a 4 Hz slice, +0x174).
- **3**: FUN_00131620 (336 B) — teleport: FUN_001f7880 sets the sim transform from the AIEl marker.
- **4 traffic**: FUN_00132d80 (508 B) node-occupancy bookkeeping (init FUN_00132808 924 B).
- **5 chase**: FUN_00133850 + FUN_00131770 (4,240 B target management) + FUN_0012e808 (target speed)
  + FUN_00130050 (236 B weapons) + FUN_00130508 (2,432 B lead-aim FUN_00132f80, then the drive arms
  FUN_00133f60/FUN_00135f70/FUN_00137310/FUN_001377c8).
- States 2/4/5 share **FUN_0012e808 0x12e808** (2,408 B, target speed → +0x8c) and
  **FUN_00130508 0x130508** → **FUN_00133ab0 0x133ab0** (1,196 B, the drive-control applier).
- Tail: actor vtbl+0x9c(1) (mark-updated) [M].

### 3.1 Target speed (FUN_0012e808 0x12e808) [M]

- State 5 requires FUN_00195a30(actor) > 0 else target = 0 (dead/despawned → stop);
  state 4 requires ≥ 0.5 [M 0x12e808:9-29].
- vtbl+0x144 query ≠ 0 → 0 (some per-actor stop request); byte +0x170 ≠ 0 → 0 [M 0x12e808:41-52].
- Then per profile +0x84 / pattern: the **current segment's speed limit rs+0x4c** is the base
  [M 0x12e808:165]; observed constants in the selector: 10.0 (a minimum), 25.0, 30.0 (chase case),
  7.5 (a follow distance: physics param +7.5, FUN_00132f80), multipliers 0.35/0.5/0.75/0.8/0.85/1.1/1.25
  (the 1.25× case scales a gap by `phys0x4c8` of self+target) [M — per-case mapping not fully
  disambiguated; the profile→case table is [S]].

### 3.2 Drive control (FUN_00133ab0 0x133ab0) [M]

Inputs: target speed (param_1, after the caller multiplies by the +0x134 profile factor), the AI
object, the aim/heading vector (*param_3). Steps, in order:

1. `dir = normalize(aim − carPos)` (sim obj +0x10/0x20 positions; `_vsub/_vmul/_vrsqrt`) and three
   dot products against the car's facing vectors (sim +0x5c rows) → alignment factors [M 0x133ab0:40-96].
2. `steer = clamp(alignment, ±1)` (FUN_00210510) → **actor vtbl+0xd4 SetSteer** [M 0x133ab0:135-137].
3. **SetThrottle(0), SetBrake(0), SetHandbrake(0)** reset [M 0x133ab0:138-143].
4. `speed = |car velocity|` (sim +0x20, `_vsqrt`) [M 0x133ab0:103-113].
5. `target' = max(target · |alignment|, 0.5)` [M 0x133ab0:100-102].
6. Arrival: `radius = max(speed · 0.5, 5.0)`; if `dist(aim, carPos) ≤ radius` → `target' = 0`
   [M 0x133ab0:114-130].
7. If +0x19c (forced stop): throttle 0, brake 0, handbrake 1 [M 0x133ab0:173-180].
8. Else `ramp = min(|target' − speed|, 7.5) / 7.5`, floored at 0.05; `target' > speed` →
   **SetThrottle(ramp)**; otherwise brake-side early-outs (speed ≤ 0 → return; alignment ≥ threshold
   and target' ≥ 2.5 → return) then **SetBrake(ramp)** [M 0x133ab0:144-172].

### 3.3 Weapons (the chase states' fire) [M]

Profile +0x84 == 9 → FUN_0012f730, == 10 → FUN_0012fa28, else FUN_0012fd58 (dispatched through
FUN_00130050, gated by FUN_00195a30(actor) > 0): line-of-sight FUN_0012f530, aim-dot thresholds
0.9/0.99, then FUN_001a27a0(animObj, channel {9 | 5/6}, frames 8→10→0xb, …) — animation-channel fire
commands on the actor's +0x48 object; cooldowns +0x226/+0x22c tick down in frames × rate +0x228
[M 0x130050/0x12f730/0x12fa28/0x12fd58 call shapes].

## 4. Spawn/despawn per mission [M]

- **The AIEl database**: the `Map `→`AIEl` chunk (mission setup FUN_00176658's "Init AI Elements"
  calls FUN_001c4840 then FUN_0012be48) is parsed into the 0x58-byte database at DAT_00327c30 by
  **FUN_0012b8b0 0x12b8b0** (1,216 B): slot availability u32s at +0x00/+0x04/+0x08 (henchman/vehicle/
  heli; defaults 32/6/16 from attributes 0x36b120/0x36b138/0x36b150), the 0x100-byte record array at
  +0x0c, per-slot runtime records (0x50 B, init FUN_0012af48: activation distances 200.0/220.0).
- **AIEl record** (0x100 B): +0x44 s32 type (0 = henchman, 1 = vehicle); +0x48 s32 must equal 1 to
  activate; +0x50 ptr to the linked target element (the chase target); +0x4c f32 scale (× the global
  FUN_002084a8); +0x54 char[11] model name ("rendername" attr → `data\car\model\`); +0x7c sub-count,
  subs 0x20 B at +0x80 [M FUN_0012b8b0 + FUN_0012cb08 attr loader].
- **The pool**: "Init AI Controllers" **FUN_0013e688 0x13e688** (436 B) preallocates the AIVehicle
  objects: VEHICLE_SLOTS_AVAIL (6) heap + **22 static** (2 at 0x327d90 + 20 at 0x3282d0, reset by
  FUN_0013ff98) + 16 AIHelicopter (0x130 B) [M].
- **Activate**: a trigger/script (FUN_0014ae60 through the manager at *DAT_00339f5c, vtbl+0x5c
  GetFree) → **vtbl+0x24 FUN_00131508** (sets the +0x58/+0x5c enables and state 3 = teleport to the
  AIEl marker; config from record+0x50's linked target) [M].
- **Deactivate**: FUN_0014c110 → vtbl+0x2c **FUN_00131590** → state 1 standby [M].
- **FUN_00126c98 0x126c98** (2,136 B) — earlier read as the traffic director; it serves a
  different class (path state at +0x1d0, vtable at +0x140, animation calls FUN_0012aa20 /
  FUN_0012a8f0), not the AIVehicle (§9). It picks a
  spawn segment by route occupancy (FUN_0013d3f0), updates rr+0x2e/0x2f, reads rs+0x59's trigger, and
  places its object at `nodeB + rs dir·0.75` [M 0x126c98:134-208].

## 5. The port

`crates/nf-game/src/driving/road.rs`: `RoadNet::parse_rngp(&[u8])` — the RNhd counts + the rn/rs/ri/rr/rj
record tables exactly as FUN_00220c28 builds them (records kept as byte slices with typed accessors,
every offset from §1); `RoadNode`/`RoadSeg` accessors (`node_pos`, `seg_a`, `seg_b`, `seg_dir`,
`seg_speed_limit`, `seg_flags`, `seg_lanes`, `links`), the straight-segment position evaluation
(FUN_00222468), the join direction rule (FUN_00225c48's dot test), and the next-segment choice
(FUN_002247b0's filter chain: flags 0x2/0x200/0x400, self-id skip, direction reversal by node XOR).
`crates/nf-game/src/driving/ai.rs`: `AiCar` — the §3 subset a headless sim needs: state, target speed
(FUN_0012e808's traffic path: the segment limit rs+0x4c, the alive gates), and the FUN_00133ab0 drive
control (steer clamp, throttle ramp min(|Δ|,7.5)/7.5 floor 0.05, arrival radius max(speed·0.5, 5.0)),
stepping position along the segment at the game's 60 Hz. No drawing, no RNG draws.

Tests (`crates/nf-game/tests/driving_ai_follow.rs`): a synthetic but format-exact RNgp fixture
(straight segments + a curved one), an AI car entering at a node, stepping at 1/60 s:
- it converges onto the segment and reaches the segment's speed limit (rs+0x4c) via the game's ramp;
- it crosses to the next segment at the shared node and keeps the direction rule (A→B vs B→A);
- the arrival rule stops it inside `max(speed·0.5, 5.0)` of the aim;
- the 0x200/0x400 flag filters and the car-group check exclude segments exactly like FUN_002247b0;
- a sister assertion per test (a wrong-format fixture must fail).
ISO/DRIVING-gated (`NF_DRIVING` set, like driving_golden.rs): parse the real `MIS01.VIV` track RNgp,
walk an AI car along a real segment chain, pin the real record counts (numbers only).

## 6. Live checks (open)

- **L-1 — rs+0x4c is the speed limit**: breakpoint FUN_0012e808 0x12e808 during a traffic mission;
  confirm the float read at `*(rs)+0x4c` lands in +0x8c and matches the mission's traffic speed
  (paris_mis01, state-4 cars) [M expectation].
- **L-2 — lane offsets**: dump the 6-entry lane-offset table at DAT_00427000 after "Init AI
  Controllers"; expect lane-center offsets in game units [M expectation; the port uses the table's
  existence, not its values].
- **L-3 — curve eval order**: breakpoint FUN_001a8380 0x1a8380 on a curved segment (flags&0x4000);
  record the four control points passed (param_2/3/5/4) to confirm P0/P1/P2/P3 ordering vs rs+0x10
  [S expectation].
- **L-4 — real RNgp golden**: run `NF_DRIVING=<extracted DRIVING folder> cargo test -p nf-game
  --release driving_ai` — the gated tests parse the real paris_mis01 RNgp; confirm the counts against
  a PCSX2 dump of DAT_0033a5fc..'s array lengths after "Init RoadNetwork" [M expectation].

## 7. What's left / not done

- The per-case mapping of profile +0x84 → FUN_0012e808's constants (10.0/25.0/30.0 and the 0.35..1.25
  multipliers) — the selector is 2,408 B with a nested pattern switch; only the traffic base rule
  (rs+0x4c) is pinned [M]; the rest is [S].
- `ri`/`rj`/`rr` non-occupancy fields (§1.4).
- The curve interpolation of FUN_001a85d0 0x1a85d0 (the port carries straight segments + the control
  data; exact curve eval is a live check, L-3).
- Weapons fire (§3.3) is documented, not ported (needs the actor/animation layer).
- The AIEl chunk binary layout inside the `.crp` (record +0x44.. is proven from the parsed runtime
  records; the on-disc AIEl chunk record layout itself is [S] — the port reads AIEl only through the
  documented slots, real-data AIEl parsing is a live check).
- The driving physics landed beside this: the seam is wired (2026-10-15) — see §8 (`SimCar::of_car` + `AiDriver`, no trait needed).

## 8. The physics wiring (`drive-ai-traffic2`, 2026-10-15)

The AI landed beside the physics with `SimCar` as the unwired seam; the physics then moved from the
single file `driving.rs` to `driving/physics.rs` (the `driving/` directory had made the two layouts collide,
rustc E0761) with a top-level re-export
keeping every public path (`nf_game::driving::{Car, CarInput, CarParams, DT, FlatWorld, Rigid,
Tuning, Friction, SURFACES}` — the module test pins them). The wiring:

- `ai::SimCar::of_car(&Car)` [M the field sources]: pos = instance+0x10, vel = instance+0x20,
  forward = the yaw's `[sin, 0, cos]` row (the row `Car::step` drives along [M physics `step`'s
  `fwd`]); `alive` = 1.0 — FUN_00195a30's health gate is the actor layer, out of headless scope
  [S].
- `ai::AiDriver` { `brain: AiCar`, `car: Car`, `world: FlatWorld`, `rigid: Rigid` }: one 60 Hz
  step = the brain's FUN_0012e808 → FUN_00133ab0 command mapped 1:1 onto 506's `CarInput`
  (SetSteer/SetThrottle/SetBrake/SetHandbrake [M 0x133ab0:135-180]), `Car::step` integrates it
  [M 0x1ff3d8's order], the **pre-integration** sim speed drives the path progress (the AI reads
  |vel| before the vehicle sim integrates [M 0x133ab0:103-113]; `AiCar::advance_path` is the
  progress walk moved verbatim out of `AiCar::step`, which keeps its exact rail behavior), then
  the car lands back on the spline [M 0x222468] facing the travel direction.
- The brake rest clamp [S]: 506's brake integrates through zero into a negative vel (reverse is
  the R-gear throttle path 506's gearbox work owns), which would back the rail car up; the driver
  zeroes a brake-induced backwards velocity, and at rest the AI's own speed ≤ 0 early-out
  [M 0x133ab0:153] keeps it parked.
- Known gap: `Car::step` does not read `handbrake` yet (506's report lists "handbrake in the
  step") — the forced-stop command is exact [M 0x133ab0:173-180] but holds speed by no force;
  the test `the_forced_stop_handbrakes_and_the_speed_holds_until_506s_handbrake_lands` fails the
  day 506's handbrake lands, on purpose.
- Tests: `crates/nf-game/tests/driving_ai_physics.rs` (8) — the layout paths, the sim view, the
  acceleration numbers with a bit-exact bare-`Car` replay (the driver adds no physics of its
  own), the steering-angle falloff, the arrival rest, the forced-stop record, the clean-link
  crossing at the physics speed, standby.

## 9. Function-by-function review (2026-11-20, branch `s/drive-ai-traffic`)

Every row was read in `work/decomp/DRIVING.ELF/` (and the disassembly where the decompile drops a
float argument). Verdicts: **correct** (the port matched), **wrong → fixed**, **added** (the
port had nothing; ported now), **reviewed** (read, not ported — stated so nothing is claimed).
Port = `crates/nf-game/src/driving/ai.rs` unless noted. Tests =
`crates/nf-game/tests/driving_ai_traffic.rs`.

### 9.1 Which car is "traffic"

A live civilian car is an AIVehicle in **state 4, profile 0xc, pattern 3, navigator kind 1**:
`FUN_0013f868` 0x13f868 creates it (state 4, +0x58/+0x5c enables); `FUN_00132ba8` 0x132ba8 parks
it (profile 1, pattern 1, teleported far up, sim +0x6e = 3); each state-4 frame `FUN_00132d80`
0x132d80 calls the respawn `FUN_00132808` 0x132808, which re-places a parked/out-of-range car
and sets profile 0xc (`FUN_0012e6f8(ai, 0xc)` :130) and pattern 3 (`FUN_0012e468(ai, 3, 0)`
:131); `FUN_0012e468` gives a state-4 car with pattern ≠ 1 navigator kind 1 [M].

### 9.2 The table

| function | address | verdict | fix / note |
|---|---|---|---|
| Update | 0x12e350 | correct | state 1 → spawn-delay (nothing driven), 4 → 0x132d80, 0x12e808, 0x130508; 2/5 add retarget/chase calls. |
| target speed | 0x12e808 | **wrong → fixed** | The port returned the segment's rs+0x4c. Profile 0xc stores the constant **7.5** (0x40f00000); rs+0x4c is read only by profile 0xf (× AI +0x134, when +0x158 is clear). Paris has rs+0x4c = 0 on 451 of 811 pieces [D], so the old cars stood still on most roads. Gates (alive, stop query, +0x170) were right. Profiles 1, 0xd, 0xe and 0xf-with-target (the chase arms) return `None`: not ported. |
| pattern + nav kind | 0x12e468 | added | constants only (`PATTERN_ROAD`, `NAV_KIND_TRAFFIC`). |
| profile setter | 0x12e6f8 | reviewed | sets +0x84; profile 0xd also flags the actor (+0x244) for one model name. |
| pattern dispatch | 0x130508 | **added** (pattern 3, state 4) | lookahead → carrot → drive control; the gated stop (all pedals 0, then `min(|v|, 0.25)` on the brake, or the throttle when rolling backwards, once `|v| ≥ 0.1`); a navigator that repeated its piece (+0x94) aims at the car itself. Not ported: patterns 1/2/4/5, the +0x198, +0xe4 (stop-and-wait) and +0x154 (avoidance 0x1377c8) arms — taken as clear [S]. |
| lookahead | 0x132f80 | **added** | `max(|v|·0.35, box_z + 7.5)`, box_z = sim+0x5c→+0x4c8, the z extent `FUN_001f6e98` 0x1f6e98 stores (floored 0.001). The extent's source is not traced; the port defaults to the floor. |
| carrot | 0x130140 | **added** (plain path) | The port aimed at the segment's far node. The game keeps a navigator point on the road: if `|car − carrot| < lookahead`, the carrot advances by the difference (one call). The +0x178 scripted path and the chase-target aim are not ported. |
| drive control | 0x133ab0 | **wrong → fixed** | (1) steer was `dot(dir, forward)` — 1.0 with the aim on the nose, a full lock on a straight road; the game steers by `dot(dir, row0)`, the side row. (2) Missing: aim behind (`dot(forward, dir) ≤ −0.2`) → full lock to the side's sign. (3) The brake side: the game's first test is `dot(normalize(v), forward) ≤ 0` → **throttle(ramp)** (disassembly 0x133ea8 `c.olt.s 0, va` / `bc1fl` → 0x133f00); the port issued nothing at speed ≤ 0. (4) Missing the +0x178 guard (no command). Ramp, floors, arrival radius, brake hold, forced stop were right. |
| clamp | 0x210510 | correct | clamp(x, ±1). |
| road advance | 0x224e00 | **wrong → fixed** | The port moved t by the car's speed over the node-to-node length, dropped the leftover past a node (`t −= 1` in the old piece's scale), stopped after one crossing and snapped the car onto the line. The game moves the carrot (not the car) by a distance over `|P0 − P1|` (lane-offset points), carries `(dt − (1 − t))·len` into the next piece and recurses; within a piece t clamps to [0, 1]. The car is a free physics body. |
| join | 0x2257c0 | **added** | kind 4 or `forward` → toward B (end 1, dir = rs+0x40, t); else toward A (end 0, −dir, `|1 − t|`); lane offset `±table[class][|lane|]`. The old port's comments had end 0 = A→B; the game's end is the exit node's slot (1 = toward B). |
| lane points | 0x2267d8 / 0x226a28 | **added** | toward B: P0 = A + side_a·off, P1 = B + side_b·off; toward A: P0 = B − side_b·off, P1 = A − side_a·off. rs+0x20 / +0x30 are the side rows: unit length on every disc track [D]; on the Paris pieces sampled they are +0x40 turned 90° in the ground plane, `(dir.z, 0, −dir.x)` [D]. |
| road class | 0x221e98 | added | flags 0x10 → 3, 0x820 → 2, 0x800 → 1, else 0 (`RoadSeg::road_class`). |
| lane tables | .data 0x33a618 / 0x33a678 | added | [D, read from the disc's DRIVING.ELF by the test] Snow2a set for `snow2a_mis4` / `snow2a_race` (0x220c28's strcmp), Default set otherwise: class 0 = 0, 2, 6, 10, 14, 18 (Default) / 0, 2.5, 7.5, 12.5, 17.5, 22.5 (Snow2a). |
| next piece, kind 1 | 0x223f98 | **wrong → fixed** | The port ran a reduced kind-2 walk (`RoadNet::next_segment`, modelled on 0x2247b0, which cars never run) with a car-group filter that is in neither function — the viewer passes groups 1..4, so its traffic cars stopped at the first node. Ported 0x223f98 whole: dead end → same piece, turn round; a plain link (flag bit 0 clear, `FUN_002270e8`) → take it, no draw; a junction → the slot rules, the lane-mask parity test on the road past each connector, the 0x1020/0x1000 lane rules, each eligible link taken on `rand/65536 < 0.3` (the last eligible one stands); the new lane offset raises the lane-change flag. The DAT_003c827c = 0 arms are dead (its only write sets 1, 0x220c28:63). |
| next piece, kind 4 | 0x2235c8 | reviewed | the A→B-only chooser for pattern-4 cars (0x130e88's road-limit followers); not ported. |
| next piece, kind 2 | 0x2247b0 | reviewed | not a car chooser; `RoadNet::next_segment` kept for its old test, unused by the AI. |
| plain link / other node | 0x2270e8 / 0x222300 | added | helpers. |
| lane blend | 0x227058 | reviewed | +0x98/+0x9c/+0xa0/+0xa4/+0xa8/+0xac blend of a lane change; idle outside a lane change (it returns 0 when +0x98 = +0xa0). Not ported. |
| spawn lane | 0x13c5e8 | **added** | `spawn_lane`: k = (route occupancy + spawn count) % lanes, slot k → lane, rs+0x58 bit k = drives A→B, else the lane is negated and the car drives B→A. The random start t is the caller's (`AiDriver::on_segment` uses 0). |
| spawn pick | 0x13ca60 | reviewed | picks from a 75..150 ring list (`FUN_0013d618(75, 150)`), route cap rr+0x2e < rr+0x2c, a 0.2-cell spacing check against live cars, a minimum player distance DAT_00327d68. Not ported. |
| respawn | 0x132808 | reviewed | out-of-range test `FUN_0013c860(box_z, …)`, the live cap DAT_0032b750, the 5·DAT_00339f4c delay, teleport to the carrot facing nav+0x30. Not ported (the placement it ends with is what `on_segment` does). |
| state-4 tick | 0x132d80 | reviewed | route occupancy rr+0x2e in/out, the +0xe4 stop flag (vtbl+0x17c, cleared at ≤ 0.2). Not ported. |
| 0x126c98 | 0x126c98 | doc fixed | not the AIVehicle traffic director (see §4). |
| generator | 0x17dc28 / 0x17db68 / 0x17dbb0 | added | `DriveRng`: `p = state·mult; state = p & 0xffff; out = (p >> 8) & 0xffff`; mult 0x075bcd15 (checked in the disc ELF); reset state 0xf874af01 + 39 warm-up draws. Every driving system draws from it; the position of the stream at a traffic pick is unproven. It is not the on-foot stream. |
| retarget / chase | 0x131080, 0x133850, 0x131770, 0x133f60, 0x135f70, 0x137310, 0x1377c8 | reviewed, not ported | the hostile cars' pursuit, ramming and avoidance. The port had no code for them before this branch and has none now. |
| weapons | 0x130050 / 0x12f730 / 0x12fa28 / 0x12fd58 | reviewed (§3.3), not ported | needs the actor/animation layer. |
| retarget / chase | 0x131080, 0x133850, 0x131770, 0x133f60, 0x135f70, 0x137310, 0x1377c8 | see §10 | the ram roll and sight rule of 0x131770 are ported in §10 (`s/drive-ai-hostiles`); the steering arms are still not ported. |
| retarget / chase | 0x131080, 0x133850, 0x131770, 0x133f60, 0x135f70, 0x137310, 0x1377c8 | see §10 | the ram roll and sight rule of 0x131770 are ported in §10 (`s/drive-ai-hostiles`); the aim point, the pursue/road switch and the chase drive in §11. |
| weapons | 0x130050 / 0x12f730 / 0x12fa28 / 0x12fd58 | see §10 | ported as fire decisions in §10; the animation-channel side stays with the actor layer. |

### 9.3 Stand-ins in `AiDriver` [S]

`Car::step` (the port's physics) has no tyre lateral force and lets a brake push the car
backwards through zero. `AiDriver::step` therefore (a) zeroes a brake-made backwards velocity
and (b) turns the velocity onto the heading each tick. Both are port gaps, not game behaviour.
The side-row sign is paired with `Car::step`'s steer sign so that the game's `steer = dot(dir,
row0)` closes on the carrot; the game's own row-0 / SetSteer sign was not read.

### 9.4 Open

- The traffic car's physics numbers: civilian cars carry their own pvehicle `.atr`
  (`hatchback.atr`, `sedan2.atr`, `suv1.atr`: MAXSTEERING 0.2, MAXACC 13.29 [D MIS01.VIV]); the
  viewer builds AI cars from the player car's parameters (vanquish: MAXSTEERING 0.09, a ~21-unit
  turning circle), which overshoots city corners.
- The lane set needs the track name; `AiDriver::on_segment` uses `LaneSet::Default` (wrong only on
  `snow2a_mis4` / `snow2a_race`, 2.0 vs 2.5 for lane 1).
- Curved pieces (flags 0x4000 — 105 of Paris's 811 pieces; Paris also has 105 junction connectors, flag bit 0 [D counts]) are
  evaluated straight; the curve object FUN_001a8380 / FUN_001a85d0 is not ported.
- A live check worth doing: break on FUN_0012e808 in Paris and read +0x84 / +0x8c of a moving
  civilian car (expect 0xc / 7.5).

## 10. Hostile AI and the AI vehicle lifecycle (2026-11-21, branch `s/drive-ai-hostiles`)

Port: `crates/nf-formats/src/driveai.rs` (the `AIEl` / `AICo` decode) and
`crates/nf-game/src/driving/hostile.rs` (the lifecycle, the chase speeds, the ram/sight timers,
the weapons). Tests: `crates/nf-game/tests/driving_ai_hostiles.rs` (20; the last one reads the
disc). Every row below was read in `work/decomp/DRIVING.ELF/`, with the disassembly where the
decompile drops the argument of `FUN_0013dfc8` (which actor a speed is read from) or a
branch. Verdicts as §9; **added** = the port had no code for it before this branch.

### 10.1 The data: `AIEl` and `AICo` [M + D]

- `AIEl` (Map child, tag 0x4149456c): `count` elements × 0x100 B, copied verbatim into the
  database [M 0x12b8b0]. Fields: +0x20 facing row, +0x30 spawn position, +0x40 flags, +0x42
  difficulty filter, +0x44 type (0 henchman, 1 vehicle), +0x48 state, +0x4c health (the live
  actor's health points here), +0x54 the pvehicle attribute name, +0x58/+0x5c/+0x60 henchman
  class/model/weapon, +0x74 a ctor byte, +0x77 helicopter, +0x7c rider count, riders at +0x80
  (0x20 each: seat, model, weapon, …, +0x14 health, +0x1c skill), +0xf8 the spawn command
  reference. Flags: 0x1 respawn where it left, 0x2 held by script, 0x4 persistent, 0x8 chase
  brain (sim kind 2), 0x10 counted (runtime), 0x20 damaged (runtime), 0x40 not counted.
- Rider seats are attributes in the vehicle's own `.atr` (`AIC_EWIND_POS` window,
  `AIC_ESUNR_POS` sunroof, `AIC_ESNOW_POS`, `AIC_EVGUN_POS`, `AIC_EVDRI_POS`, `AIC_BOND_POS`);
  weapon index 1 = `ssmg`, 2 = `bazooka` [D the ELF's 0x327b90 table]. The disc test checks
  that every rider seat the missions use exists in that car's `.atr`.
- `AICo` (0xa0 B): what `FUN_00145ff8` copies into the AIVehicle when a script command runs:
  +0x4c follow distance (AI +0x220), +0x5c catch-up (+0x13c), +0x60 speed cap (+0x138), +0x64
  seconds between shots (+0x228), +0x68 ram chance (+0x14c), +0x6c lead (+0x150), +0x78 target
  mode, +0x79 pattern, +0x7a profile, +0x87/+0x90/+0x91/+0x92 weapon bytes (each honoured only
  when the car's `.atr` says `HAS_MACHINEGUNS`/`HAS_CANNONS`, `HAS_ROCKETS`,
  `HAS_MISSLES`/`HAS_TORPEDOS`).

Per mission [D, the ISO test]:

| archive | AIEl | AICo | henchmen | held | chase brain | vehicle set (count) | slots h/v/heli | health × by difficulty |
|---|---|---|---|---|---|---|---|---|
| MIS01 | 25 | 61 | 0 | 21 | 14 | paris_henchbigbox 15, paris_bombvan 5, paradis_car 3, courier_truck 1, road_block_truck 1 | 32/6/16 | 1.0/1.5/2.0 |
| MIS11 | 45 | 68 | 0 | 45 | 7 | alpha_sub 2, mini_sub 9, helimine 34 | 32/6/16 | 1.0/1.2/1.4 |
| MIS13A | 28 | 16 | 14 | 25 | 12 | jungle_hench 8, ultralight 1, jungleheli 3, staticultralight 2 | 32/15/16 | 1.0/1.5/2.0 |
| MIS13B | 37 | 24 | 11 | 9 | 19 | jungle_hench 11, jungle_bombvan 1, ultralight 7, jungletank 2, patrolboat 5 | 32/15/16 | 1.0/1.1/1.2 |
| MIS13C | 43 | 35 | 7 | 43 | 36 | ultralightbig 32, patrolboat 1, jungletank 3 | 32/15/16 | 1.0/1.0/1.0 |
| MIS3 | 63 | 61 | 38 | 58 | 18 | small_snowmobile 8, supersnow 8, openheli 3, junglesnow 2, paris_hench 3, paris_bombvan 1 | 32/6/16 | 1.0/1.1/1.2 |
| MIS4 | 33 | 37 | 5 | 33 | 19 | alpine_police 9, supersnow 9, subcopter 3, paris_hench 6, plane 1 | 32/6/16 | 1.0/1.2/1.4 |
| RACE | 3 | 7 | 0 | 2 | 3 | paris_hench 1, alpine_police 1, supersnow 1 | 32/6/16 | 1.0/1.2/1.4 |

The speed factors (MSet +0x2b8/+0x2bc/+0x2c0) are 1.0 on every track. The jungle world `.atr`
raises `VEHICLE_SLOTS_AVAIL` to 15. Each AI car is built from its own
`data\sim\attrib\pvehicle\<name>.atr` (`FUN_001862c0` → `FUN_001968a8`) [M]: the hatchback
steers 0.2 against the player's vanquish 0.09 [D]. Traffic cars come from the table at
0x36c070 (coupe, sedan2, hatchback, sedan, station_wagon, suv1, taxi) through `FUN_0013fae0`;
Paris sets `MAX_TRAFFIC=20`, `NUM_TRAFFIC_TYPES_PS2=3`, so three types are drawn at random from
the seven (MIS01.VIV ships only hatchback, sedan2 and suv1 of them [D]; what the engine does
with a drawn type that has no `.atr` in the archive was not traced).

### 10.2 The table

| function | address | verdict | port / note |
|---|---|---|---|
| AIEl load | 0x12b8b0 | added | `ElementDb::new`, `MapAi::from_crp`: records copied, henchman/vehicle/rider health × the difficulty health factor, slot defaults 32/6/16 from `HENCH_/VEHICLE_/HELI_SLOTS_AVAIL`. The snow2a_mis4 `paris_hench` → `junglesnow` swap is left to the caller (it needs the rendername). |
| runtime init | 0x12af48 | added | radii 200 / 220, period 1. |
| range check | 0x12afb8 | added | `range_check`: in range (< 200) → period 1; out → period +1 up to 4, and a live element beyond 220 despawns; a held live element despawns; both only when not persistent (disassembly 0x12b000-0x12b088). |
| period | 0x12b0d0 / 0x12b0e0 | added | constants. |
| measure | 0x12b440 | added | `measure`: distance, priority key `(dot+1)·0.5/dist`, the nearest-live stat, the damaged flag (< 4 wheels). |
| slot manager | 0x12bec8 | added | `ElementDb::tick`: wanted/candidate lists, fit test against the free slots, else a sort by key (FUN_002a8198 with FUN_0012dbf0, descending), persistent first, riders must fit, despawn the unkept, spawn the kept; mission flag 1 = no live hostile within 120. Called once per world frame by `FUN_00159d10`. |
| key compare | 0x12dbf0 | added | descending; ties stay in index order in the port (the game's qsort is not stable) [S]. |
| may despawn | 0x12dc18 | added | `may_despawn`: flag 0x4 clear. |
| spawn | 0x12cb08 | added | `ElementDb::spawn`: the difficulty filter (switch on +0x42), vehicles park traffic within box + 6, sim kind 2 with flag 8 else 4, helicopters in the heli pool, the kill counter (mgr +0x3dc) +1 + riders at the first spawn unless flag 0x40, riders with health > 0 seated (an unknown seat ends the seating), henchmen only for class 0/10/0x12 and with DAT_00326000 set. The world cap (FUN_002012d0: 36 vehicles, 48 with its third argument) that can make the create fail is not modelled. |
| despawn | 0x12c740 | added | `ElementDb::despawn`: flag 0x1 copies the live position back; a vehicle is gone at health ≤ 0, a missing wheel or its +0x3e8 word set, else respawnable; a helicopter is gone at health ≤ 0; a henchman by its character's query; the live counts drop with the riders. |
| despawn by id | 0x12da78 | reviewed | called by the helicopter code (0x139be8, 0x13b2a8); not ported. |
| despawn all | 0x12c448 | reviewed | mission teardown; not ported. |
| element index | 0x12c608 / 0x12c640 | reviewed | pointer → index helpers. |
| controller kind | 0x13f118 | reviewed | sim kind 0 → 0x13f618 (parked), 2 → 0x13f748 (state 5 chase, +0x12c = `HAS_TURRET`), 3 → 0x13f868 (traffic); kind 4 gets no AIVehicle controller. |
| traffic set | 0x13fae0 | added | `traffic_types`: the 7-name table, `MAX_TRAFFIC` (20) / `NUM_TRAFFIC_TYPES_PS2` (`max` if < 8 else 7), random type slots when < 7, a colour draw per car. The attribute behind the colour count (FUN_00196818) was read as `RENDER_NUMCOLORS` [S]. |
| vehicle ctor | 0x1862c0 / 0x1ff850 | reviewed | builds the car from the pvehicle attribute at +0x5c; `AiVehicleSet` maps names to `CarParams::parse_atr`. |
| command apply | 0x145ff8 | reviewed | the AICo → AIVehicle copy (decoded in `AiCommand`); the copy itself is not ported, its fields are inputs of the functions below. |
| target mode | 0x12e670 / 0x12e770 | reviewed | 0 = chase the player (manager handle 0), 1 = drive to the goal point; an element target when +0x50 names a live vehicle. |
| target speed, chase arms | 0x12e808 | **added** | `chase_target_speed`: profile 1 wait, 0xd pursue (follow distance, the 1.25 box sum against the player, the 0.707 pattern-4 alignment, 0.8 / catch-up / the 30 floor, the 25 lost-target speed, the cap × speed factor), 0xe escort, 0xf road with +0x158. `ai::target_speed` still returns `None` for these arms; this function is their port. |
| pursuit timers | 0x131770 (part) | **added** | `ram_tick`: every 5 s while the target is seen, `rand/65536 < +0x14c` → ram 3 s (1 s against the player); `sight_tick`: every 0.25 s, seen = `FUN_001330d0`; +0x208 = not seen. |
| sight | 0x1330d0 | added | `can_see`: ≤ 200 and the world ray (FUN_00215cc8) reports fewer than 2 hits. |
| pursuit aim | 0x131770 (rest) | reviewed, not ported | the aim point +0x1c0 (the target position; the pattern-3 lead `pos + vel·min(max(lead, 0.2), speed)` — the store of that sum is not visible in the decompile), the hazard scan (DAT_0032b780), the +0x268 block. |
| pursuit aim | 0x131770 (rest) | see §11 (the aim point is ported there; the lead store is visible) | the aim point +0x1c0 (the target position; the pattern-3 lead `pos + vel·min(max(lead, 0.2), speed)` — the store of that sum is not visible in the decompile), the hazard scan (DAT_0032b780), the +0x268 block. |
| weapons dispatch | 0x130050 | added | `fire_step`: alive, +0x170, +0x224, a target; profile 10 rockets, 9 guns, else guns / rockets / missiles by the car's flags. The turret aim `FUN_0012f170` is not ported (`turret_on_target` is an input). |
| machine guns | 0x12f730 | added | cone 0.99 or the turret on target, line of fire clear; reload `round(fps·rate)`, no floor; the flash words. |
| rockets | 0x12fa28 | added | cone 0.9, channels 6/5 alternating, a busy launcher holds the shot, reload `round(max(fps·rate, fps·0.5))`. |
| missiles / torpedoes | 0x12fd58 | added | cone 0.707, channels 8/7, the same reload rule. |
| line of fire | 0x12f530 | added | `line_clear`: blocked by another chase-state car (+0x6d < 4, not traffic) inside a 0.99 cone and nearer than the target. The shot direction the callers pass is not visible in the decompile; the port takes it as an input. |
| drive arms | 0x133f60 / 0x135f70 / 0x137310 / 0x1377c8 | reviewed, not ported | the pattern-specific steering (pursuit, block, avoidance). |
| drive arms | 0x133f60 / 0x135f70 / 0x137310 / 0x1377c8 | see §11 | 0x133f60 and 0x137310 ported in §11; 0x135f70 / 0x1377c8 reviewed. |
| retarget | 0x131080 / 0x133850 | reviewed, not ported | state 2. |
| riders | 0x1224b0 / 0x120b70 / 0x11e490 / 0x11ff18 / 0x11aca8 | reviewed, not ported | the seated AICharacter classes (window shooter, sunroof gunner, …); their aim and fire are the character layer. |

### 10.3 Unproven

- DAT_00339f4c is used as ticks per second (it multiplies seconds everywhere above); its value
  in play (60) was not read live. DAT_00339f54 is a tick counter; its writer was not found.
- The player "forward" in the priority key (DAT_00333ed8 → +4 → +0x20) is read as the velocity.
- Which difficulty number is which menu setting.
- Which section of the world `.atr` the engine reads (the port reads the first, world-named one).
- What moves a kind-4 vehicle (no AIVehicle controller is attached).
- Not ported: the per-pattern steering arms, the pursuit aim point, the turret, the riders'
  own shooting, the helicopter controller, the world vehicle cap, the wiring of all of this
  into the viewer (nf-view was out of scope).
- Live check worth doing: in Paris at the first chase, break on FUN_0012cb08 and read the
  record (+0x40 flags, +0x54 name), and on FUN_0012fa28 read +0x228 / +0x22c of the firing car.

## 11. The per-pattern steering, avoidance, turret, world cap, helicopter leaf rules (2026-11-21, branch `s/drive-ai-patterns`)

Port: `crates/nf-game/src/driving/patterns.rs`. Tests: `crates/nf-game/tests/driving_ai_patterns.rs`
(21; the last one reads the disc). Every row was read in `work/decomp/DRIVING.ELF/`; where the
decompile hides an argument or a vector order the disassembly was read (capstone over the disc
ELF, EE = file offset + 0x107080). Verdicts as §9/§10: **added** = no port code before this
branch, **reviewed** = read, not ported.

### 11.1 The patterns

AI +0x78 picks the drive arm in `FUN_00130508` [M]. The game has five patterns and no separate
"block" or "flee" pattern: backing off is the reverse logic of `FUN_00133f60` (11.2), and a road
is blocked by pattern 3/4 cars holding their carrot.

| pattern | aim (+0x1f0) | arm | AICo commands on the disc [D] |
|---|---|---|---|
| 1 hold | own position | `FUN_00133f60(0)` in state 5 only | 25 |
| 2 pursue | `pos + (+0x1e0)·lookahead` | avoidance (`FUN_00137310` in state 5, `FUN_001377c8` otherwise, when +0x154), then `FUN_00133f60(+0x8c)` | 12 |
| 3 road chase | the carrot (`FUN_00130140`) | state 4: the traffic path; else `FUN_00135f70`, `FUN_00133f60` | 9 |
| 4 road follower | the carrot | `FUN_00135f70` unless nav +0x94, `FUN_00133f60` | 52 |
| 5 scripted path | none | the sim copies the path object's transform and velocity (`FUN_001b18a8` / `FUN_001f7880`) | 55 |

Each element's spawn command is AIEl +0xf8, an `sr` reference `0x7372_nnnn`. **Corrected in
§12.1:** `sr#nnnn` is a record in the Map window whose payload names the target,
`CARP::<<Map>>::{AICo}::<byte offset>`, so the command is record `offset / 0xa0` (MIS01 element
0: `sr#1` → offset 8960 → record 56). The `nnnn - 1` reading first written here was wrong.

### 11.2 The table

| function | address | verdict | port / note |
|---|---|---|---|
| aim point | 0x131770 (aim block) | **added** | `aim_point`: profile 0xf → the target; mode 1 → the goal +0x1a8; no target → lost; a target moving at ≥ 5: pattern 3 adds `n(v_t)·min(max(lead, 0.2), |v_t|)` (disassembly 0x131cc4, EE `max.s`/`min.s`), other patterns `v_t·0.2`; +0x1d0/+0x1e0 from that point, +0x210 = n(nav+0x10 − pos); lost: pattern 2 → the target (or pos + forward), else the carrot, after the distance was taken. §10.2 said the lead store was not visible: it is (`vmulbc`/`vadd` into auStack_1c0, stored at +0x1c0). |
| pursue/road switch | 0x131770 (profile 0xd) | **added** | `pursue_switch` / `finish_rejoin` / `road_heading_latch`: pattern 3 → 2 when the road turns away from the target (`dot(nav dir, +0x1e0) < 0.5`) or a slow target is inside the follow distance; pattern 2 → re-join the road when the target is on its road (lane inside the half width, along > 0.7, own speed ≥ 10) or lost, becoming pattern 3 once `dot(fwd, +0x210) ≥ 0.7`; a 60-tick hold (+0x74). The re-join itself (`FUN_00225c48` + the 7.5 advance) is the caller's. |
| hazard scan | 0x131770 (DAT_0032b780) | reviewed | 30 hazard records; kind 4 tested with `FUN_00140218(5.0, …)`; a hit sets +0x168 and a one-second +0x164. Not ported. |
| stuck re-join | 0x131770 tail | reviewed | +0x15c counts ticks the actor's vtbl+0x18c is clear; at ≥ fps a non-hold car re-joins; profile 0xf with +0x21c clear re-joins forward once. Not ported. |
| dispatch | 0x130508 | **added** | `dispatch` → `Arm` (11.1). The gated stop stays `ai::gated_stop`. |
| chase drive | 0x133f60 | **added** | `chase_drive`: steer `dot(d, row0)` at `|s| ≥ 1` with forward motion, full lock opposite the side while reversing; ramp over 15 (floor 0.05); the +0x140 hysteresis (on at `ahead < −0.15`); the timed reverse +0x25c/+0x260 after a bump (one second, cut short in its second half once `ahead > 0.7`); the head-on rule against the player (the vtbl+0x9c query is made on the target: disassembly 0x1343d8) skips the probe and cancels the handbrake (0x134950: `a1 = 0` in the delay slot); stopping: below 1 release with the handbrake at ≥ 0.1, else throttle 1 rolling backwards / brake 1; reversing: brake 1 while still rolling forward, brake 0.5 up to 7.5. |
| bump probe | 0x133238 | **added** | `bump_probe`: needs +0xdc and +0xe0 == 6 (cleared on use; `a1` = +0x140 from 0x1345b4); forward ray `2·box_z`; reversing two corner rays at `±box_x`; blocked = ≥ 2 hits. |
| flying arm | 0x133f60 (+0x118) / 0x1349c0 | **added** | `fly_aim` (altitude hold: the target's height + `sin(phase)·bob`, clamped to [+0x120, +0x11c]) and `fly_drive` (yaw/pitch from `up × fwd` / `fwd × side`, ramp over 5, no hysteresis). +0x118 is the actor's +0xbc byte at controller creation (0x13f748). The bob height 10, or 2.5 for an `IS_HELI` car, and its phase come from `FUN_0012e468` pattern 4. `sin_turns` is `FUN_002cbd50`'s truncated series (0.0033 off at ±π, as in the game). |
| chase avoidance | 0x137310 | **added** | `avoid_chase`: 64 slots; every other non-traffic, non-target car on the course marks +0x80 over its width; the aim shifts by `best_slot − 32` (−0.5 for an even own width) along `n(d.z, d.y, −d.x)`. Arguments read from the disassembly 0x137388-0x1374e0 (the decompile lost the other car's position copy). |
| slot build | 0x1369a0 | **added** | `slots_init`: the two 32-unit side rays from the aim (the hit distances are inputs), `+round(|32 − i|·5)`, the centre slots 31..33 cleared. DAT_00399c70 = 32.0 [D]. |
| on course | 0x135b10 | **added** | `on_course`: ±5 height band, range `clamp(2·speed, 20, 35)` (15 against a faster car), ahead. |
| slot pick | 0x135cf0 | **added** | `best_slot` (disassembly-checked: the second argument is unused). |
| road lane pick | 0x135f70 / 0x135d88 / 0x1351d0 / 0x226f18 | reviewed, not ported | the pattern 3/4 version: slots from the road's lane mask (`FUN_00222060`) plus a lane bias, other cars marked ±0x40..0x80 by escort rules, the lane offset blended through `FUN_00226f18`. |
| traffic yield | 0x1377c8 / 0x135530 / 0x135978 / 0x13efc8 | reviewed, not ported | the junction yield (route flag 0x80, nodes with ≥ 3 links) and the car-ahead stop for non-chase cars; sets +0x194/+0x198/+0x19c. |
| turret | 0x12f170 | **added** | `turret_step`: wanted angle `acos(dot(+0x1e0, fwd))/2π`, `1 − a` when `(dir × fwd).y < 0` (the vopmula/vopmsub order decoded at 0x12f26c); `TURRET_RANGE` (string at 0x36b540 [D]) gives two arcs in degrees; slew 0.005 turns per tick, snap within 0.005, wrap; the wrap direction is used only when the wanted angle is ≥ 0.75 or ≤ 0.25; +0x238 = on target; the bone gets `round(angle·360)` on channel 0x10 frame 8. It feeds §10's `turret_on_target`. |
| world cap | 0x2012d0 | **added** | `world_cap`: 36 (48 with the flag); over it the farthest car not of sim kind 2 is removed, then the create needs count < cap; at exactly the cap nothing is removed and the create is refused. |
| helicopter update | 0x138ef0 | **added** (gate) | `heli_update_gate`: +0x58/+0x5c; outside states 2/9/10 with +0x78 clear the goal tracks the target; then the death check `FUN_0013b2a8`, else the state machine. |
| helicopter speed | 0x13b4c8 | **added** | `heli_advance`: `clamp(x·0.5, 10, 20)`. |
| flight control | 0x13b4f8 | **added** (lift, roll) | `heli_lift`: the goal height (ground + alt), roll +0x94 stepping 0.02 toward `dot·max/50` when higher than 2, lift +0xa0 `2·(min(Δy, 10) − v.y) + 9.8` × `clamp(rand·4/65536, 0.5, 1)` when higher than 2; **one driving-generator draw per call**. The yaw +0x98 (the `FUN_001fd548` matrix path) is not ported. |
| helicopter states | 0x139be8 / 0x139380 / 0x1387a0 / 0x138b88 / 0x1385d8 / 0x1386b8 / 0x13b2a8 | reviewed, not ported | the state machine on +0x7c (1/4 idle, 2 parked at +0xa0, 3 approach, 5 chase, 6/7/8 the road fly-by with the 150 / 35 offsets, 9/10, 16 dead), about 12 KB; only its leaf rules are ported. |
| rider gunner | 0x122a68 (class vtable 0x369200, ctor 0x1224b0) | **added** | `gunner_sweep`: ±2° per tick, bone `clamp(yaw − 55, −80, 30)`, yaw clamp [−60, 80]. Which seat the class fills and what drives +0x84/+0x160 were not traced. The other rider classes (0x120b70, 0x11e490, 0x11ff18, 0x11aca8) and every rider's shot are the character layer: reviewed, not ported. |
| path copy | 0x130508 pattern 5 | reviewed | transform from the path object (`FUN_0013e0b0` → +0x60), velocity = path +0x88, sim +0x40 = velocity × sim +0x4c. The path object is scripted data the port does not load. |

### 11.3 The disc test

`each_pattern_on_its_mission` (NF_ISO): for each pattern, the first ground vehicle whose spawn
command has it (with the §12.1 resolution all five are paris_henchbigbox elements on MIS01),
built from its own `.atr`, runs 600 ticks at 1/60 against a scripted player (40 ahead, 15 u/s, a
slow weave) through `aim_point` → `hostile::chase_target_speed` → `dispatch` → `avoid_chase` /
`chase_drive` → `Car::step`. Results (re-run on `s/drive-gunners-heli` with the corrected
commands): the hold car stays put with no pedal; the pursuer (profile 0xd, cap 20) ends 27 from
the player against 190 for a parked car; the road chaser drives 163, the road follower 25;
pattern 5 drives nothing. The helicopter elements (MIS11 helimine, MIS13A
jungleheli, MIS3 openheli; MIS01's AIEl has none) run `heli_lift` for 600 ticks: 600 draws, the
height settling at 34.6 for a 35 goal. Harness stand-ins [S]: the flat world at y = 0, the
nearest road piece by brute force, the traffic junction chooser instead of the hinted kind-3/4
one, no other cars, the velocity turned onto the heading (as `ai::AiDriver`), +0xa0 read as a
vertical acceleration.

### 11.4 Unproven / not ported

- Not ported: the road lane pick 0x135f70, the traffic yield 0x1377c8, the hazard scan and the
  stuck re-join of 0x131770, the hinted junction chooser of navigator kinds 3/4 (0x2235c8 and
  the 0x130140 direction hint), the helicopter state machine 0x139be8 and its helpers, the
  helicopter yaw, the rider classes other than 0x369200 and all rider shooting, the pattern-5
  path object, and the wiring into the viewer.
- What consumes the helicopter's +0x94/+0x98/+0xa0, and the sign of the pitch call vtbl+0xdc,
  were not read.
- ~~The AICo index base (`nnnn - 1`) is inferred from the reference range.~~ Wrong; see §12.1.
- `FUN_002cbd50`'s VU accumulation order is summed plainly.
- Live checks worth doing: in Paris at the first chase break on FUN_00133f60 and read +0x140 /
  +0x25c and the pedal calls; on FUN_0013b4f8 for a helicopter count the generator draws per tick.

## 12. Helicopters, rider gunners, the path object, the spawn-command reference (2026-11-21, branch `s/drive-gunners-heli`)

Port: `crates/nf-formats/src/driveai.rs` (the `sr` names, `MapRef`, `AiPath`, the AICo
placement/altitude), `crates/nf-game/src/driving/heli.rs` (the AIHelicopter controller),
`crates/nf-game/src/driving/riders.rs` (the seated gunners). Tests:
`crates/nf-game/tests/driving_ai_heli.rs` (16; the last two read the disc). Every row was read in
`work/decomp/DRIVING.ELF/`; where the decompile hides call arguments (all calls inside
0x139be8 / 0x139380) the disassembly was read (EE = file offset + 0x107080).

### 12.1 How a reference word maps to a record [M + D]

`{u16 index, "sr"}` words (`0x7372_nnnn`) name `sr  ` records of the same window (the Map
article's, 2787 of them on MIS01). Each `sr` payload is a symbol name that the loader resolves
by name (`FUN_002dc1b0` → `FUN_002cd5e0`, the pointer written back by `FUN_002d45d8`) [M].
The names are `CARP::<<Map>>::{<chunk>}::<byte offset>` for Map chunks and
`CARP::<group>::{<tag><hex index>}` for short-tag records [D]. So:

- AIEl +0xf8 (the spawn command handed to `FUN_00145fc8` at spawn, 0x12cb08:226-229) names
  `{AICo}::<offset>` on every element of the eight tracks: record `offset / 0xa0`
  (`MapAi::command_index`). `FUN_00145ff8` reads that pointer as an AICo record [M].
- AICo +0x80 on pattern-5 commands names `{AISp}::<offset>`: an AISp path record
  `offset / 0x70` (`MapAi::path_index`) [D, every pattern-5 command].
- AICo +0x50 / other references name `{AIEl}::<offset>` (`/ 0x100`) or `{AISp}` [D].
- `FUN_0012c608` / `FUN_0012c640` turn an AIEl record pointer back into an element index:
  `|ptr − db+0x10| >> 8` [M]. `FUN_00145ff8` also writes element +0xf8 = the command when the
  command's +0x7a is not 9 or 10 [M].

The exact site that rewrites the AIEl +0xf8 word into the resolved pointer was not located (the
`sr` record's own +0xc gets the pointer; the field copy into the element is not seen in
0x12b8b0, which copies the chunk verbatim) — the mapping above is the names' [D].

### 12.2 Where the Paris helicopter comes from [M + D]

MIS01's AIEl has no helicopter element and no AIHelicopter runs there. The MSet payload's name
is `helicopter` [D]; `FUN_00204d18` keeps it (only `db5`/`z8` are replaced by the player-car
choice), `FUN_002051f8` returns it, and `FUN_00176658` builds **the player's vehicle** from it
(`FUN_001ff850(sim, 0, name, …)`, then `FUN_00195a08(*DAT_00339f5c, mgr+0x378)`) [M]. MIS01.VIV
ships `data\sim\attrib\pvehicle\helicopter.atr` and `data\car\model\helicopter.crp` [D bigf].
`FUN_00186878` picks the helicopter physics class (`FUN_002f1428`, 0x230 bytes) for a sim named
`helicopter` or `helivanquish` [M]. That flight model is not ported. `FUN_00125860` and
`FUN_00119e20` test the player actor's name against `helicopter` [M; what they change was not
followed].

### 12.3 The AIHelicopter controller [M]

Pool of `HELI_SLOTS_AVAIL` (16) objects, 0x130 bytes, built by `FUN_00138220` in
`FUN_0013e688`; vtable 0x36b888 (0x13bd30, 0x138280, reset 0x1382f0, update 0x138ef0, set-path
0x13b468, release 0x13b498). A spawned helicopter gets one through `FUN_0013fa28` (+0x58 = 1; a
radar object unless the actor's +0x13c is set); `FUN_00145ff8` hands it each command through
`FUN_00138fa8` and sets +0x5c. The update needs both.

| function | address | verdict | port / note |
|---|---|---|---|
| reset | 0x1382f0 | **added** | `Heli::reset`: state 1, mode 1, +0xd4 = −1, +0xd6 = 0xff. |
| command apply | 0x138fa8 | **added** | `Heli::apply_command`: state = AICo +0x7a (10 when 0); +0x78 hold; mode = +0x79 when set (5 attaches the +0x80 path, `FUN_0013dfd8`); +0x80/+0x84 = +0x5c/+0x60; rockets `(char)(int)+0x68`, 0xff in state 10 when 0; +0x40 → +0x90 (states 9/10) else the goal; state 2 → mode 1, the body at the goal with the +0x00 rows; mode 1 → 2; radius `max(|+0x4c|, 5)`, +0xdd = sign ≥ 0; altitude `max(+0x58, 3)`; interval +0x64; timers 0; +0xd6 = 0. |
| set path / release | 0x13b468 / 0x13b498 | **added** | `set_path`, `release`. |
| update | 0x138ef0 | **added** | `Heli::update` (the gate was §11's): goal = the target outside states 2/9/10 with +0x78 clear; `FUN_0013b2a8`; the state machine. |
| death check | 0x13b2a8 | **added** | `death_check`: +0xde clear: a proximity mine (actor +0x13c) within +0x140 of the target's box blasts and counts; +0xde set: health handed on (`FUN_00195a58`), despawn by actor (`FUN_0012da78`), +0xde = 0. |
| state machine | 0x139be8 | **added** (6/7/8 reviewed) | `Heli::step`. Dead → 16. 1/4: movement(1, 0). 11: +0x78 = 1, movement(0, 0). 2: parked at the goal, velocity 0. 3: goal = target (+ its velocity × 2·radius with +0xdd), movement(1, 0), the gun at 65 while +0xdc ≠ 0 and (outside the radius or no lead). 5: movement(0, 1), pitch = dot(+0xc0, flat forward). 9/10: movement(1, 0), face the target (+0x78 clear) or the post (0x13a078 branch, a1 = s4+0x90), pitch = dot(+0xc0, flat side), aim = +0xb0 when that is under 0.2 else the forward row; 9 the gun, 10 the rockets, range 1000, at the post +0x90. 16: the crash below. 6/7/8 (the road fly-by, 150 / 35 offsets, ~2 KB): reviewed only — **no AICo record on the disc sets them**. 12-15 have no arm. |
| crash | 0x139be8 case 16 | **added** | `crash`: mode 1; pitch −0.1 (floor −1), roll +0.01 (cap 1), lift ×0.99 (cap 0.25), actor +0x74 −2; impact = on the ground with contact, or the sweep ±n(fwd)·(sim+0x2c)/2 hits, or its front end is at or below the ground; then the blast, +0xde + 1, velocity ×0.35, roll 0. Because `FUN_0013b2a8` despawns as soon as +0xde ≠ 0, the case's own +0xde ≠ 0 arm (spin +5, a quarter-second wait) is not reached from the update (0x139be8's only caller is 0x138f8c). |
| movement | 0x139380 | **added** | `Heli::movement(a1 = yaw boost, a2 = heading blend)` (0x1393a0/0x139394: a1 → s2, a2 → s3). Mode 2: speed `min(50·k, cap)`, 0 inside the radius (the `max(d/r·0.75, 1.01)·50` term is never below 50); flight(0, speed, goal, boost 0); pitch = dot(+0xc0, flat side). Mode 3: speed by state (5: `min(|v_t|·k, cap)`, 0 inside; others with +0x78 clear: inside `min(|v_t| if dot(v_t, +0xc0) > 0, cap)`, outside `min(max(|v_t|·k, 25), cap)`; +0x78 or 9/10: cap, 0 inside); the target's road `FUN_00135018` → +0xd4; outside the radius a re-join when the held road runs away from the goal or the route query differs (`FUN_00225c48`, `FUN_00225358`, `FUN_00224e00(adv)`); a carrot nearer than `adv = clamp(speed/2, 10, 20)` advances `2·adv − d`; flight(+0x11c, speed, carrot, a1); pitch = dot(+0xc0, flat side), or with a2 inside the radius dot(+0xc0, (fwd + flat side)/2). Mode 5: the path pose (below). |
| face | 0x1385d8 | **added** | `face`: +0xb0 = n(p − pos), +0xc0 = n(flat). |
| flight control | 0x13b4f8 | **added** (full) | `Heli::flight`; the yaw is new (§11 ported lift and roll). Rows decoded from the VU code: `FUN_001fd5b8` → `FUN_002cc230` = the quaternion's first matrix row (side), `FUN_001fd598` → `FUN_002cc350` = the third (forward); the transposed `FUN_001fd548` matrix times the velocity gives `dot(side, v)`. Yaw: input = mode 3 outside state 5 `dot(probe carrot − pos, flat side)` (navigator +0x8c, kind 5, re-joined along the forward row), else `dot(flat(goal − pos), flat side)`; with max > 0 (`__extendsfdf2` + `__gtdf2`) and the boost `8·(input − 0.5·dot(side, v))`; state 5 `input` while slower than max else 0; otherwise `−dot(side, v)`. |
| gun | 0x1387a0 | **added** | `Heli::gun`: alive, +0x120 ≥ 0, the +0x102 timer; in range: +0xf0 = n(aim + 5·dir); inside 0.707 with a weapon: spread (three draws; level aim x/z ±(r − 0.5)·0.025, y −r·0.15; downward aim ±(r − 0.5)·0.05 each), the shot `FUN_001a27a0(w, 9, 10)`, the timer `(short)(int)(fps·interval)`; the firing flags (ai +0x4c → +0xb0/+0xb4) = inside the cone. |
| rockets | 0x138b88 | **added** | `Heli::rockets_fire`: held in state 10 with no rockets; the +0x100 timer; in range with speed ≤ 5 or aim.y ≤ −0.05; inside 0.8434 of the flattened aim; launcher 8 / 7 by +0xfe when `FUN_001a2d10` says free; +0x110 = target; +0xdc − 1 (wrapping outside state 10), timer `max(fps·interval, fps/2)`, toggle. |

Disc facts [D, `each_helicopter_for_1200_ticks`]: 34 helicopter elements on MIS11 (helimine), 3 on
MIS13A (jungleheli), 3 on MIS3 (openheli); MIS4's subcopter elements do not set +0x77. Every
helicopter spawn command is state 1 or 10 with mode 2 or 5; across the three tracks the only
command with +0x7a in 2..9, 11 or 16 is MIS3 record 14 (state 3, mode 3, altitude 20,
rockets 0.1 → +0xdc = 0, so that approach never guns). Records with +0x7a 13/14/15 are the
ground profiles; a helicopter given one would idle (no arm). The proximity-mine test reads the
actor's +0x13c; its writer was not traced (it is not the AIEl record).

### 12.4 The rider gunners [M]

`riders.rs`. Seats (0x12cb08:303-385): 3 window → class 0x369200 (ctor 0x1224b0, +0x78 0xd),
4 sunroof → 0x368f08 (0x120b70), 0xe snowmobile → 0x368950 (0x11e490), 0xf vehicle gun and
0x10 driver → 0x368c18 (0x11ff18 with 1 / 0), 0x11 Bond's seat → 0x3681f0 (0x11aca8); only 3,
4 and 0xe get the rider's weapon. Base `FUN_00118720`: range² 5625 (75), clip 15, 60 ticks
between shots, ready 1; skill 2/4 → range² ×4, 3/5 → ×9. Sunroof weapon clamp 1..2: 1 → clip 25
(state 0xb), 2 (bazooka) → clip 0 (state 10).

| function | address | verdict | port / note |
|---|---|---|---|
| target pick | 0x122e20 et al. | **added** | `pick_target`: the car controller's +0x1a4 target (vtbl+0x5c, +0x58 set, `FUN_0013e0b8`) else the player; the vehicle gun always the player. |
| window sight | 0x122e20 | **added** | `window_sight`: d² ≤ range², heading in (−0.25, 0.3888889) turns. |
| snowmobile sight | 0x11f528 | **added** | `snow_sight`: (−1/3, 1/3), the heading stored at +0x164. |
| sunroof sight | 0x121818 | **added** | `sunroof_sight`: range, and neither other rider (+0x170/+0x174) up. |
| vehicle-gun sight | 0x120478 | **added** | `vgun_sight`: range only. |
| aim test | 0x123058 / 0x11f768 / 0x1219c0 / 0x119290 | **added** | `aim_test`: ±1/180 turn; +0x84 = heading > 0 (the sweep's direction, §11's `gunner_sweep`). |
| reload test | 0x119270 | **added** | `needs_reload`. |
| window acquire / tick / fire | 0x122780 / 0x122d28 / 0x122920 | **added** | aim hold fps·300 on acquiring; the countdown; fire on the cadence with or without the aim test, one draw `rand·5 >> 16`, no reload. |
| snowmobile fire | 0x11e9a0 | **added** | one draw per shot; the animation index 0 unless on target. |
| sunroof fire | 0x120fe0 | **added** (fire arm) | reload from the clip; fire only on target, index `(rand·6 >> 16) + 6·weapon`; the `FUN_00121c10` branch (state 0x10) is an input. |
| the round | anim callbacks (0x122358, 0x120a18, 0x11e2f0, 0x11fdc0) | not traced | where the shot spawns and flies, its spread and damage. |

### 12.5 The scripted path object [M + D]

AISp records (0x70): +0x00 placement rows (position in row 3), +0x40 an `sr` reference to a `pt`
record (`{pt  NNNN}`), +0x44 the track name (`path_mine45L`, …) [D]. `FUN_0013dfd8` builds the
follower (`FUN_0013da80`, 0xb0 bytes): +0x4c = the `pt` pointer, +0x60 = an EAGL animation player
(`FUN_001b1c20`) whose +0xd0 is the name, +0x70 = the placement (`FUN_002db690`) [M]. Mode 5 / pattern
5 copy each tick: rows = player rows × +0x70 (`FUN_002cbb88`), position = player +0x30, velocity
= +0x88 [M 0x139380 case 5, 0x130508]. The `pt` records start with an f32 300.0, a u32 2 and the
name at +0x18 [D]; the track format and its playback are EAGL's and are not decoded.
`heli::PathPose` is the seam.

### 12.6 The disc tests

`spawn_commands_and_the_paris_helicopter`: every element reference on the eight tracks resolves
to an AICo record and every pattern-5 command to an AISp path; MIS01 element 0 → record 56; MIS01
has no helicopter element and its MSet name is `helicopter`. `each_helicopter_for_1200_ticks`:
each helicopter element of MIS11/MIS13A/MIS3 with its spawn command, and MIS3 record 14 applied at
tick 300 to MIS3's mode-2 helicopter, against a scripted player for 1200 ticks. Path helicopters
make no draws; hover/road helicopters exactly one per tick (plus three per gun shot); the MIS3
approach goes 1 → 3 and never guns; rockets only from state 10, launchers alternating. Stand-ins
[S]: flat ground 30 below the spawn; the integration of the four flight inputs (vertical
acceleration lift − 9.8, yaw rate = yaw clamped to ±1 rad/s, the flat velocity easing toward
forward × roll·50) since the physics class is not ported; the navigator a `RoadNav` joined on the
nearest piece with the traffic chooser; the path pose held at the AISp placement; launchers busy
30 ticks after a shot; the trigger that issues MIS3 record 14 (and to which element) is a guess.

### 12.7 Unproven / not ported

- The helicopter physics class (`FUN_002f1428`): how +0x94/+0x98/+0x9c/+0xa0 move the body.
- States 6/7/8 (no disc command sets them), the EAGL path playback, the rider rounds.
- What sets the actor's +0x13c (the proximity mine) and +0x30 (the crash contact); the
  proximity position is read as the target's (`FUN_00195808(FUN_00201ae0())` not followed).
- That `FUN_002c0618` is `atan2(a, b)` with the argument order used by `riders::heading_turns`.
- Whether the full matrix of `FUN_001fd548` has the same first row as `FUN_001fd5b8`.
- Which trigger issues which AICo record to which element (the `Trgr` layer).
- Live checks worth doing: in MIS3 break on FUN_00138fa8 and read the record pointer (is
  record 14 ever applied, to which element); on FUN_001387a0 in MIS13A count shots per second.

## 12. Road network logic (2026-11-21, branch `s/drive-roads`)

Port: `crates/nf-game/src/driving/road.rs` (curve accessors, `bezier`, `Junction`, the route
cap) and `crates/nf-game/src/driving/ai.rs` (the navigator's curves and lane blend, the kind-3/4
choosers, the lights and the yield, the road grid, the spawn director, the respawn, a headless
`TrafficWorld`). Tests: `crates/nf-game/tests/driving_roads.rs` (21; the last three read the
disc). Every row was read in `work/decomp/DRIVING.ELF/`, with the disassembly where the
decompile drops arguments (0x224e00's chooser calls, 0x130140's `FUN_0013dfc8` arguments,
0x13c798's box subtraction, 0x1377c8's radius `max.s`). Verdicts as §9. The driving generator is
the only random stream touched; nothing here draws from the on-foot stream.

### 12.1 Curves [M + D]

`FUN_00222468` evaluates a straight piece as `A + (B − A)·t`; a curved one (flags 0x4000) goes to
`FUN_00222d40`, which calls `FUN_001a8380(curve, A, rs, B, rs + 0x10)`. 0x1a8380 stores its 2nd,
3rd, 5th and 4th arguments as P0..P3, so the centre line is the cubic through A, rs+0x00,
rs+0x10, B. `FUN_001a85d0` evaluates `[t³ t² t 1] · basis · P` with the basis chosen by the
curve object's +0x60 (`FUN_001a7dd0`): every road curve is built by `FUN_001a7ba0` (+0x60 = 2),
which selects .data 0x37e4a0, the Bezier matrix (read from the disc ELF by the test). The other
basis (0x37e4e0, ×0.5, Catmull-Rom) belongs to kind 1, which no road curve uses.

The navigator keeps its own curve (nav+0x90): P0/P3 are its lane points +0x50/+0x60, P1/P2 are
+0x70/+0x80 from `FUN_00226788` → `FUN_00226538`: the control point at the start (toward B:
rs+0x00 by node A; toward A: rs+0x10 by node B) or the end, moved off its node by
`n(ctrl − node) · |ctrl − node| / max(rs+0x2c / max(|P0 − P1|, 0.01), 0.01)` from the lane end
point. The carrot's progress is still measured along the chord |P0 − P1| (`FUN_00224e00`).
On Paris all 105 curved pieces are the junction connectors (flag bit 0) [D].

### 12.2 The table

| function | address | verdict | port / note |
|---|---|---|---|
| curve eval | 0x222468 / 0x222d40 / 0x1a8380 / 0x1a85d0 / 0x1a7dd0 / 0x2cbb88 | **wrong → fixed** | `RoadNet::seg_point` / `curve_point`: the port had a quadratic through rs+0x10 alone and the navigator evaluated curves straight. Now the Bezier through A, rs+0x00, rs+0x10, B. |
| curve ctor | 0x1a7ba0 / 0x223338 | reviewed | +0x60 = 2 (Bezier); the navigator's curve object. |
| lane controls | 0x226538 / 0x226788 | **added** | `lane_ctrl_point`, `RoadNav::c1` / `c2`. |
| lane points | 0x226a28 / 0x2267d8 | correct | unchanged (§9). |
| join | 0x2257c0 | **fixed** | the controls and the Bezier carrot on a curved piece; +0x9c/+0xa0 set with +0x98. |
| road advance | 0x224e00 | **fixed** | the lane blend step on both paths (0x227058 with the rest of the piece before the pick, then with the step), the rebuilt controls, the Bezier carrot; the chooser by kind (1, 3/5, 4). Kind 2 (0x2247b0) stays unported. The chooser arguments (a1 = the hint vector, a2 = the target piece) are from the disassembly at 0x224e2c / 0x224e54 / 0x225128-0x225138. |
| lane blend | 0x227058 | **added** | `RoadNav::blend_step`. |
| lane change start | 0x226f18 | **added** | `RoadNav::start_lane_change`. |
| kind-3/5 chooser | 0x223a30 | **added** | `next_segment_chase`: slot +0xb0, dead end, plain link, the target routing through the exit junction's neighbours and roads, else the best hint dot. |
| kind-4 chooser | 0x2235c8 | **added** | `next_segment_follow`: slot, dead end, the 2-link and the one-draw 3+-link picks among links that keep the travel orientation, the hint pick when +0xb1/+0xb2 is set. |
| hint source | 0x130140 | reviewed | the carrot passes AI +0x1fc as the target piece and, as the hint, `n(target − car)` (profile 0xd with a target), `n(target − car)` / `n(car − target)` for nav +0xb1 / +0xb2, else AI +0x1e0. The caller side is not wired (no chase brain drives a kind-3 navigator in the port yet). |
| lights tick | 0x13ef40 | **added** | `JunctionLights::tick`: phase + 1 every 300 ticks; run once per world frame by 0x1472e0. |
| lights test | 0x13efc8 | **added** | `JunctionLights::may_go`: objects within 50 and less than 5 below hold all roads; the last 150 ticks of a period are all red; else road `phase % n` goes. One phase is shared by every junction. |
| lights reset | 0x13e688 / 0x13e840 | reviewed | phase 0, deadline now + 300 (`JunctionLights::new`). |
| yield | 0x1377c8 | **added** | `traffic_yield`, called by `AiCar::step_with` when a world is given and +0x154 is set (the ctor default, 0x12ded8:58). |
| blocked test | 0x135530 | **added** | `path_blocked`. |
| stop short | 0x135978 | **added** | `stop_short`. |
| state-4 drive | 0x130508 | **fixed** | the +0x198 carrot hold and the +0x154 yield on the aim before the repeat test and the drive control. The actor's vtbl+0x17c query is taken as 0. |
| grid cell | 0x1f8908 | **added** | `cell_of`. |
| road grid | 0x21cf30 / 0x21cf80 | **added** | `RoadGrid::parse_crp` / `cells_in_box`: CGrd +0x00 min x, +0x08 min z, +0x14 1/cell, +0x18 rows, +0x1c columns; cell = the `cn` record whose count field is its index; the road list at payload +0x07 (count) / +0x0e (byte offset from +0x10) [D Paris: 108 × 162 cells of 24 units]. |
| spawn list | 0x13d618 | **added** | `TrafficDirector::refresh`: every 0x59 ticks, the pieces of the cells under the box player ± 150, deduplicated, filtered (flags 0x2/0x1/0x8/0x8000/0x80, 0x1020 both, 0x100 with a trigger id), at most 50. |
| spawn pick | 0x13ca60 | **added** | `TrafficDirector::pick`: draw a piece, route cap rr+0x2e < rr+0x2c, `FUN_0013c5e8` (second draw for t), the scratch kind-1 join, the 4-cell (8 for non-traffic) spacing to enabled vehicles, ≥ 75 from the player, counter + 1. |
| spawn lane | 0x13c5e8 | correct | `spawn_lane_occ` takes the runtime occupancy; the t draw is now in `pick`. |
| out of range | 0x13c860 | **added** | `out_of_range`: > 150 with the view test off; > 75 and outside the 0.65 cone with it on; never within 75; never while spawning is off. |
| placement | 0x13c798 | **added** | `placement_ok`: only beyond 150 + box_z of the player. Traffic therefore appears only outside the 150 ring and, with the view test on, out of view. |
| respawn | 0x132808 | **added** | `respawn_step`: park a live car that is out of range past its hold, pick a spot below the cap, join it, keep it unless seen, place it when `placement_ok`, profile 0xc / pattern 3 / kind 1, live + 1, hold 5 s. |
| park | 0x132ba8 | **added** | `TrafficDirector::park`: (0, 0, 40000 + slot), slot + 50 up to 5000, profile 1, pattern 1, live − 1, route count − 1. |
| state-4 tick | 0x132d80 | **added** (route count) | `TrafficDirector::count_route`. The +0xe4 stop-and-wait needs the actor's vtbl+0x17c query (not read). |
| traffic create | 0x13f868 | reviewed | state 4, enabled, nav kind 1, +0xd8 = 4 (3 for one model list), and parked at once (0x132ba8): `TrafficWorld::new` starts every car parked. |
| pedestrian spawn | 0x13d3f0 / 0x14eea8 / 0x126c98 | reviewed | the other user of the list (rr+0x2d/+0x2f); not traffic. |
| lane pick | 0x135f70 | **added** (§12.5) | `join::lane_pick`. |
| lane pick helpers | 0x135d88 / 0x135cf0 / 0x135b10 / 0x1351d0 / 0x222060 | **added** (§12.5) | `lane_costs` / `best_window` / `makes_room_for` / `scratch_join` / `lane_mask`. |

### 12.3 The headless traffic world [S where noted]

`TrafficWorld::step` runs one world frame: the lights tick (before the cars; the order against
the cars is not proven), then each car in list order: the respawn, the route count, and for a live
profile-0xc car the brain (with the yield) and `Car::step`. Parked cars are not driven (pattern 1's
drive is not ported). The vehicle list the yield and the spacing read is updated car by car, as
the game's list walk sees it. Stand-ins: the view test of the respawn (`FUN_001c65b0`, a sphere
against the view frustum) is a caller closure; the two world flags (+0x350 view test, +0x3cc
spawning off) are inputs; the vehicle extents +0x4c0/+0x4c8 are the model's `Base` half box once
`TrafficWorld::set_extents` is called (§12.5; the earlier reading of them as `FUN_00210938`'s
inverse inertia was wrong: that triple (sp+0x90, copied to sp+0x20) is param_9; param_10 is sp+0x30, which
`FUN_001c9778` fills).

Disc runs (Paris, 3600 ticks): with the director live and a player driving the network at 20/s,
9 cars were placed (all outside 150 and out of view) and drove 249 units; with 20 cars put on
pieces spread over the city and spawning off, the cars covered 3,669 units, entered 109 pieces
and spent 22,374 of 72,000 car-ticks held at red lights (each road is green for 150 of every
n·300 ticks, n = 3 or 4 at the Paris junctions). Every carrot stayed on a lane line (within the
widest lane offset, 15.0). Car bodies stayed within 24.4 of a centre line; 1,490 car-ticks were
more than 4 past the widest lane line: the rail physics turns no tighter than
`wheelbase / tan(MAXSTEERING)` = 9.4 for the hatchback and swings wide at Paris's 4-to-8-unit
connectors before the carrot pulls it back. That is the port's physics stand-in (§9.3), not the
road logic.

### 12.4 Not ported / unproven

- The lane pick and the nearest-road join are ported in §12.5. The caller side of the kind-3/4
  choosers is ported there in part: +0xb0/+0xb1/+0xb2 come from the AICo command (+0x7e/+0x84/
  +0x85, `FUN_00145ff8`), +0x1fc from `FUN_00131770` (`chase_target_piece`); the rest of
  `FUN_00131770` (its tracking flag) is not.
- The `DAT_003339a8` objects the lights check; `FUN_001c65b0`; the world flags +0x350 / +0x3cc;
  the actor query vtbl+0x17c; AI +0x94 (gates the respawn and the lane pick; taken as 0).
- `DAT_00339f54` (the world tick) and `DAT_00339f4c` (ticks per second) were not read live.
- Live checks worth doing: in Paris, read DAT_003c828c / DAT_003c8290 over a few seconds (expect
  +1 every 300 ticks), and break on FUN_0013c798 to see a placement distance (expect > 150).

### 12.5 The nearest-road join, the lane pick and the car box (branch `s/drive-road-join`)

Port: `crates/nf-game/src/driving/join.rs`; tests `crates/nf-game/tests/driving_road_join.rs`
(27; four read the disc). Verdicts as §9. Nothing here draws a random number. The disassembly was
read wherever the decompile drops VU lanes or `max.s`/`min.s` (R5900 words `4601xx28` / `29`).

| function | address | verdict | port / note |
|---|---|---|---|
| nearest piece | 0x225490 | **added** | `nearest_piece`: one grid cell only (`nearest_cell`: x on the columns, z on the rows, truncation, a negative or overflowed index to 0, past the end to the last cell); pieces below the RNhd count, once each; 0x100 never; kind 4 only flag-8 pieces of its group (nav +0x04 against rs+0x5a), other kinds never flag 8; kind 5 only flag-0x8000 pieces, other kinds never; the clamped chord projection, moved onto the Bezier on a curved piece; 3-D distance, first of equals. An empty cell returns piece **0** with no point (the game then joins piece 0 with a stale t; the port reports `EmptyCell`). |
| chord point | 0x222968 / 0x222a30 / 0x2223b0 / 0x221888 | **added** | `project_clamped`. |
| join | 0x225c48 | **added** | `RoadNav::join_nearest`: the end by `dot(dir, rs+0x40) ≥ 0` (a zero dir counts as 1; kind 4 without +0xb1/+0xb2 always toward B); the offset = the distance to the piece (lateral only when behind P0), signed against `(d.z, d.y, −d.x)`; kind 1 `snap_lane(max(2.5, off))` (`max.s` at 0x2263e8), so traffic always joins a lane on its own side; kind 2 past the outer lane; kind 4 clamped to the road widths (`max.s` 0x22627c, `min.s` 0x226294); kinds 3/5 keep the offset; param_4 = 1 keeps the centre line. Then the lane points, the controls, t on the lane line, the carrot. |
| lane snap | 0x221f20 | **added** | `snap_lane`. |
| road widths | 0x221ac8 / 0x221cb0 | **added** | `road_width`: flag 0x8008 reads rs+0x00 / +0x04; a connector takes the widest plain link at either node; else 5 (0x10) + 2.5 (0x800) + 5 (0x200 / 0x400) + 5 per lane (0x1000: 2.5 per lane of both sides); 0 becomes 5. |
| cleared nav | 0x2233d0 | **added** | `RoadNav::cleared`. |
| reverse | 0x225358 | **added** | `RoadNav::reverse`. |
| pattern set-up | 0x12e468 | **added** | `nav_for_pattern`: state 4 → kind 1 (pattern 1 → 0); pattern 3 → kind 3 + join; pattern 4 → kind 4 + join, on the centre line when +0x154 is off; 1 / 5 → kind 0; 2 leaves the kind. The +0x124/+0x128 writes and the pattern-5 path `FUN_0013dfd8` are not ported. |
| target piece | 0x131770 (lines 466-517) | **added** | `chase_target_piece`: −1 unless tracking, pattern 3 and a profile other than 0xe/0xf; unchanged while +0x94 holds; no target car: the scratch nav +0x114 (kind 3, or 4 for a kind-4 car) joins the goal +0x1c0; with one: that car's kind-3 scratch piece. The `FUN_00226c88` rejoin when the piece differs from the car's own is not ported. `chase_hint` = +0x1e0 = n(+0x1c0 − own position). |
| nav slot / hints | 0x145ff8 | reviewed | AICo +0x7e → nav +0xb0 (when ≥ 0), +0x7f → nav +0x04 (the group), +0x84 → +0xb1, +0x85 → +0xb2; no other writer of +0xb1/+0xb2 in the decompile besides the clear in 0x2233d0. |
| other car's road | 0x1351d0 | **added** | `scratch_join`: re-joined at `pos + vel·0.25` when the stamp is 0 or 15 ticks old (kind 4: or a new group), turned to the asker's heading, and kept turned. |
| lane mask | 0x222060 | **added** | `lane_mask` / `lane_mask_index`; the tables [D .data 0x33a6d8 snow2a, 0x33a818 the other tracks, ten of 32 bytes; the test reads them from the disc ELF]. |
| cost map | 0x135d88 | **added** | `lane_costs`: the mask + `round(5·|slot − c|)`, capped at 255; 0xff slots kept. |
| best window | 0x135cf0 | **added** | `best_window` (its second argument is dead). |
| room test | 0x135b10 | **added** | `makes_room_for`: 3-D distance within clamp(2v, 20, 35), 15 for a car at least 1 faster, height within 5 unless both overtake, not behind. |
| lane pick | 0x135f70 | **added** | `lane_pick` (the doc comment lists every weight). The speeds are horizontal (`vmul.xz` at 0x136058). The other car's offset rounds half away from zero (`floorf` 0x2bfb50, `ceilf` 0x2bf970, the double floor 0x2bf460). AI +0xb7..+0xb9 are cost-map slots 31..33. |
| car box | 0x1862c0 / 0x1c9778 / 0x1f6e98 / 0x1c7940 | **fixed** | `car_half_extents` + `floored_extents`; `TrafficWorld::set_extents`. +0x4c0/+0x4c4/+0x4c8 are half the model box (min +0x10, max +0x20 of the object at model +0xc = `*in#0[+0x1c]`), floored at 0.001. On the disc that reference is the article's `Base` record [D every car: the `as` payload +0x00 is the `Base` tag]; that the loader resolves it to the `Base` payload is read from the layout, not from the loader. Traffic: hatchback (0.817, 0.776, 1.952) [D]. |

Disc: 200 points near Paris roads (DriveRng picks a piece, t, a side offset within ±12 and a
heading), each joined as kind 1 and as kind 3: all 400 join; each lands on a lane line of the
piece that a brute-force search of the cell also names; every traffic offset is a table lane of
the piece's own side; 138 of the 400 land on another piece than the one the point was made from
(a parallel piece or a connector nearer at that offset).

Unproven / not ported here: whether the player's own controller answers the lane pick's AI reads
(the vehicle list holds every vehicle); the actor queries vtbl+0x14c / +0x9c the weights read;
the writers of AI +0x148 / +0x204 / +0x208 / +0x118 / +0xd8; the tracking flag of
`FUN_00131770`; `FUN_00226c88`; nav +0x96. No live capture backs any of §12.5. Nothing in the
port calls `lane_pick` or `nav_for_pattern` per frame yet (no chase brain drives a kind-3/4
navigator).

## 13. The hostile runtime (2026-11-21, branch `s/drive-hostile-runtime`)

Port: `crates/nf-game/src/driving/hostiles.rs` (`HostileWorld`), plus AICo +0x93 / +0x94 in
`crates/nf-formats/src/driveai.rs`. Tests: `crates/nf-game/tests/driving_hostiles_runtime.rs`
(12; the last reads the disc). The runtime adds no game rule of its own: it owns the element
database (§10) and, per live element, the controller the game attaches, and calls the ported
functions of §10-§12 and `weapons.rs` in turn. Driving generator only.

### 13.1 What one `step` does

1. `ElementDb::tick` (FUN_0012bec8) with the player's position and velocity, the live elements'
   positions and health. Each Spawn becomes a live object as `FUN_0012cb08` creates it [M
   0x12cb08:152-260]: a ground vehicle is a `Car` from its own pvehicle `.atr`, at the record's
   +0x30 facing its +0x20 row, with an AIVehicle chase brain for sim kind 2 (flag 8) and none for
   kind 4 (`FUN_0013f118` has no kind-4 arm); a helicopter (+0x77) is a `heli::Heli`, attached
   (`FUN_0013fa28`); a henchman is held at its record. The health starts at AIEl +0x4c
   (`FUN_00195a08`). The spawn command +0xf8 is applied (`FUN_00145fc8` → `FUN_00145ff8`) and the
   vehicle spawn asks for the traffic within box + 6 to be parked.
2. Each live ground vehicle with a brain: `sight_tick`, `ram_tick`, `aim_point`, the profile-0xd
   `pursue_switch` / `finish_rejoin`, the pattern 3/4 `road_heading_latch`,
   `chase_target_speed`, the road carrot (`RoadNav::advance_with`, kinds 3/4 as `FUN_0012e468`
   joins them, with the 0x130140 hint), `dispatch`, `avoid_chase`, `chase_drive` (or
   `gated_stop`), then `Car::step`; the turret `turret_step`, `line_clear` and `fire_step`.
3. Each helicopter: `Heli::update`. Each seated rider: the window / snowmobile / sunroof sight
   and fire cadence (`riders.rs`), the window class with its `gunner_sweep`.
4. Shots: a gun / cannon burst is one shell (`weapons::delivery` damage with
   `ai_damage_scale(AI +0x131)`, the aim roll `aim_is_exact(AI +0x130)`, one draw); a rocket or
   missile channel (0x33a0b0 barrel lists: 5/6 Rocket, 7/8 Missile, 9 Gun, 0x48 Cannon) launches a
   `Projectile` (`launch_params` with an AI launcher) that `Projectile::step` flies at the player.
   A shell that lands and a projectile that reaches the player return a `damage::Hit` (kind 2;
   dent 0.25 for the shell as `FUN_0015de48`, 0 for the projectile) for `damage::inflict`.

`command(element, record)` applies a script's `EAICommand`; `activate` / `deactivate` are
`EActivateAIElement` (clear HELD, 0x1476b8) and `EDeactivateAIElement` (set HELD and despawn,
0x14b510). `draw_list()` gives per live element the model stem, the world matrix (rows right /
up / forward, translation row 3), the wheel centres, steer angle and roll angle, and a
helicopter's +0x74 spin; `projectiles` the rounds in flight. `models_needed(map, set)` lists each
vehicle of the track's set with its `RENDER_FILENAME`.

### 13.2 Read for this branch [M]

- `FUN_00145ff8` tail: +0x93 → AI +0x130, +0x94 → AI +0x131 (now `AiCommand::accuracy` /
  `damage_mod`); +0x86 → +0x119, +0x88/+0x8c → +0x11c/+0x120 (the flying car's hold, ceiling,
  floor: not used by the runtime; +0x118 is the actor's +0xbc byte, whose writer was not traced); +0x7b also puts the car at
  the record's +0x40 facing its +0x20 row (vtbl+0x164) after parking the overlapping traffic
  (radius = both boxes); +0x70 (unless 0 or 1.5) sets the velocity along the forward row; the
  same record is not re-applied (+0x88).
- `FUN_001476b8` (`EActivateAIElement`): clears HELD unless `DAT_00327c2c` is set, the element is
  a non-persistent vehicle and `FUN_0012c5d0` (db+0x50 < db+4) is false. No writer of
  DAT_00327c2c was found; the port clears HELD.
- `FUN_0014b510` (`EDeactivateAIElement`): sets HELD; a vehicle is despawned through
  `FUN_0012da78` → `FUN_0012c740`.
- `FUN_0012e468`: pattern 3 → navigator kind 3 joined at the car along its forward row
  (`FUN_00225c48(…, 0)`); pattern 4 → kind 4 (last argument 1 when +0x154 is clear) and the bob
  height 2.5 with `IS_HELI` (string 0x36b528) else 10, the phase from DAT_00327c38 stepping 0.25;
  pattern 5 → kind 0 and the path follower.
- `FUN_00182978`: a projectile's contact with the player's car calls only `FUN_00200088` (the
  explosion with the projectile's +0x718 damage, kind 2); the direct `vtbl+0x1ac` / `vtbl+0x1c(32,
  0, …)` pair runs only when the projectile is the player's (`FUN_00195c70`).

### 13.3 Stand-ins [S]

- The nearest-road join `FUN_00225c48` / `FUN_00225490`: the nearest of five samples per piece.
- The shell's flight (`FUN_001fffa8`): an exact shot hits; a wobbled one hits when its line passes
  within the player's box half length. The explosion's radius rule (`FUN_00159fb8` →
  `FUN_001847e8`): a projectile that comes within the player's box half length + 0.5 books its
  full damage once.
- The launcher channel busy test (`FUN_001a2d10`): 30 ticks after a shot.
- The helicopter physics class: the integration of §12.6; the EAGL path pose: none.
- The rider round is not traced: rider shots are events with no damage.
- `FUN_001351d0` (the road under the target): pattern 2 re-joins only when the target is lost.
  The bump probe's contact flag, the avoidance's side rays and the element target (AICo +0x50)
  are not used. The order of the element manager against the controllers in `FUN_00159d10`.
- On the rail stand-in (`FlatWorld`) the land controller's reverse rule (0x18cef8 brake-hold) is
  applied by the runtime and the steer mirrored while reversing; a `surface` ground gives the
  height (as the view does for its traffic).

### 13.4 The disc test

`mis01_and_mis4_for_3600_ticks`: a scripted player drives the road network at 20 u/s from the
track's start for 3600 ticks; each element is released when the player first comes within 200
of it (the mission triggers are not run). Checked every tick: a spawn is within 200 (+ the
1..4-tick range period) unless PERSISTENT; a despawn is beyond 220, a slot lost to priority or
HELD; live vehicles ≤ max(`VEHICLE_SLOTS_AVAIL`, live persistent) (the 0x12bec8 rule: persistent
elements keep their slots even past the budget), helicopters ≤ the heli budget, henchmen +
riders ≤ the hench budget; the database's live count equals the live list. Results on the rail
stand-in: MIS01 5 spawns (all persistent, the pursuers end within 2-36 of the player), worst
stray from the road 1.5 past the spawn's own distance, 113 rider shots, no vehicle weapon (no
MIS01 hostile `.atr` declares one [D]); MIS4 8 spawns, 4 despawns, worst stray 15.9, 494 shells,
240 damage calls of 2.5 (gun 10 × `ai_damage_scale(−75)` [D AICo +0x94]).

On the land physics (`CollisionGround`) the same run is reported, not asserted: a MIS01 pursuer
hits a kerb or wall about 4 s in, is thrown up (v.y 7.5), rolls over and falls through the
ground mesh (y −2400 by the end); a MIS4 car strays 140 (cause not traced). The MIS01 case is the
`Car::step` land model under AI inputs, not the runtime.

## 14. The elements in the mission script (2026-11-21, branch `s/drive-hostile-facts-2`)

Port: `HostileWorld::element_facts` / `script_elements` / `elements_for` / `trigger_cars` and
`hostiles::element_refs` (crates/nf-game/src/driving/hostiles.rs); the view fills
`script::Facts::elements` and `Facts::cars` from them every tick (crates/nf-view/src/driving.rs).
Tests: `crates/nf-game/tests/driving_hostile_facts.rs` (7), the script unit tests
`speed_condition_reads_the_car_that_ran_the_trigger` and
`element_rules_fire_from_the_element_facts`, `crates/nf-view/tests/driving_hostile_facts_live.rs`
(2, NF_ISO).

### 14.1 What the rules read about an element [M]

- A kind-4 selector holds the element's record pointer; `FUN_0012c640(DAT_00327c30, id)` turns it
  into the runtime record `db+0x14 + |id - db+0x10| / 0x100 · 0x50` with no state test.
  `FUN_0012af48` stores the AIEl record pointer at rt +0x20.
- Health (`FUN_00209820`): `*(rt+0x20) + 0x4c`, the AIEl record's health. The spawn points the
  actor's hit points at that float (`FUN_00195a08(actor, rec + 0x4c)`), `FUN_00195a58` subtracts
  and floors at 0, and the despawn `FUN_0012c740` never rewrites it: an idle element reads its
  loaded health, a live one its actor's, a wreck 0 for the rest of the mission.
- Position (`FUN_002098a0`) rt +0x30 and speed (`FUN_00209758`) rt +0x3c, written by
  `FUN_0012b440`: the live actor's position and |velocity|, else the record's +0x30 and 0.
- Properties (`FUN_00209aa0` → `FUN_0012b138`, strings 0x36b098..0x36b0b8): EMPTY = a vehicle whose
  riders' +0x14 healths sum to 0 (any state; no riders counts); BLOWOUT = live vehicle, not a
  helicopter, `vtbl+0x13c` (wheels on) ≤ 3 and no rider at health ≤ 0; SMOKED / OILED = the live
  vehicle's AIVehicle +0x168 / +0x16c; AWAKE = live, and for a vehicle health > 0 (a henchman by
  its character's vtbl+0x14). The port has no wheel loss and no smoke / oil writer: BLOWOUT,
  SMOKED and OILED read false [S]; the port's henchmen are held at their record and read AWAKE
  while live.
- Bearing (`FUN_0020ac70`, `FUN_00209958` for the matrix): only a live ground vehicle; d =
  normalised (player − rt +0x30), angle = acos(row 2 · d) in degrees, negated when
  cross(row 2, d).y < 0, then `FUN_0020abe0`'s window. That the libm call pair is acos is
  read from the call shape.

### 14.2 Deaths and removals

Nothing of the script runs when an element dies or is removed [M, D]:
- The AIEl / AICo records carry no `el` list (every reference word of the eight tracks' records
  names `AICo` at AIEl +0xf8, `AIEl` at AICo +0x50 or `AISp` at AICo +0x80) [D].
- The damage virtual `FUN_00189598` at the wreck only books mission statistics
  (`FUN_00206900` mgr +0x3c0, `FUN_00206960` mgr +0x3c4) [M]; no rule reads those words
  (`SRuleProgCounter` reads mgr +0x3e4 + 4·id).
- `FUN_00188200` despawns a wreck at once (`FUN_0012da78`) only for vehicles named "tunnel",
  "openheli" or "redheli" (with car +0x2c8 = 2 and `FUN_001a2cc0` false); other wrecks stay
  until the range manager drops them (state 2, gone) [M].
- The mission learns of a death by polling: `SRuleDeath` (health int == 0), `SRuleDamage` (lost
  more than the threshold since the last test) and `SRuleHealth` (crossing) on the element's
  record every frame they are evaluated.

Rule counts with an element selector [D]: MIS01 Death 13, Damage 4, Health 3, Property 11;
MIS4 Death 8, Damage 2, Range 14, Property 10; MIS11 Death 6, Health 4, Range 30, Property 1,
PlayerDir 3; MIS13A Death 7; MIS13B Death 12, Range 7, Property 1; MIS13C Death 10, Property 1;
MIS3 Death 7, Health 2, Property 3; RACE Death 1, Range 3.

### 14.3 The trigger walk

`FUN_0022f0c0` walks the 0x30 vehicle-manager slots whose +0x6e is 2 (set at creation,
`FUN_001f6e98`; a parked traffic car is 3, `FUN_00132ba8`). `FUN_001862c0` maps the creation type
to the kind byte +0x6d: 0 and 1 → 1, 2 → 2, 3 → 3, 4 and above (and below 0) → 2. The hostile
spawn `FUN_0012cb08` creates its ground vehicles as type 2 (flag 8) or 4, the traffic
`FUN_0013fae0` as type 3: so every AI element's car fires the flag-bit-3 triggers and the traffic
the bit-6 ones. `EFireTriggerSpeedCondition` reads the speed of the car in the event's walk slot
(`FUN_001ff088(0x339190, event+0x14)` +0x20), now ported. MIS01 has 615 bit-3 triggers, MIS4
240 [D]; many are the pursuers' route points (`EAICommand`, `EAIKill`, `EDeactivateAIElement`).

Not traced: the car-slot order (+0x46) the game gives the player, the traffic and the elements
(the port walks the player, the traffic, then the elements in element order); whether the
helicopter objects (`FUN_002003c8`) sit in the 0x30 car slots (the port leaves helicopters and
henchmen out of the walk).

### 14.4 MIS01's fail at the end of the sniper run

List 686 (trigger 685) changes to stage 19. Stage 19's R105 / R107 are `SRuleDeath` on elements 3
and 11 (AIEl 768 / 2816, the two henchman cars that start on the road): each disables trigger 644
and starts timer 4 (2 s); R113 (timer 4 below 1 s) calls stage 39, whose R106 fires trigger 712.
Trigger 644 (player, at (555, 0, 445) on the flight path) calls stage 42, whose R5
(`SRuleDifficulty` mask 6: difficulty 1 or 2) runs `EAIKill(element 23)`, `EPlayerLose` and
disables trigger 712 [D]. So on difficulty 1 or 2 the run fails when the flight reaches trigger
644 with both pursuers alive. Before this branch the view gave the rules no element facts, so
R105 / R107 could never fire and the view always lost there (tick 424 with the flight's own
speed).

