# capsule-cull-and-los-masks-17 — report

Task: continue `origin/s/capsule-cull-and-los-masks-16` (merge `origin/master` first), port the
feet-band from the decompile, and make walkreal 01 and 14 green with 02/03/08/0a and the rest
unchanged. Branch `s/capsule-cull-and-los-masks-17` = origin/s/capsule-cull-and-los-masks-16 +
merge origin/master (fd24fbf4), clean; first checkpoint pushed (04fa4138).

## The headline: 01 and 14 are green on the merged tree — through master's test-route fixes

The merged tree's walk spine is byte-identical to -16 (`git diff origin/s/capsule-cull-and-los-masks-16
HEAD -- crates/nf-game/src/world.rs crates/nf-game/src/player.rs` is EMPTY — the master merges since
a19be699 touched interactables/levels/tests, not the walk model). What moved -16's 01/14 reds is the
TEST side: master's level-14 work added the square-up waypoint to `walk_the_infiltration_to_the_delta_exit`
(`w.walk_to(Wp::new(26.0, 1.0, -22.0)...)`, walkreal_07000014_live.rs :975-977) and the 01 route fixes
landed earlier; measured on this tree, every walkreal binary is green (the matrix below). The
substitute-tail gap that -16 diagnosed therefore does not bite the merged tree's pinned routes —
and the faithful walk-model switch this task aimed at (the ported feet band replacing the
substitute's vertical) was NOT landed, for the reason below.

## Why the walk-model switch was not landed this window

The feet-band decompile pass (all [M], this window) settled the detection side and advanced the
vertical question:

- **Player_FeetOnPoint 0x194188** whole: the ray from obj+0x30 to `HITTEST+0x30 − up·(cc+0.6)`
  [:89-98 — cc = PD+0xcc [M Player_Collision 0x194df0:86, the B arm 0.55−cc :330-337]]; the band
  `|from−hit| − 0.1 ≤ cc` [:106] with HITDATA+0xc = the ABSOLUTE distance [M Intersect_RayGeom
  0x10a8c8:312-325]; `*BL+0x110` = 1.0 then the band hit's Q14 plane-normal y [:85/:106-109 with
  0x10a8c8:326-337]; the shadow copy `PD[0..0x60] = hit[0..0x60]` under the surfaced + non-water
  gate [:187-192]; the inherit/riding block (Control_InheritVelocity, PD->60 |= 0x80, the
  synthetic up-normal hit) [:124-155]; grounded flag 0x8 = (HITTEST+0x9c != 0 || band hit)
  [:176-185, the +0x9c read pinned at disasm 0x1943a8 `lhu v0,156(s6)`].
- **Player_CollisionHandler 0x1949a8**: pos += HITTEST+0x10 (the DeltaP accumulator) [:136-140];
  grounded (flag 0x8) → `BL+0x14 = 0; Vec_Zero(BL+0x50)` [:168-175] — the port models both
  (player.rs's `pos += push`, the grounded velocity zero, the floor_ny band gate).
- **Collide_Update 0x1ea398** (the engine loop, NEW this window): zeroes `HITTEST+0x9c` per
  object per tick [M], runs `Collide_Pick`, then the per-candidate `Collide_Intersect` with the
  +0x9a mask — so the game's grounded = ANY capsule hit this tick (not just the 0.7 walkable
  faces the port's `walkable` feeds). The port's `grounded = walkable || band` is a NARROWER
  stand-in — the any-hit widening is a walk-model change (it re-gates gravity on every level)
  that cannot be verified against the full suite in this window, so it stays out (the
  merge-lesson call: no unverifiable walk-model change).
- **The walk case's dropped store**: HITTEST+0x98 = 0x1C0 [M disasm 0x1951b0 `sh a0,2008(s2)`,
  a0 = 448; the decompiler keeps it only in case 0xe :382]. With 0x1C0, all four hit-cancelling
  arms in Collide_Intersect 0x1ebaa8:300-315 are dead for the walk (mask & 0x800/1/2/8 all 0) —
  the walk's hits always register.
- **The push tail's v operand stays the open live check**: the decompile reads
  `v = q − HITTEST+0x50` (:421-431) — the lifetime accumulator [L ee_start_* boots (0,0,0); the
  ee_snapshot "drift" ≈ the owner's distance from the origin, consistent with acc ≈ (0,0,0)
  always → the tail dead on every level whose geometry sits > 0.275 from the origin]. But the
  disasm window 0x10a53c-44 STORES the copy's +0x50 and +0x00 (`sqc2 vf1,80(s1)`, `sqc2 vf2,0(s1)`)
  right after the Dist2Tri gates — the copy's +0x50 is re-written per triangle (the 6-b
  "query − Q" scratch reading), which the decompile's accumulator reading does not model. Which
  of the two the metal executes decides whether the game's walk push is dead-by-origin or
  dead-by-scratch — exactly the -6-c live check #2, still needing the main agent's debugger.

With 01/14 already green on the pinned routes and the tail-operand question open, landing the
walk-model switch (the CLIP/GHOST default flip) would have invalidated the full-suite baseline
this window ran (the sandbox kills background runners between tool calls — re-measured again;
the suite died at 748/856 binaries and its log with it) — the switch is reported here with the
evidence instead of shipped half-verified.

## The measurements

**The walkreal battery (every walkreal_* binary; each total quoted):**

| binary | result | where |
|---|---|---|
| walkreal_07000001_live | **5/0** (70.07 s) | foreground this window |
| walkreal_07000002_live | 1/0 | suite |
| walkreal_07000003_live | 4/0 | suite |
| walkreal_07000004_live | 7/0 | suite |
| walkreal_07000005_live | 10/0 | suite |
| walkreal_07000006_live | **3/0** (57.17 s) | foreground |
| walkreal_07000007_live | 2/0 | suite |
| walkreal_07000008_live | 3/0 | suite |
| walkreal_07000009_live | **4/0** (179.42 s) | foreground |
| walkreal_0700000a_live | 7/0 + 2i | suite |
| walkreal_0700000b_live | **3/0** (60.98 s) | foreground |
| walkreal_0700000c_live | 2/0 | suite |
| walkreal_0700000c_eastroute_live | 1/0 | suite |
| walkreal_0700000d_live | **1/0** (36.87 s) | foreground |
| walkreal_07000011_live | 1/0 | suite |
| walkreal_07000011_route_live | 1/0 | suite |
| walkreal_07000012_live | 1/0 | suite |
| walkreal_07000013_live | 1/0 | suite |
| walkreal_07000014_live | **6/0** (split: 5/0 in the full run + the 6th 1/0 isolated, 43.12 s — the sandbox OOM-kills the 6th test in-process, exit 137, before it runs) | foreground |
| walkreal_07000015_live | 1/0 | suite |
| walkreal_07000016_live | 8/0 + 1i | suite |
| walkreal_0700001b_live | **9/0 + 1i** (20.69 s) | foreground |
| walkreal_07000041_live | 5/0 | suite |
| walkreal_0700004a_live | 1/0 | suite |
| walkreal_0700004a_satchel_live | 2/0 | suite |
| walkreal_celprobe_live | 2/0 | suite |
| walkreal_dump_live | **1/0** (6.92 s) | foreground |
| walkreal_probe* (the probe family, ~40 bins) | all green (e.g. walkreal_probe01 3/0, walkreal_probe014_param26_live 1/0) | suite |

**The suite** (tools/test/suite.py, started 01:15 UTC background per the task; the sandbox
killed it between tool calls at 748/856 binaries and deleted its log — the per-binary results
were parsed from the log BEFORE the kill): **2926 passed, 7 failed, 209 ignored over 708
binaries with results**; the 7 failures are ALL accounted for and none in the walk family:
- `disc_golden` 11/3 — the 3 sanctioned slim-ISO failures;
- `identity_disc` 0/1 — asserts the full disc's 1,191,072 sectors (docs/build-identity.md);
  the sandbox's slim onfoot ISO has 518,432 — a slim-ISO limitation, same family;
- `fix_0c_pylon_live` 0/2 and `probe_pylon_render` 0/1 — panic at the wgpu `request_adapter`
  expect (fix_0c_pylon_live.rs :19): **no GPU in this sandbox** (merge lesson 6 — the main
  agent renders these);
- foreground completion for the 8 walkreal binaries the suite never reached (the table above)
  adds 31/0/1i over 8 binaries.

**Combined: 2957 passed, 7 failed, 210 ignored across 716 binaries with results** + 4 binaries
the sandbox OOM-kills in-process on this 4 GB box regardless of load (`mp_remote_fx_live`,
`frame_cost_live`, `net_desync_report_live`, `mp_death_hands_online` — exit -9/137 inside their
big-archive tests, reproduced standalone; not test failures). nf-game lib ran inside the suite:
588/0/5i.

## What's done

1. The feet-band decompile pass, cited above (FeetOnPoint whole; the handler; Collide_Update's
   per-tick +0x9c zero; the walk case's dropped 0x1C0 store; the Dist2Tri arg map showing
   s1+0x50 feeding Dist2Tri as an operand — the copy's scratch, disasm 0x10a4f0-0x10a4fc).
2. The full walkreal battery green on the merged tree, every total quoted.
3. The suite run + the forensic total after the sandbox kill.
4. The triage of every non-green (3 sanctioned disc_golden; 1 slim-ISO; 3 no-GPU; 4 OOM).

## What's left (the next message)

1. **The walk-model switch decision** with a full suite: widen `grounded` to the game's
   any-capsule-hit rule [M Collide_Update 0x1ea398 + FeetOnPoint 0x1943a8] and/or land the
   CLIP/GHOST walk model as default, then run the FULL suite on the changed tree (the fast
   runner needs a sandbox that does not kill background jobs between tool calls — or foreground
   batches like -6-c's).
2. **The tail operand decode** (the live check below) — it decides whether the port's
   substitute tail is (a) exactly the game's dead arm (keep) or (b) live where the game's is
   scratch-relative (then the faithful port is the CLIP model's dead tail).
3. Re-verify the 02 geisha-2 save and 03's route stay green through the next merge (the
   sensitive legs).

## Live checks for the main agent

1. **The push tail's v operand** (gdbwatch.py, the -6-c #2 still open): break at 0x10a564
   (`lwc1 f0,128(sp)` — the dot result) during the 07000014 stair climb and a flat walk; read
   the VU source registers feeding the 0x10a548 VSUBC and the memory at BL+0x790 (the copy's
   +0x50 source). Expected under the decompile reading: the operand = the LIFETIME accumulator
   (BL+0x790, ≈ (0,0,0) far from the origin); under the 6-b reading: the CURRENT triangle's
   query point. One run settles which.
2. **The grounded any-hit rule** (gdbwatch.py): watch HITTEST+0x9c (BL+0x7dc) across a walk
   that touches only steep faces (a wall graze on the 07000005 ledge) — expect != 0 on the
   graze ticks (the any-hit OR at Collide_Intersect :390) while the port's `walkable` stays
   false; if BL+0x110 (the feet-ray out) holds 1.0 there, the game's gravity gate stays closed
   on a wall graze the port would open (the port's floor_ny = 1.0 default matches, so the
   observable may be identical — the check is whether the port's narrower rule ever diverges
   in a pinned test).
3. **The feet band on the 14 landing** (fieldprobe.py): the pit-band jump west onto the -0.91
   landing — record Bond's rest root y across a few boots; the port rests at floor + cc
   (−0.91 + 1.0347 = +0.1247, the assert's `< 0.0` holds); confirm the game's live value to
   pin the band's exact settle.

## Usage

not visible to me (this CLI session exposes no token/credit/context-window readout).
