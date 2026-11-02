## Your task: `burst-fire-held-trigger`, burst-mode weapons fire their full burst from one pull

`crates/nf-view/tests/fire_modes3_live.rs` (~line 182) is #[ignore]d: one trigger pull on the burst mode measures 1 shot, not 3. The refire window needs the game's held-trigger path, Player_WeaponFiring 0x1a5460:601-655 [M]. Port that path exactly: how many rounds one press releases, the interval between them, and what releasing the trigger mid-burst does. Drive it through the real input path (Sim::step_bond with the trigger held, then released). If the other ignored legs in this file (~248 the 0x2D press after the 0x2C camera, ~281 the 0x2B trigger edge, ~321 the 0x32 draw stall) share the same held-trigger or edge cause, fix them too. Otherwise name each one's separate cause in the report. Un-ignore what passes. Run fire_modes3_live, fix_weapon_fire_modes_live, weapon_matrix_live and nf-view's lib tests.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/burst-fire-held-trigger`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

