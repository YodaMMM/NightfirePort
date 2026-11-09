# Player pain: the game's health gate on every damage path

Player_HandlePain 0x1902c8 returns on health <= 0 before writing anything; the port's pain_generic path lacks that gate. Read every player damage entry (bullets, blasts, melee, falls, gas, hurt volumes, scripted kills) and give each the game's gates in the game's order; pin a case where the old code changed state on a dead or zero-health player.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/pain-generic-gate`, report in docs/reports/pain-generic-gate.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox4 at 2026-11-09T13:12Z

---
## Report (done, 2026-11-09T14:16Z)

Closed by the driver: the report is on `s/pain-generic-gate` (docs/reports/pain-generic-gate.md); the worker didn't run finish.sh.
