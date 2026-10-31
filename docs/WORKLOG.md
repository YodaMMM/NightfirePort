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
