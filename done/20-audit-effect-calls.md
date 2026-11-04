# audit-effect-calls — every visual effect call against the port

Today's sweeps of sounds, text and HUD panes each found gaps that tests never caught, so do the same for effects. List every call in ACTION.ELF for on-foot play to the effect families: Effect_*, Sparks, smoke, explosions, decals, gas, shards, light flashes and screen shakes. For each: the caller, the condition, the effect and its parameters, and the port site. Shell casings are out of scope; do not change them. Fix every missing or wrong call that has a clear port site and pin each with a test; anything needing new drawing goes in a table with its data. Merge origin/master first.

---
claimed-by: sandbox1 at 2026-11-04T22:26Z

---
## Report (done, 2026-11-04T23:39Z)

