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
  t 0.41; live check listed), the deck→parachute wedge transit (#459 — walk attempted first,
  sister assert pins the pickup; live check listed), and the two stance loops (#365/#474 —
  interaction measurement spots, not route transits, justified in-file). walkreal_0700000b 3/3.
