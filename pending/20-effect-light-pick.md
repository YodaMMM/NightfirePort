# Effect lights picked at the piece, not the origin

Effect draws (debris, smoke) pick their lights at the world origin in the port; the game picks them at the object. With several lights nearby this lights pieces differently. Read the game's light pick for effect objects and port the per-object pick in the sim-side data (the draw use is the main agent's); pin which lights are chosen for a piece near two lights.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/effect-light-pick`, report in docs/reports/effect-light-pick.md. No draw changes: name what is wrong if it is a draw issue.
