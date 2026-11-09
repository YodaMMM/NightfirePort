# Explosion debris fades out

Debris_Update steps the piece's alpha byte obj+0x106 down as it dies, but the port draws debris at full alpha until it vanishes. Read the fade (start, step, end) in the decompile and port it into the debris pieces' sim state so the draw can use it; pin the per-tick alpha. Name the draw hook needed (the draw side is the main agent's).

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/debris-alpha-fade`, report in docs/reports/debris-alpha-fade.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox1 at 2026-11-09T13:43Z
