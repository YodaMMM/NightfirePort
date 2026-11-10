
## level-05-basin-floor — branch cut

Branch cut from origin/master 4c2203cc6. Task: The Exchange's (07000005) valley basin has its floor
east of the lip. Measured on s/level-05-valley-hurt: past the lip at (132.0,14.17,125.37) every point
resolves find_cel = room 14, and room 14's collision build covers the east basin but has NO floor quads
there, so Bond cannot walk down into the valley the game lets him enter; the west basin floor is room 15
and works; the basin is hurt #3's band (x 80.78..168.38, z 96.66..144.68, y ~5.1). s/level-05-valley-hurt
has only a branch cut so far (no report on it yet). Find why the port's room-14 build drops that floor:
compare 07000005.bin's collision data for the room [D] against the port's parse, and the game's FindCel
relink [M build_FindCel 0x1c8cb0] against the port's ScopeGraph. Fix the build with a test that fails on
the old code (a down ray in the basin hits the floor); then un-ignore
walkreal_07000005_live::the_valley_plane_hurts_bond_walked if the walk reaches the plane. Run
walkreal_07000005_live, interactables_07000005_live, and every walkreal_*_live binary (shared collision
build changes).

## level-05-basin-floor — the basin floor was the hurt plane; the 0xE4 fold

The valley basin east of the lip has no room floor on disc: the outdoor room's collision block
(07000005.bin main map block 262, Room_Road_A1 plc 86, 3121 tris) ends at z 125.98, and its
own up-facing floor quads (y 3.5-7) stop at z ~121.5 — the block walk + scatter probes
(`probe_05_collblocks`/`probe_05_floorscatter`) leave the band x 80.78..168.38, z 96.66..144.68
with exactly one floor-level surface: hurt placement 331's own 2-tri plane at y 5.107 (kind 0xE4
"Hurt_B"; the west twin plc 332 at y 4.378). That IS the floor the game walks: `Hurt_Create`
0x17e870 builds a class-0x1C object, `Control_SetGList` :44 attaches the glist (the collision
block) and `build_LinkToRoom` :46 links it to a room, so the player capsule meets the plane as an
object hit and `Player_DealWithObjHit` 0x194458 applies the damage — solid ground and the damage
are one mechanism. The port's `instances(true)` never built kind 0xE4, so the basin had no floor.
Fix: fold the UNGATED (gate param 2 == 0) 0xE4 placements' collision into the instance list, the
0x19/0xd5 precedent (their runs ride the always bucket; the game links each hurt object to the one
room `build_FindCel` 0x1c8cb0 picks — on 05 the east plane resolves to cel 14 with a single
containing candidate). `walkreal_07000005_live::the_basin_floor_exists_past_the_lip` proves red on
master's code (scoped + flat down rays None) and green on the fix (every basin point hits y 5.107,
normal up, through cel 14). The walked valley test is NOT un-ignored: the floor exists but the raw
run+jump drive still holds Bond at the lip (132.0,14.2,125.4) with every direction ray clear — the
lip-edge walk behaviour is a separate bug, ledger untouched. Report:
docs/reports/level-05-basin-floor.md.

## level-05-basin-floor-2 — branch cut

