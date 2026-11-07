# Function map: bullets

Branch from current `origin/master`. `docs/port-map/drones.md` listed every drone function as ported or not reached, and that sweep found a real missing rule (the 08 escort fail). Do the same for this family: one line per function in the decompile, ported with the port location, or not reached with the reason. A comment does not count as ported; find the code. Any function a real level reaches with no port code is a gap: fix it with a test on the real level, proved red on the old code, each fix in its own commit. Write the table to docs/port-map/bullets.md. Do not write a hex address containing the characters of a function key in a table; if the push guard trips, describe the row by name.

Family: every `Bullet_*`, `Gun*`, `Weapon*`, `Explode_*`, `Effect_*` function.

---
claimed-by: sandbox3 at 2026-11-07T00:22Z

---
## Report (done, 2026-11-07T01:04Z)

Closed by the driver: the report is on `s/port-map-bullets` (docs/reports/port-map-bullets.md); the worker didn't run finish.sh.
