## Your task: `bond-hitbox-pose`, Bond's 16 hit boxes on every level, posed by his state

Start from `origin/s/reload-and-bond-hitboxes` (NOT merged). It replaces the guard rounds' cylinder test on Bond with the game's 16 `HumanCollisionBones` boxes (0x2c6be8 via `Collide_Jointy`, skin 0x05000007; `crates/nf-game/src/damage.rs` `bond_boxes`) over one measured standing idle pose. It cannot merge: `bond_boxes` returns NO boxes when skin 0x05000007 is not in the level's skin table, and then no guard round or melee ever hits Bond — `fix_bond_damage_live` (5 tests: "100 x 0.6 = 60 health, got 100") and `anim_events_live::melee_event_creates_the_close_combat_bullet` fail.

1. Bond's skin is always loaded in the game (the player object's AnimObject, model record hash 0x5000007 [L ee_start_05]). Find how the game gets it on every level [M — the player init / the session skin table] and make the port's boxes available on every SP level the same way, with a test that loads every on-foot level and asserts 16 boxes. No silent empty list: if the skin is truly missing, that is a load error, not an unhittable Bond.
2. Pose the boxes by Bond's state (at minimum crouch vs stand) [M — where Bond's palette is updated and which third-person anim runs; L a crouched palette read], with a test: a round just above a crouched Bond's head misses, the same round hits him standing.
3. The full suite on the final tree: `fix_bond_damage_live` and `anim_events_live` green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/bond-hitbox-pose`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.



---
claimed-by: sandbox4 at 2026-10-26T11:32Z

---
## Report (done, 2026-10-26T13:35Z)