Branch cut from origin/s/level-05-basin-floor + origin/master (merge fe2e52db9). Task: round 1's
ungated 0xE4 solid fold went red on its own level —
interactables_07000005_live::the_valley_walkoff_runs_the_plunge_death_nis_chain and
walkreal_07000005_live::the_door_band_exit_chains_to_07000006_walked_real_damage. Find what the
now-solid hurt planes change on those two paths (does Bond land on a plane before the PlungeTrigger
at y ~13.17 fires, or does the plunge chain read a fall the floor now stops?), fix it the game's way
(read the plunge trigger and death NIS code first), port the gated planes' solidness when their
channel rises (round 1's open item), and act on the live PCSX2 fact (2026-11-10): Bond placed past
the east lip at (132.0, 14.4, 126.6) settles at y 14.15 and plays the plunge death — the game does
NOT let Bond walk down into the basin, so the_valley_plane_hurts_bond_walked's route is impossible:
delete that test citing this, pin the lip as a plunge death if not already covered. Keep the solid
hurt planes only if game-correct AND the plunge test is green; otherwise drop the static fold.

## level-05-basin-floor-2 — the solid fold kept; the plunge re-pinned on the lip; the door-band shift located

The two reds of round 1 answered on the game's own evidence. The plunge chain: the old
basin-interior walk had ridden the port's pre-fold fall THROUGH #287's sloped funnel (the
disc's out-190 probe: #285 is the live fact's y 13.17 flat plane over the east basin; #287's
funnel runs (−20.28,10.18,120.77)→(59.71,−2.13,96.66..144.68)); the solid fold (kept,
game-correct [M Hurt_Create 0x17e870:51-57, Collide_Pick 0x1eb320:140]) lands that fall on
hurt 0's plane (y −0.70) 0.3-0.6 above the funnel and the chain stays dark — the game never
walks there anyway (the live fact: the lip settle at 14.15 IS the kill), so the test now pins
the lip settle-kill → SP #284's 0x06000952 → ch 0x62 (6/6 green). the_valley_plane_hurts_bond_walked
DELETED citing the live fact. The door-band real_damage: fold-on/off control runs (an env
switch, removed) prove the route tick-identical to 9557 with `other 0.0` hurt damage — the
fold shifts the return-fire rolls (the world's bullet surface changed) and the stair-top pair
#12/#13 at d 1.8-2.2 kill the scripted legs' stand (35 vs 31 hits); both tactic levers are
mute in scripted legs; the flag-gated lever and the next-session pre-emptive pattern are in
the report. Gated-plane solidness (Hurt_Update 0x17e5d0:26-37 + Collide_Pick 0x1eb320:140,
the every-tick 0x20 law) left pinned for the next session. Report:
docs/reports/level-05-basin-floor-2.md.

## level-05-basin-floor-3 — branch cut

11:36 UTC 2026-11-10. Branch s/level-05-basin-floor-3 cut from
s/level-05-basin-floor-2 (200251338) and merged origin/master (4bdaf3f5) clean.
Goal: land the solid hurt-plane fold with every level green — fix the
door-band roll shift via the report's pre-emptive lever (REAL_DAMAGE_BACKOFF-gated),
port the gated planes' every-tick 0x20 solidness (Hurt_Update 0x17e5d0:26-37,
Collide_Pick 0x1eb320:140), and fix any other fold-caused suite red.
Full suite started in the background in the first minutes.

## guard-capsule-05-down-bump-harness — branch cut

13:01 UTC 2026-11-10. Branch s/guard-capsule-05-down-bump-harness cut from
origin/master (4bdaf3f5; WORKLOG carried from the s/level-05 lineage — master's
docs/ has no WORKLOG yet). Goal: the knocked-down guard bump test runs on
07000005 — either walk to a 05 guard on Bond's own floor on real input, stun him,
and creep in until the body-contact latch fires inside 0.85 (the 07000006 sister's
proven shape, contact at dxz 0.771), or prove no 05 guard is same-floor reachable
without a teleport and delete the 05 harness. Run guard_capsule_radii_live and
walkreal_07000005_live.

## guard-capsule-05-down-bump-harness — the 05 harness runs; the 0.3 down ball confirmed on 05

13:26 UTC 2026-11-10. The Level::parse census of 07000005's 16 kind-0x0F placements
[D probe05guards] answered the first pass's two blockers: guards #5 (61.34, 27.13,
99.50) and #6 (50.85, 28.27, 93.63) stand ON the courtyard west band — the same
floor as the proven courtyard-west nav route's end (60.1, 28.4, 96.4) — so the
harness walks THAT route on real input instead of the gatehouse chase that ran the
guards downhill. The held Stunner fires on the walked Bond there (accum 1 clip 99
at t0 — the first pass's "beam never fires on the walked Bond" blocker was the
ledge-crossing aim, not the walk), the guard falls TASER at hold tick 20 →
STUNNED+down, and the creep's latch (the guard's own +0x26, DealWithObjHit case 3
0x1405c8) first fires at dxz 0.772 — the game's 0.55+0.3 = 0.85 ball, the 06
sister's 0.771 reproduced on 05 [L two runs, deterministic]. Deletion not needed.
guard_capsule_radii_live 2/0; walkreal_07000005_live 16/0 +1 ignored (master's
parked notch test); guard_capsule_live 2/0; interact_07000005_live 11/0.
