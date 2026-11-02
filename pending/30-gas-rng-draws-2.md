# gas-rng-draws-2 — land the gas RNG draws without breaking the level runs

Branch from origin/master. Base the work on origin/s/gas-rng-draws (merge it into your branch first).

That branch adds the game's RNG draws to the gas-puff init (`GasPool::init` now takes `&mut Rng`). Merged onto master it turns 20 tests red: the extra draws shift the shared drone RNG stream, so guard decisions, accuracy rolls and routes change on every level that spawns gas.

Red test files (all pass on master without the branch):
doublecross_transitions_live, enemy_accuracy_live, fix_blast_scripts_live, fix_finale_live, hold_npcs_live, hostage_killer_07000002_live, mayhew_hold_live, mp_bot_armour_online_live, mp_perf_16_live, sight_frustum_live, walkreal_07000001_live, walkreal_07000002_live, walkreal_07000003_live, walkreal_07000014_live, walkreal_07000016_live, walkreal_0700001b_live

Also: crates/nf-view/tests/goggles_render_live.rs calls `gas.init` with the old 8 arguments; update it.

Do:
1. Prove which RNG the game's gas init draws from [M the gas init decompile]: the shared stream or a separate one. If separate, give the port's gas pool its own Rng and the drone stream stays unchanged (most of the reds should go away).
2. If it really is the shared stream: each red test must be re-checked, not re-pinned blindly. A walk that now fails to complete a level is a real regression to fix; only a digest or a count that the RNG shift legitimately moves may be re-pinned, citing the decompile.
3. Run every listed test file and the goggles test; all green. Then the full suite.
