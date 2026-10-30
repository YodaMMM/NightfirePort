# WORKLOG

## level-0c-remaining-teleports-to-walks (branch s/level-0c-remaining-teleports-to-walks)

- start (master fd24fbf4). Task: walkreal_0700000c_live.rs still moves Bond with the remaining `bond.pos` writes (five on master). For each: read its comment (why it was a stand-in), find the game's own route there [D collision/placements/nav; M any class the route uses], walk it on real input, and delete the write. Where the data shows no route, keep the write but pin the gap with a sister assert and say exactly what live check would settle it. Every test in the file green; run the touched binaries and quote their totals.
