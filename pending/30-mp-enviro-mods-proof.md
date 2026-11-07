## Your task: `mp-enviro-mods-proof`, Enviro-Mods rows are proven in real matches

Branch from origin/s/mp-enviro-mods-apply, merge current origin/master, and read its docs/reports/mp-enviro-mods-apply.md. The apply work is in, but its proof did not fit that window. Take its "What's left" in order. Write crates/nf-view/tests/mp_enviro_mods_live.rs: row 1 off gives no mounts on a mount-carrying arena; on with weapon set 5 gives `turret == 5` on each mount; on with set 10 gives the seed-keyed pick, below 10; a two-machine net_tick run with mounts on agrees on every tick's hash and on both mount arrays, which is the in-step proof the old clear was waiting for; row 2 off leaves `props.destroy` empty on a destroyable arena and on fills it; row 3 on gives the load's kit 0x50 and off does not. Pick the arena from the mp_world_sync inventory print. Then audit the setup and summary pages that echo the rows, and use `mp_enviro_value_label` there if they print them. If mp_screens pins the locked row's old text, update it citing 0x210030. Run every mp_*_live binary, mp_world_sync_live, mp_defaults_setup, mp_menu_flow, menus_e2e_mp, mp_screens and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mp-enviro-mods-proof`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

