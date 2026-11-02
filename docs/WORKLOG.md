
## consider-explosive-gate (branch s/consider-explosive-gate, from origin/master a1cfe068)

Started 2026-11-02 13:38 UTC (sandbox reused from the level-15 session: repo, work/, ISO, alsa prefix, queue worktree all present; 723M free — will strip/prune before big builds).
- Task: GAPS G-055 — the port's DroneFunc_ConsiderExplosive gate refuses more states than the game's (game: 0x28, 0x2d, 0x4a-0x4b, 0x53-0x55, 0x63, 0x86-0x88 [M 0x1431c0:40-67]); port a guard in AimStandFire 0x68 never flees a grenade. Plan: (1) read the decompile at 0x1431c0 and the port's gate, (2) failing test: guard pinned in 0x68 flees a live grenade through real input; sister: 0x63 does not, (3) port the gate with the address, fix fragile walks on the walker side, (4) run ai_live, the grenade/explosive binaries, every walkreal_*, nf-game; suite.py in background; quote totals.
