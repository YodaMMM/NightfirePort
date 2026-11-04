# trigger-glist-extents — the report

Task: parse the trigger cels' glist extents where the game places them (find the game's extent
read + the ref gate [M], re-pin 0a #614, 07000011 #235, 0c's ch-27 bay [D]); then make the
standing toucher's deep reach unconditional and retire the pos+1.26 box top, if the evidence
allows. Branch `s/trigger-glist-extents` cut from origin/master b21edd082.

## The headline findings

1. **The port's parse is already the game's placement — measured, not assumed.** The game keeps
   the glist at `obj+0xD8` and tests it in the object's LOCAL space (`Collide_Intersect`
   0x1ebaa8:106-107 copies `obj+0x90`, inverts it, and `RotPreTransVec`es the HITTEST's cap
   centres into that local frame; `Intersect_CylGeom` 0x109da0 then walks the glist's own
   collision boxes/tris — the vertex decode `s16·scale+origin` the port's
   `Collision::triangles_with_planes` already implements [M 0x109da0:208-244]). `obj+0x90` is
   stamped by `Trigger_Create` 0x1b1720:52/97 → `control_create_object` 0x1344e0:27
   (`View_RotTransMatrix(rotation, position)`) — the placement's own rotation/translation, the
   same matrix the port builds at `triggers.rs::build` (`view_rot_trans_matrix(p.rotation,
   p.translation)`). Measured against the fresh-boot snapshot `work/ee_start_0700000a.bin`
   (file offset = EE address; object list head 0x2705a0, `control_add_object_to_list`
   0x134698: head+0x14 = first; class-0x1d trigger objects): every STATIC trigger's `obj+0x90`
   translation row equals `obj+0x30` equals the placement's translation (e.g. the LOAD_LEVEL
   out 0x0b at (21.72338, 1.1248139, 19.78649) — identical in all three), and the only
   placement-vs-matrix deltas in the whole level are the KEY-SWEEP beams mid-sweep at the
   snapshot tick (out 0x22 = #614: matrix z 11.294829 vs the placement's 13.0287 — [D keys
   13.0287→10.27843→13.0287, 240-tick pass, the level-0a-ring-sweep-timing work]; out 0x26 =
   #625: matrix x 17.112177 vs 18.877008 — [D keys x 18.877→16.078→18.877]). The probe
   `crates/nf-formats/examples/probe_trigger_extents.rs` (committed) prints both sides.
2. **The "0a #614 band 0.9 under the west corridor's floor" premise is disproven.** The west
   corridor's floor is y ≈ 0, not 1.2 — [D ai_network nodes, docs/levels/0700000a.md ("the
   dropped-floor theory was disproven: the office floor there is y ≈ 0")]; the walkreal route
   itself walks the corridor at waypoint y 0.4 and calls #614 "wall to wall at ankle height"
   (`walkreal_0700000a_live.rs:537-549`). #614's parsed band y 0.2525..0.5164 is ~ankle height
   ABOVE that floor, and the game's own behaviour agrees ([L 2026-11-01: a walk fires, a
   0.9–1.3 takeoff clears]). The snapshot's obj 0x01ba7030 (out 0x22) matrix y = 0.385 = the
   placement's 0.3876277 [L] — the game places the beam exactly where the port parses it.
