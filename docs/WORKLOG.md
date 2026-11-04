# WORKLOG

## audit-effect-calls
Started 2026-11-04 22:30 UTC. Branch s/audit-effect-calls (cut from origin/master b2cd660b3).
Task: audit every visual effect call in ACTION.ELF for on-foot play — Effect_*, sparks, smoke,
explosions, decals, gas, shards, light flashes, screen shakes. For each: the caller, the
condition, the effect and its parameters, and the port site. Shell casings out of scope. Fix
every missing or wrong call that has a clear port site and pin each with a test; anything
needing new drawing goes in a table with its data. Merge origin/master first.
