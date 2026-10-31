## level-0a-bridge-reach (2026-10-31, s/level-0a-bridge-reach)

Branch from origin/master (bf6b5a2a, batch-1031g). Task: 0a reach the bridge on real input —
walkreal_0700000a_live still teleports Bond to [-11.35, 1.12, -18.6] (REACH onto the bridge's
west end, ~line 884) and [-8.0, 1.12, 16.29] (~line 1005). Find the game's route to each spot
[D collision/placements/nav; M the classes used], walk it on real input, delete the writes;
where the port wedges where the game does not, find the collision difference with evidence
(capsule rules: head-on cull on the raw denom [M 0x10a1f0-0x10a2e8], Dist2Tri flags 0/99
[M 0x10a330/0x10a33c]). If the data shows no route, pin that with a sister assert and name
the live check.

## level-0a-bridge-reach, outcome (2026-10-31)

Both REACH teleports in walkreal_0700000a_live are deleted and replaced with real-input walks:
the bridge west end (−11.35, 1.12, −18.6) is reached by walking the main walk's own prologue +
ring verbatim to the keypad's wall, then south-east onto the strip; the hall line (−8.0, 1.12,
16.29) is reached by the through-line through the doorway + the east return along the hall.
Findings pinned in the test comments: the straight lobby approach wedges on the jamb column's
south-east corner where the game's route never goes (the through-line is the game's route); the
desk lane's civilian g8 patrols the lane itself and a fresh-state crossing meets him head-on →
challenge → the alarm press → fail 0x4000032 — the main walk's phase (the PC detour) keeps him
out of sight, so the bridge test shares its bit-identical timeline. walkreal 7/0/2 ignored,
interactables 7/0/0, walk 3/0, keyboard 1/0. Partial suite (218/986 binaries in the sandbox
window): 741 passed / 0 failed / 62 ignored, no TIMEOUT/CRASH.
