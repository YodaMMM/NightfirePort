## level-03-fall-through-floor (2026-11-01, s/level-03-fall-through-floor)

Branch from origin/master (3f760e7d). Task: 03 — on 07000003 Bond fell through the world (y −5577)
while a walk flanked servant 1's guard in the courtyard [origin/s/stealth-los-2's first suite run;
that walk's route no longer passes there]. Reproduce on real input near servant 1's captor (find the
spot by walking the old flank — walkreal_07000003_live history on origin/s/stealth-los-2 before
003102622), find the collision gap (Dist2Tri flags 0/99 skip [M 0x10a330]; seam between floor
meshes; compare NDrone2_Collision 0x140e90's gap bridging; player: Player_FeetOnPoint 0x194188 /
Player_Collision 0x194df0), fix with the address cited, pin Bond on the floor walking that line
(sister: probe at the seam). Run every *07000003* binary, the 0a/01 walks and nf-game; quote totals.

## Progress (2026-11-01, ~09:05 UTC)

- Probes landed: flank_fall_repro (the flank walk + the lip seam probe), courtyard_wall_probe,
  feet_ray_hole_map, hole_tris_probe, slot_cross_walk (all assertion-free except the repro pins).
- The geometry [D 07000003.bin 0x2E]: a through-world void slot x 8.0..8.9, z −19.5..−16.0 — no
  collision tri at ANY depth under it (flat feet-ray grid + the raw tri dump), between the west
  ground (4.4 at x ≤ 8.0) and the courtyard plateau (5.34 at x ≥ 8.94); the wall at
  (6.35, −15.7..−17.7) blocks the west→east walk ("parks Bond west of a wall" [00310262]) and the
  nav's own way over it is airborne [c118ed46] — over the slot.
- THE FIX [M Player_FeetOnPoint 0x194188:176-185]: the game's ground flag is
  `HITTEST+0x9c != 0 || ray in band` — the port measured the capsule pick bit but did NOT gate
  grounded (the hatch-crouch-0c4 divergence). On the lip press at (12.61, −19.69) the band misses
  by 0.025 u and no walkable qualifies, so the port hung Bond ungrounded over a floor the game's
  pick bit grounds. player.rs now: `grounded = walkable || band || pick`. Measured: the lip press
  went from hanging ungrounded to 0 ungrounded ticks.
- walkreal_07000003_live still carried master's un-retraced flank leg (blob 61bb92a2 = 00310262^);
  with the fix the walk wedges at the wall (grounded ON the lip — the game's own park spot) where
  stealth-los-2's run fell through. Adopted 00310262's route (retrace the opening legs + the 0x25
  doorway fallback; "Every assertion is unchanged") — walkreal_07000003_live 4/0 (227.7 s).

## Outcome (2026-11-01, ~09:15 UTC)

- Run on the final branch tree: walkreal_07000003_live 4/0 (227.7 s), walk_07000003_live 2/0,
  interactables_07000003_live 8/0/1i, wiring_07000003_live 4/0, walkreal_0700000a_live 7/0/2i,
  walkreal_07000001_live 5/0 (187.1 s), flank_fall_repro 2/0, nf_game lib 631/0/5i. The full
  fast-suite pass did not fit (background reaped between tool calls; the 9.9G disk — same as the
  previous session). Report: docs/reports/level-03-fall-through-floor.md.
