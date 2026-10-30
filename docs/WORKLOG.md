# WORKLOG

## level-1b-scripted-carry (branch s/level-1b-scripted-carry)

Start: 2026-10-30 12:31 UTC, branch from origin/master 9deb3338.

Task: walkreal_0700001b_live::walk_arrival_to_drakes_defeat is ignored: the west wall plc #128
OBJECT_Boundary_1 (x 4.433) is body-solid [M], so the game's crossing must be scripted. The
candidate is the SP#211 chain (0x060007fd -> 0x060008f0): it pulses ch 75 (the float entry) and
its tail pulses ch 74 (0x600096e, t392-399 [D]). Decode those scripts fully [D the streams;
M the script player/event handlers 0x1d94c0, 0x1d8548]: find any player-position write, attach
or teleport event, and which entity the player follows. Port what is missing, then make the walk
cross on real input (the script doing the carry) and un-ignore it. Every test in the file green.

Outcome 13:35 UTC: the crossing is NOT a scripted carry — the premise is refuted on the disc and
the shaft box is SEALED for the player body. [D] The whole 0700001b archive decodes through the
5,263-stream-validated parser with exactly ONE player-touching event row (NIS 0x06000a20 s4 ev05
Drone_EnableAll) — no event 4 (Player_Disable), no event 6 (the only player-matrix writer:
Script_EventHandler 0x1d94c0 case 6 anchors SCRIPTINFO+0x40 at the player/entity matrix; Script_Free
0x1da880 re-enables the player AT that matrix under flag 0x100 — never armed on 1b), so the script
system cannot move Bond. The SP#211 chain decodes clean [D]: 0x060007fd = the station-exterior
fly-by (one sky entity 0x02000823, 240 keys, 4780 ticks, (1051,174,527)→(−187,158,52), an event-3
hold pair, the engine sound); 0x060008f0 = 8 sky entities scattering + SUBs 0x06000971/0x0600096e
(type-0xe1 destroy props + ev0x0d [75,1/0] t384-391, [74,1/0] t392-399) + the fight gate ev0x0d
[10,1] at t400. ch 74's only reader is NDrone2_HandleTalking 0x13f688 (radio chatter) [D sweep].
[M re-verified] Player_Collision 0x194df0's default arm (substate 9) writes the shape word
+0x7d4 = 0x800 and never the filter word +0x7d8 (zipline-only 0x1c0 :382) — the boundary's 0x40
material can never be filtered for the body; the wall is body-solid.
[D geometry, segment probes on the ported collision]: the boundary box is closed on every face —
the west wall x 4.433 (y −16.56..+16.74, z ±20.4), the top slab y 16.74 (normal down, x 4.5..24,
no chimney at the wall), the pit floor y −16.56, and END-WALLS at z ±20.40 spanning the shaft
(x 4.4..28, y −16.5..+16.5) whose ONLY openings are deck-level windows 0.8..1.0 tall (x 7.5/8.0:
y −1.8..−1.0; x 10.0: −2.2..−1.2) — narrower than the float body (~1.58 tall [M the default arm's
capsule A = pos+up·0.55, B = pos+up·(+0x7CC−cc), r 0.55; cc 1.0347 [L ee_start_05]]). North also
carries a ledge beam (z +20.40, y ≈ 2..3) and south a corner rail (z −8.13) plus a second rail
(z −14.25, y −1.9..−0.7) — the walk pilots thread the first two and wedge at the end-wall inside
the window band (min z −19.85 = −20.40 + 0.55).
Landed: walkreal_0700001b_live's walk_arrival_to_drakes_defeat stays #[ignore]d with the full
sealed-box evidence + three live checks (the float substate's HITTEST shape; the fresh-boot
crossing watch; ch 74's reader); NEW green sister the_end_walls_seal_the_shaft_box pins the
end-walls + the window bands + the body-height impossibility; the sweep/probe tools landed as
examples (sweep1b_events, geo1b). The 1b file: 9 passed / 0 failed / 1 ignored.