3. **The game's raw-refine discriminator is bit 0x40 of the glist flags — the port's
   `seg_beam` (flags == 0x645) is a narrower stand-in.** [M Collide_Intersect 0x1ebaa8:93-99:
   `*(uint *)(obj+0xf0) & 0x40` → `HITDATA+0x51 |= 2`, which routes the refine past the
   ref-point law to the raw per-triangle distance (asm 0x10a530-0x10a538); obj+0xf0 is copied
   from celglist+0x44 [M Trigger_Create 0x1b1720:95] = the port's `MapObject.flags`]. The
   committed census probe (`crates/nf-formats/examples/probe_glist_flag_census.rs`) buckets
   every trigger placement's target flags across 21 levels: bit 0x40 is set on the dominant
   0x50 bucket everywhere (plus 0x150/0x130/0x251/0x1140/0x1050/0x40/0x645); only 0x30/0x20/0x0
   lack it. 0x645 appears in exactly 3 levels (0a ×10, 0b ×8, 14 ×8). #235 (0x50), #274/#506
   (0x50) and the beams (0x645) are ALL bit-0x40 cels.
4. **The three pins, real positions [D]** (the probe prints them; the placement record + the
   collision block are the disc evidence):
   - 0a #614 (out 0x22, obj 540 "laserbeam", flags 0x645): placement translation
     (-21.985458, 0.3876277, 13.0287), rotation y π/2; parsed world band lo
     (-24.706913, 0.2525129, 12.895908) hi (-19.449589, 0.51642895, 13.173253) — matching the
     measured game line z 12.90..13.17 [L trigger-plane-records]; the y band is 0.25..0.52 over
     the corridor floor y ≈ 0 [D].
   - 07000011 #235 (out 0x66, obj 146 "Trigger Bond Moment", flags 0x50): placement
     (9.355237, 49.780663, 19.598518) rotation z −π/2; the local glist is a zero-x-thickness
     slab (y ±0.6586944, z ±0.70672417); parsed world band lo (8.696546, 49.780663, 18.891823)
     hi (10.013928, 49.780663, 20.305214) — a VERTICAL wall quad at y 49.78 (zero height). The
     standing segment fires it the game's way under the raw law: the vertical quad is PARALLEL
     to the vertical segment, so the plane-crossing degenerates to the CENTROID anchor
     [M 0x10a3f0-0x10a4fc] — a horizontal-only test, no pos+1.26 box top needed.
   - 0700000c ch-27 (out 0x1b) plates #274/#506 (obj 129 "Mission Trigger_Trigger Touch",
     flags 0x50, mask 8): parsed world bands lo (4.625393, 13.6251, -112.076) hi (6.950223,
     14.5615015, -110.1164) and lo (4.6727543, 12.730839, -155.13469) hi (6.9975834,
     13.66724, -153.1751). Under the raw law the bay-floor rest (feet 11.84, the standing
     capsule entirely BELOW the volume floor 13.625) stays SILENT: the volume floor tri's
     record plane is behind the clamped anchor → the one-sided code-99 miss [M Dist2Tri
     0x1dfca8:58-68] — the mechanism the drone.rs toucher comment already documents.

## What was tried and measured — the deep reach and the box

The two lever changes were implemented and run against the task's battery (release,
--test-threads=1, NF_ISO on the slim ISO):
- the standing segment's raw law widened from `flags == 0x645` to the game's bit 0x40
  (`raw_glist`, the field + the parse landed in the probes' tree), and
- the riding gate on the script-plates-only deep toucher removed (its stated rationale — the
  deep reach firing #614 "through the floor" — is the disproven premise above; the
  RIDE_TOUCHER skin gate already excludes every placed plate and every laser glist).

GREEN with both: nf-game lib 658/0/5; fix_truck_live 2/0; mayhew_handover_live 3/0;
npc_doors_live 6/0/1; fix_guard_misc_live 6/0; walkreal_0700000a_live 13/0/2 (the office-ring
qworm walks + the beam pins all green).

RED with both — the "floor plates fired ahead of the box and moved scripted sequences"
family, the same mechanism trigger-beams-3's c56bbe87 message records (17 tests then):
walkreal_07000002_live 1/2/1 (`walk_the_castle_all_objectives_to_the_exit` — STUCK at the
ch-0x17 plate, the captor-deaths pin; `..._real_health_agent` — the geisha-2 ch-0x4A save pin,
STUCK at west wing/south hall nav), walkreal_0700000c_live 2/1/1
(`walk_the_whole_mission_..._real_damage` — level_exit reached +0 ticks then the west-edge
wire #284 grab never happened: the exit sequence moved), walkreal_07000011_live (the binary
OOM-killed at 2.6 GiB RSS on the 3.9 GiB box — present on this base too, see below; per-test:
`..._real_damage` the 6000-tick cap at the south-band doorway, `..._real_damage_agent` Bond
died to drone crossfire 29 hits/100.0).

**Verdict: the evidence does NOT allow the unconditional landing yet.** The route pins are
tuned against the port's measured contact (the box whose spine starts at feet+0.985 and never
fires at-floor plates; the riding-gated deep toucher). The game's own law fires at-floor
plates earlier (the segment's spine bottom = feet+0.55, r 0.55 → a plate 0.1 proud of the
floor fires at horizontal ≤ 0.317). The route retune (which plate stamps move, per level) is
the named continuation — the probes + this report carry every number it needs. The two lever
changes are REVERTED on the pushed branch (master behaviour byte-exact); re-apply from this
report's numbers, retune the three levels' routes, then re-run the battery.

