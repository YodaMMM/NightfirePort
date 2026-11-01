## level-03-fall-through-floor (2026-11-01, s/level-03-fall-through-floor)

Branch from origin/master (3f760e7d). Task: 03 — on 07000003 Bond fell through the world (y −5577)
while a walk flanked servant 1's guard in the courtyard [origin/s/stealth-los-2's first suite run;
that walk's route no longer passes there]. Reproduce on real input near servant 1's captor (find the
spot by walking the old flank — walkreal_07000003_live history on origin/s/stealth-los-2 before
003102622), find the collision gap (Dist2Tri flags 0/99 skip [M 0x10a330]; seam between floor
meshes; compare NDrone2_Collision 0x140e90's gap bridging; player: Player_FeetOnPoint 0x194188 /
Player_Collision 0x194df0), fix with the address cited, pin Bond on the floor walking that line
(sister: probe at the seam). Run every *07000003* binary, the 0a/01 walks and nf-game; quote totals.
