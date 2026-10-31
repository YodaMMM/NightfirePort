# WORKLOG

### level-0c-secret-door-reach (start, 07:35 UTC, 2026-10-31)

Task: `level-0c-remaining-teleports-to-walks` (merged) found the secret door #544's plates
#558 (E-arm) / #559 (body touch) sit in a corridor whose touch volume starts at x 52.24,
while the body-touch capsule reaches only x <= 51.84 from the yard side — so the door cannot
be opened from the yard, and `walkreal_0700000c_live` still places Bond at the plates. Find
the game's way into that corridor [D the corridor's north end, the mezzanine annex vent
#180/#182/#184 (ch 0x54), any door/hatch that opens onto it; M the classes involved], walk it
on real input, open the door through the plates, and delete the placement. Fix any port gap
with evidence. Run walkreal_0700000c_live and interactables_0700000c_live; quote the totals.

### level-0c-secret-door-reach (progress, 09:07 UTC, 2026-10-31)

The game's way into the secret-door corridor is FOUND and evidenced, the walk legs are not yet
written. Evidence (all [D] via the port's parsers, this session's committed probes
`walkreal_probe0c_secretreach_live.rs` (the flood-fill + band maps), `..._voidspan_live.rs`,
`..._upperlane_live.rs`, `..._bridge_live.rs`):
1. **The corridor's north end is OPEN**: the 0.25-grid flood over x 8..62, z −192..−115 (0.55
   step law, drops ≤3.5 one-way, knee checks) floods the yard + the warehouse ground + the
   corridor as ONE component (comp 2: 15433 cells, x 40.8..62, z −186..−139.5, y 0.38..3.38) —
   the corridor floor 0.90 continues north past the vent (z ≤ −161.5) and merges with the
   warehouse ground (0.69/0.89) at the corridor's north end; at z −165 the corridor (x
   52.5..55.5, 0.90) is step-continuous east to the warehouse ground (x 56.0, 0.69).
2. **The mezzanine annex vent (#180/#182/#184, ch 0x54) hangs in the corridor at z −165..−163.5
   under the 5.44 blocking poly**; the mezzanine deck (5.44..6.03, x 52..56, z −176..−152) is a
   separate flood component NOT reachable from the ground — the vent is an ALTERNATE entrance
   only if the mezzanine is reached from the warehouse upper floor (itself an island) — the
   north end is the game's ground route.
3. **The display ground (6.27) is sealed east** (the x ~15 wall stands y 6.4..10.5+ [D rays]) —
   the game's exit is HIGH ("the ways in and out of this building are on higher floors" [D
   USATxt]): the ST-1 walkway runs south to z ~−155.5 and steps 14.6 → 14.3 → 13.3 onto the east
   deck (x 18..21, z −162..−156.5, the "13.3 deck"), the 9.2 patch (x 22..23) and the east ramp
   (6.9 → 3.87, x 18..29, z −155..−143) hang below; the ramp ends at x ~29.5 over a FLOORLESS
   pit (x 30..32, no surface 8→0) crossable by the engine's own drop-jump carry (4.42 flat
   measured, 5.2 unit law [M player.rs]) onto the north-east strip (0.06, x 33..40.5), which
   joins the warehouse ground (0.38, x 41+) knee-clear at y 0.9/1.4 (one post x 41.06 z −147).
4. The lane south (12.3, the walkreal's own walked territory z −115..−128) continues to z
   −158.5 [D upperlane scans]; the walkway/drop legs (14.6→13.3→9.2/6.9→ramp→pit jump→strip)
   are the NEW legs — drops of 4.1..6.4 m onto the ramp sections (the game's own 4.2 m band
   walk-off [L 2026-10-30 fresh boot] bounds survivable drops; the 6.4 m variant is [S]
   unverified) — the exact leg tuning is the next session's work.

Remaining: write the walk legs (the lane south extension, the walkway/deck, the drop chain,
the pit jump, the strip→warehouse→corridor walk), delete the `[plate2 + 0.6]` write, keep legs
5+ untouched (the plates chain + the door + the yard are already real), run
walkreal_0700000c_live + interactables_0700000c_live + the mission sweep, quote the totals.

### level-0c-secret-door-reach-2 (start, 09:00 UTC, 2026-10-31)

Task: `level-0c-secret-door-reach` (blocked) found the corridor's north end [D]: the corridor
floor (0.90) runs north past the vent and joins the warehouse ground (0.69/0.89) — yard,
warehouse ground and corridor flood as ONE component — so the plates #558/#559 are reachable
on foot from the north end. This session: base the branch on that work (merged origin/master),
write the walk legs on real input (yard -> warehouse ground -> the corridor north end -> south
to the plates), open the secret door #544 through the plates (body touch #559 + E #558 -> AND
#560 -> ch 8), walk out through it, and delete the `[plate2 + 0.6]` placement in
`walkreal_0700000c_live`. Then walkreal_0700000c_live + interactables_0700000c_live totals.