## What landed

- The two prints-only probes (nf-formats examples, no production code touched):
  `probe_trigger_extents.rs` (the disc side: placement/target/collision/parsed-band dumps for
  #607/#614/#621/#235/#274/#506 + the 0a snapshot object walk) and
  `probe_glist_flag_census.rs` (the 21-level flags census).
- This report + the WORKLOG entry.

## Verification (the pushed tree)

- The pushed tree is behaviour-byte-identical to master in every test binary (the diff = two
  examples + docs), so the suite baseline running on it at finish time is this tree's run. It
  did not reach its total line inside the sandbox window (the 1102-binary run was still
  mid-suite at the 18:17 UTC finish; its partial output showed the three disc_golden failures
  the slim ISO allows plus CRASH exit-9 lines on mp_shot_effects_live and
  walkreal_07000011_live — the 3.9 GiB no-swap box's OOM, the same kill the battery saw on
  walkreal_07000011_live — and one glass_guards_14_live failure). The suite total line for
  this tree is the first item for the next session: re-run
  NF_ISO=$NF_ISO nohup python3 tools/test/suite.py > suite.out 2>&1 & and read tail -1. The
  named battery on the LEVER tree (reverted here) is quoted above as the measurement.
- Per binary on the lever tree: nf-game 658/0/5, fix_truck 2/0, mayhew 3/0, npc_doors 6/0/1,
  fix_guard_misc 6/0, walkreal_0700000a 13/0/2.

## What's left

1. The bit-0x40 raw-law widening + the deep-reach unconditioning, WITH the route retune for
   07000002 (the ch-0x17 plate + the geisha pins), 0700000c (the level-exit/wire-#284
   sequence) and 07000011 (the south-band doorway + the agent-damage route) — the retune is
   "the tests are tuned to the port gap"; the decompile says the game fires earlier
   [M 0x1ebaa8:93-99].
2. The box-top retirement (pos+1.26) after #235 moves to the segment's centroid anchor and
   the 0c bay rest goes silent under the raw law — both proven mechanisms above, blocked only
   by the same route retune.
3. The 07000011 walkreal OOM (2.6 GiB RSS): reproduce on the base tree and, if base-green,
 treat as the lever tree's sequence shift inflating the sim; if base-red, it is a master
   issue independent of this task.

## Live checks for the main agent

1. `fieldprobe` on 07000011, Bond standing just south of the "Trigger Bond Moment" quad
   (placement (9.355, 49.78, 19.60), the wall at x ≈ 9.36 spanning z 18.89..20.31): read
   Bond's BLData translation when the Bond-moment slow-mo fires — expected: the fire is
   horizontal-only (the centroid-anchor law), i.e. it triggers at |x − 9.36| < 0.55 whatever
   the stance height. This would confirm the parallel-branch reading of #235.
2. `gdbwatch` the ch-0x1b (27) write on 0700000c during a bay-floor rest, then a real
   jump/climb INTO the #274 volume: expected silent at the rest, a stamp on the crossing tick
   (the record-plane one-sided entry). The port currently fires at the rest (the box) — the
   [L] divergence Toucher::player's comment documents.

## Usage

not visible to me — this sandbox exposes no token, credit or context-window figures for the
session (no such counters appeared in any tool output or environment surface).

## ntfy failed

All three fallbacks failed (2026-11-04 18:17–18:20 UTC); github.com pushes from the same
sandbox succeeded, so the block is specific to ntfy.sh egress:
1. `curl -sS -m 20 -H "Title: NightfirePort" -d "trigger-glist-extents: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds` (exit 28).
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'trigger-glist-extents: done', headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`.
3. `wget -q -O- --post-data "trigger-glist-extents: done" https://ntfy.sh/nightfire-yoda-8k3x --timeout=20`
   → no output; diagnostic without -q: `Resolving ntfy.sh (ntfy.sh)... 129.212.146.227, 2604:a880:800:14:0:3:cb8d:4000` /
   `Connecting to ntfy.sh (ntfy.sh)|129.212.146.227|:443... failed: Connection timed out.` /
   IPv6 `failed: Network is unreachable.`
The finish marker itself ran OK (`finish.sh` printed `OK done 20-trigger-glist-extents`; the
first run resolved the report path against the nf-queue worktree — repaired by re-running with
the absolute path, `OK already closed in done/, report attached`).
