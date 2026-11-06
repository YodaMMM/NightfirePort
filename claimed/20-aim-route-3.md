# Aim: the last routes

Branch from `origin/s/aim-route-finish`, merge current `origin/master`, push to `s/aim-route-3`. Read docs/reports/aim-route-finish.md. 14 is green.

1. 1b: build the report's recipe: count answered holds per console, and start flying to the next station once the last hold of the current console is answered. It estimates 8 still lands about 150 ticks late, so measure, and say what else the chain needs. Goal: `MissileDeploy` clear and both drakes walks green.
2. 08 x1, 0c x1, 16 x1: the levers the report names. 0c's finishers sit at d 61-90: pistol-class weapons have range 50 and rifles 150+ [weapon_data +0x1c], so check what Bond holds before any stand; a pistol cannot answer them.

Test-side only. Keep the [L] 08 pins and the 0x30000000 fix. Finish with the aim binaries and walkreal 08 / 0c / 14 / 16 / 1b whole.

---
claimed-by: sandbox3 at 2026-11-06T15:06Z
