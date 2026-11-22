## real-health-00agent-01 (2026-11-22, s/real-health-00agent-01)

Task: the 01 00 Agent real-damage contact walk
`walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_00agent`
is ignored on the s/hard-walks-d ledger: Bond leaves the ch-0xF pocket at 57.4 (the hit-box
shield + torso aim lane stand) but the stand kills only #17; #12 and the captain #3 outlast
it behind the contact, and the contact dies in the pocket at t50960 (mission fail). The
driver's order: FIRST check whether the contact's death is the port doing something the game
does not (guard target choice against the contact, his health, cover), then re-route. Branch
base master dbf17a37d + origin/s/hard-walks-d merged (its ledger is the newest on this test
and is not on master yet). Helpers stay d3-gated; real input only, no top-ups, never lower
damage or health, never change guard code. Un-ignore when green.

Progress (2026-11-22 18:40 UTC): the driver's FIRST question is answered — the contact's death
is NOT a port difference. Evidence (work/decomp on this branch, helpers read via the branch):
- Guard target choice: NDrone2_FindOpponent 0x1424e0:100-121 — in SP, team != 2 drones take the
  player object unconditionally (early return, no sight test) and NDrone2_SetOpponent 0x1422d0:19-20
  stores it with no team filter; no branch lets a team-1 guard target a team-2 ally. [L]
  ee_start_07000001.bin: all 14 team-1 guards hold +0x170 = Bond's obj (0x18f5790), the contact
  (skin 0x0500_004f) holds +0x170 = 0. Port: drone.rs:21302-21313 — faithful.
- Crossfire landing: NDrone2_DealWithObjHit 0x13fff0 case 5 — a class-5 (bullet) hit on a drone
  lands iff shooter team 1 and victim NOT team 1 (bVar3/bVar4 team fold; Bond's own rounds always
  land on teams 1/3+). A guard round aimed at Bond that crosses the contact lands on him.
  Drone_CollisionHandler 0x140968:84-95/209-220 delivers the obj's hitdata chain. Port:
  drone.rs:27417-27430 (drone_round_applies) — gate-for-gate match, no divergence.
- Contact health: no placement health field — NDrone2_DefaultInit 0x14b300:52 sets +0xAC = 10.0;
  difficulty scales damage only (NDrone2_HitDamage 0x145558:135-156), never spawn health.
  [L] ee_start_07000001.bin: contact team 2, health 10.0, dtype +0xC5 = 12 (team 2 per
  NDrone2_GetDroneTypeAttackTypeFriend 0x14cbd8:48-61), voice 9 — 2-3 guard rounds kill him.
- Cover: NDrone2_NotNearArmedDrone 0x177640 (sole caller AllyLeadHide 0x15ec08:30, radius 14.0)
  hides the contact while an ARMED TEAM-1 peer is within 14 u and seen/latched; exit arms timer C
  = now + 2·FRAME_RATE and re-paths. Port drone.rs:21727-21786 + LEAD_HIDE 10598-10626 — faithful.
Conclusion: the game's contact dies the same way in this crossfire (10 hp, guards always aim at
Bond, rounds are physical objects tested against every other-side body). The lever is the ROUTE.
Next: reproduce on this branch, then re-route so the pocket guards die before the contact's
plate arrival (the game's own hide keeps him east while armed peers are within 14 u — fight the
lane with the contact out of it).

Progress (2026-11-22 20:00 UTC): reproduced and measured the walk on the merged tree (4 real
runs, [L traces in /tmp run1-4]). The contact's death cause CORRECTED: he overtook Bond during
the ambush approach (parked at the stairwell head by t24900) and died to FOUR PLAYER rounds
(3.5 each, by_player true, t25015-25440) — the 624-tick #12 stand's full-bloom P2K spray
(0.66-0.70 u off-line at 2.5-3 u = the 0.2464 cap); the t50960 ledger tick was the restart
processing. The port-vs-game check: faithful (see the previous entry). Route levers tried,
all in the test file only (d3-gated, no shared-code change): the contact-line gate
(contact_blocks_line, the cap cone) at every d3 ambush fire site — run2: zero by_player
damage on the contact all run (Bond's friendly-fire kill CLOSED); the stairwell-head fight
stance — runs 3/4: #12 killed (two 3.5 part-4 rounds) but the ×2 trade + #17's approach fire
cost 36.4 of 57.4 and #3's close grind finished Bond at t27705; the parked contact then took
guard spray (by_drone 4/17). The wall is measured: #3 (24 hp) is unkillable from any
lane-adjacent stance without crossing the parked contact. The next lever (in the ignore
message): the ~400-700-tick pre-arrival window fight, then draw #3 out of the pocket. The
walk stays IGNORED with the measured ledger; the gate + head fight stay on the branch.

Progress (2026-11-22 19:30 UTC, final): the ledger's next lever (the pre-arrival window) was
TRIED and is MEASURED DEAD ([L run5]): without the -19 retreat, #12 stands in the x2 band at
2.8 u and Bond died EARLIER (t25396 vs t27705), no window kill landed, the parked contact took
guard spray again (by_drone 12). The state was restored to the gate + head fight and the
reproduce now matches run4 exactly (Bond dies t27705 [L run7]). Structural finding for the
next worker ([M] the walk's own placement dump + [L] runs 1-7): the pop slab
(x -26.13..-24.73, z 13.25..16.84) covers the corridor's full width and is Bond-touch only,
so the contact (whose lead route waits at the head for Bond, then hides there once armed
peers are live) can NEVER descend before the pop — and while he parks at (-30.24,13.92),
every pocket guard is unkillable under the bloom gate (his center 0.03-0.32 u off every lane
line; the guards' own spray crosses him from any lane-adjacent stance). The pocket must be
cleared for his hide to release (the descent stalls otherwise). Next levers, in order of
promise: (1) a FAST-KILL budget — the head fight killed #12 in ~30 ticks of taps; if #3 can
be baited north (his advance measured to (-30.7,14.7) in run2, where the mouth line clears
the parked contact at 0.9) and killed within the contact's 10-hp spray budget, the pocket
clears before the spray accumulates; (2) a walker-side guard-spray shield for the contact
(the mirror of the player-side gate) so the guards' 1.5-hit spray stops counting against his
10 hp while the pocket clears. The walk stays IGNORED with the measured ledger.
