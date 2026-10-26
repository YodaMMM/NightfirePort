## Your task: `rigid-body-wiring`, barrels and loose props react to blasts and shots

`s/rigid-body-remainder` (merged to master) ported the rigid-body evolution core (rigid_body.rs: rb_update, rb_euler_evolve, rb_apply_impulses, rb_apply_wind, rb_explosion_hit_arm) but nothing calls it: the kind-0xD8 props (RB_Create 0x1d6bf0, placed on 07000001/02/03/04/06/0d/41 [D]) still draw as static instances. Wire it: create the 0xD8 placements as sim objects at load (`parsemap_create_dynamic_objects` 0x1d04f0 case 0xD8), tick `rb_update` 0x1d6e38 at the per-kind update position (dispatch table 0x26f9c8 {RB_Update, RB_CollisionHandler 0x1d7a80}), route blasts and bullets through `RB_CollisionHandler`'s hit list (obj+0xd0) including `RB_RestingContact` 0x1d7168 / `RB_RestingContactRot` 0x1d73a0 / `RB_CalcTorque` 0x1d7658, and expose each body's obj+0x90 matrix for the draw (the scene pose is done separately). Read the remainder report's "What's left" list. Tests: a resting body stays bit-identical with no shared-RNG draw [M]; a grenade next to a 07000006 barrel changes its matrix the game's way [M]; walkreal/level tests unchanged.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/rigid-body-wiring`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

