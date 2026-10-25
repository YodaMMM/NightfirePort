## Your task: `dmode-inits-hud-panes`, the last never-mentioned on-foot functions: drone-mode inits, HUD pane creators, Text_SendMessage

A whole-ACTION.ELF scan of master (4c1f5ffb8) finds these game functions never named or cited in crates/*/src:
- the 28 `NDrone2_init_DMODE_*` functions (Guard 0x14d978, Retreater 0x14d998, Sniper 0x14d9b8, Stealth 0x14da78, Attacker 0x14dab0, RunToPoint 0x14dad0, Assassin 0x14db18, HostageKiller 0x14db38, Hostage 0x14dc18, HostageTied 0x14dc88, JustStand4Demo 0x14dcf8, DeleteMe 0x14dd20, and the rest of the NDrone2_init_DMODE_* family in the decompile);
- `HUD_CreateAmmoPane` 0x1a0ed8, `HUD_CreateHealthPane` 0x1a0f40, `HUD_CreateInfoStatusPane` 0x1a10b8, `HUD_CreatePickupStatusPane` 0x1a1140;
- `Text_SendMessage` 0x223600.

For each [M]: what it does, who calls it on the on-foot path, and what the port has in its place. Port it where the port lacks or differs (cite the address at the port's code), or prove it equal (cite it at the equal code) or unreachable on-foot (the caller census). A test per ported behaviour (e.g. a DMODE init's settings on a placed guard of that mode; a HUD pane's layout values). Every existing test stays green; run the full suite and report the total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/dmode-inits-hud-panes`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
