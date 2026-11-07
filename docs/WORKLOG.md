# WORKLOG

## level-0c-vent-crawl

Branch `s/level-0c-vent-crawl` from `origin/master` (10956badb). Task: on 0700000c near
(5.33, 12.49, -110.94), behind the crates, Bond cannot crawl through the vent. Find the crouch
and duck rules, the grille, and the collision at the opening, and walk it end to end with real
input. Reproduce on the real level first through real input, not a teleport; compare with the
decompile, fix, and test red on the old code. No draw changes. Report:
docs/reports/level-0c-vent-crawl.md.

Entry started — the reproduction follows.

Progress (WIP checkpoint 1): the spot identified. (5.33, 12.49, −110.94) = the CROUCHED root on
the display-case vent-shaft floor 11.84 (11.84 + 0.646 = 12.486) at the shaft's north edge
(z −110.94 = the north wall plane −110.39 − capsule r 0.55), x = the port's known bay-floor rest
spot (5.33, −110.95 [fix_0c_hatch_live.rs:175-178's Δz 0.43 live check]). The player fell into
the shaft (the game's own vent route: walkway 14.01 → the open vent hole 15.03..17.42 at the
wall z −112.25..−112.34 → off the lip → the 11.84 shaft floor), and the shaft is a sealed box:
north wall solid at every body height (new probe probe_vent_north::probe_trench_north_map —
rays +z hit at y 11.9..17.4 for x 4.8..6.6), the south sill stack 14.81/15.03 3.2 m over the
floor, the vent hole the only opening. The complaint "cannot crawl through the vent" = the
one-way drop traps the player in the shaft. Open questions this session: whether the game lets
the body out (the trench-exit live check 1 — never answered: the [L 2026-10-26 hatch-3] hold was
measured SOUTH only; the game's north walk was never measured), and the port's rest-region
divergence (port rests AGAINST the north wall at −110.95; game rests at −111.38 — 0.43 further
south, bay floor shape lives partly in the cel glist). Probe 2 (real input through the real
