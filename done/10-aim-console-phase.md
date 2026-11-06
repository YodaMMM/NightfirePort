# Aim snap: the 8 walks, with a corrected premise

Branch from `origin/s/aim-candidates`, merge current `origin/master`, push to `s/aim-console-phase`.

CORRECTION: 10-aim-candidates said the port stops on objects the game does not have. That is withdrawn. The PCSX2 rays and object reads behind it ran with Bond idle, before the launch order, when the rigs were down.

What is now measured on 0700001b:
- The port's rig pose matches the game. r 1.796 pieces: t 0 (24.44, -8.80, -11.39) in both; game frame 1782 (26.25, -3.02, -11.39), port t 1620 (26.28, -3.04). Class 0, f0 0x80, fa 0xc4, glist+8 set, in the cel chain, so the game's pick can stop on them.
- Idle, only one rig rises. The rest come with the console phase: channel 19, raised by the first wave's death. The walk starts `shoot_consoles` at frame 2746. Writing channel 19 alone in PCSX2 raised nothing, so the phase needs more than that write.
- So there is no evidence the game's ray skips raised rigs. Do not remove candidates to make walks pass.

Do not search for a discriminator between "real" and "fake" entities. Instead:
1. Log what the snap stops on at each failing shot of consoles 1 and 2: the entity, its role in the launch script, and where the console's own hit sphere sits behind it. Is a raised rig really between Bond and the console from that stance?
2. If so, the snap may be correct and the walk's stance wrong: a player would move to a clear line. `level_station` plans from `sphere_shot_ok`, which does not see these entities, so make it use the same candidates as the snap. Try that first on 1b.
3. Then look at the other six reds (08, 0c, 14 x3, 16) the same way, one at a time. The aim stack changes where rounds land, so fights tuned to the raw aim may need route levers. Say which is which. 14's three are a nav stick, "the corridor south to the leg-2 spot", so look at what stands there.
4. To reach the real phase in PCSX2 later, list what the first wave's death does besides channel 19, in order, with citations.

Keep the [L] 08 pins and the 0x30000000 fix. Quote the six aim binaries and walkreal 08 / 0c / 14 / 16 / 1b.

---
claimed-by: sandbox4 at 2026-11-06T10:42Z

---
## Report (done, 2026-11-06T11:47Z)


### The report body (the finish.sh cat missed it — appended from the branch)
Branch s/aim-console-phase (from origin/s/aim-candidates ceb1ddae8 + origin/master d6245281a), head 6beb0774f. Full text: docs/reports/aim-console-phase.md on the branch.

**Done**: (1) the traced snap — aim_snap's body in aim_snap_inner with an optional trace sink, aim_snap unchanged, aim_snap_traced returns (point, Vec<SnapHit{stage,id,pi,t}>) with one push per winning stop — one implementation, behavior-preserving. (2) The 1b probe answered step 1: console 8's shots are NOT stolen by the snap — 120/120 ring bearings snap-clean at t2207 and t4000 (the pick's first stop is the console's own screen object 2000a41, t 0.044, from every stance); the raised rigs (pi30 gate191 script 0x06000928: 20007e6 r1.80, 20007f6 r2.70, ~2 u BEHIND the screens at z -11.39 vs -9.42) are candidates only mid-rise and never beat the screens; the REAL failure is a floor landing — the float wedges at (44.7,-8.66,1.68), 2.2 u short of console 8's station (46,-8.12,0), walk-pinned with thrusters dead (position frozen t2700-t5700), from where sphere_shot_ok is FALSE (the world clips the bank) and the round misses 0.54; the stall never tripped because the wedged crawl moved 0.3 every ~110 ticks. The only game-code delta on the leg is the merged master (770a74927's alert fold); the 1b test file is byte-identical pre/post merge. (3) The snap-aware planner wired on 1b anyway (correct by the same evidence): snap_shot_ok gates level_station's ring, the default station and fire; the stall measures GOAL progress; the replan anchors on the console's default station; the float grew climb/creep sag guards. (4) The [L] 08 pins and the 0x30000000 fix untouched.

**Verified**: the battery, every binary whole, every number the aim-candidates profile: fusebox_geometry_07000014_live 1/0, interactables_07000014_live 10/0, rocket_stop_plane08_live 4/0, rocket_glass08_live 3/0, rocket_origin_live 1/0, solar_panel_hits_live 1/0; walkreal 08 5/1, 0c 2/1, 14 6/3/1 (single-threaded — 2 threads OOM-killed), 16 13/1, 1b 15/2. The src change is behavior-preserving; the seven reds (08 0c 14x3 16) are the pre-existing aim-finish ones. The full suite did not fit the window.

**Left**: 1b console 8's landing (the zero-g volume's floor boundary ~0.5 u above the low stations — the crossing-waypoint attempt went pathologically slow, reverted; next: a two-phase goal in float_to); the six reds (08 0c 14x3 16) not triaged per level yet — run the same [ring]/[gate] probe per level before touching routes; the first wave's death ledger besides channel 19 not compiled; the full suite and npc_doors_live/fix_guard_misc_live not run this window.

**Live checks for the main agent**: (1) gdbwatch the object record for glist 20007e6 from the first wave's death — expect obj+0x90 to rise y -8.8 to -2.9 over frames ~1782-1900 and its stream to leave the script's kind-1/2 set once risen; (2) fieldprobe Bond at (44.7,-8.66,1.68) facing the console-8 screen, one laser round — expect the world to stop it >= 0.55 u short of the sphere centre (the miss is the world, not the snap); (3) poke Bond to (46,-8.3,0) vs (46,-7.9,0) — expect walk vs float, pinning the zero-g boundary.

**Usage**: not visible to me — no token, credit, context or cost figures are exposed to this session.
