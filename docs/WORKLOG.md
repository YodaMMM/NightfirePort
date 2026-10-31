# WORKLOG

## level-0b-teleports-to-walks (branch s/level-0b-teleports-to-walks)

- start (master fd24fbf4). Task: `walkreal_0700000b_live.rs` still moves Bond with seven
  `bond.pos` writes (~lines 221, 246, 315, 365, 391, 459, 474). For each: read its comment,
  find the game's own route there [D collision/placements/nav; M any class the route uses],
  walk it on real input, and delete the write. Where the data shows no route, keep the write
  but pin the gap with a sister assert and say what live check would settle it. Every test in
  the file green. Quote the touched binaries' totals.
- outcome (2026-10-31): three of the seven `bond.pos` writes are GONE, replaced by the game's own
  routes: the lift fallback (#221) — the boarding + ride + step-off are a hard assert (both walks
  board on real input, deterministically); the desk approach (#315) — the GAME'S route is the
  keypad: the Laser watch 0x4E burns Lock #178 [M Lock_Update 0x1ae3d8 case 0: class-5 bullet
  weapon 0x4E burns, Lock_Unlock writes ch 47] → door #151 unlocks [D ch_lock 47] → the walked E
  → n26 → the desk (walked, hard-asserted, both walks); the E-plate #166 return (#391) — the
  walked return through the burned doorway (n26→n25→n22→n21→n20→n16) or the proven terminal lane
  (n14→n15→n16, the fresh walk's position). Kept: the waiting-room exit transit (#246 — the
  documented stand-in: door #0's E opens it but the open leaf's pose still blocks the passage at
  t 0.41; live check listed) and the two stance loops (#365/#474 — interaction measurement
  spots, not route transits, justified in-file); the deck→parachute transit (#459) ALSO fell —
  the deck walk arrives (+471 ticks, both walks, hard-asserted) and only the parachute's stance
  sweep remains (the helicopter body's volume blocks the walk 2.4 m short [D the run dump cel
  189], the walk-over drive inside the sweep is real input). walkreal_0700000b 3/3.
- suite (2026-10-31, the final tree): the runner's run phase was stopped at the deadline —
  286/971 binaries done, 958 passed, 3 failed (the two GPU-less wgpu render binaries — adapter
  NotFound in this sandbox, untouched by this test-only change), 0 TIMEOUT/CRASH. Touched
  binaries green: walkreal_0700000b 3/3 (twice), interactables_0700000b 9/9. Report:
  docs/reports/level-0b-teleports-to-walks.md.
