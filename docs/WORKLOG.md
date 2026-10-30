# WORKLOG

## level-0b-teleports-to-walks (branch s/level-0b-teleports-to-walks)

- start (master fd24fbf4). Task: `walkreal_0700000b_live.rs` still moves Bond with seven
  `bond.pos` writes (~lines 221, 246, 315, 365, 391, 459, 474). For each: read its comment,
  find the game's own route there [D collision/placements/nav; M any class the route uses],
  walk it on real input, and delete the write. Where the data shows no route, keep the write
  but pin the gap with a sister assert and say what live check would settle it. Every test in
  the file green. Quote the touched binaries' totals.
