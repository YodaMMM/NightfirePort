## Your task: `breakable-damage-unit`, how much damage a player bullet does to a 0x20 breakable

`Break_Create` stores param 1 as an integer converted to float [M 0x12c230:22-27], so 0700000a's vending glass #379 has health 9. `Break_Update` subtracts each class-5 contact's HITDATA+8 [M 0x12bfd8:37]; the port passes the bullet's `damage`, the weapon's raw +0x0C (0.1 for the held gun 0x43). At that rate the glass takes about 90 contacts. interactables_0700000a_live's `vending_glass_breaks_under_fire` is ignored until this is settled.

1. Find what the collide pass writes to HITDATA+8 for a bullet contact on a scenery object (the HITDATA fill for class-5 hitters): raw weapon damage, a scaled value, or something else. Cite it [M].
2. The port's bullet segments overlap from tick to tick: one bullet scored on #379 on two consecutive ticks in a trace. The game registers one contact per bullet per object. Find the game's rule and match it.
3. If the decompile cannot settle the count, name the PCSX2 read for a fresh boot: shots fired at #379 until obj+0xFE bit 0 sets.
4. Un-ignore the vending test once the count is proven, with that count as its pin. Sister: the far glass #378 stays alive.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/breakable-damage-unit`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-23T20:27Z
