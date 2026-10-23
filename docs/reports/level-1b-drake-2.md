# level-1b-drake-2 — 0700001b: reaching and defeating Drake (round 2)

**Status: not done — the two tests stay `#[ignore]`d, both at a NEW, precisely mapped blocker.** The round's findings below are all disc/decompile-pinned; the next round continues from this branch.

## What the round set out to answer

The driver's question: *where does the game's player carry come from (the script stream, the event kind, the channel)?*

## Findings

1. **The event-6 census [D]** (probe1b_scripts::dump_0700001b_script_chain over the whole loaded lib): event kind 6 appears in exactly two scripts of 0700001b — **0x0600085c stream 7** (`ev0x04[] ev0x06[0x2]`, with stream 6's `ev0x05[]` + stream 8's `ev0x07[]`) played by **SP #147** (the level-start intro cinema, loadable pack `0780001b.bin`), and **0x0600085f stream 6** (`ev0x06[0x0]`, pack `0790001b.bin`, no SP player found). **Neither is 0x060008f0** — the previous round's "no event 6 in 0x060008f0's parsed streams" is confirmed and explained: the fly-by NIS carries no event 6 at all.
2. **The port's event-6 path already exists** [M 0x1d94c0 case 6 — the mangling of `Mat_Copy__FPC6MATRIXP6MATRIX(const CMATRIX*, CMATRIX*)` makes `glb_players+0x90`/`iVar5+0x90` the SOURCE and `SCRIPTINFO+0x40` the DEST: a snapshot, not a drive; `Script_Free` 0x1da880 then enables the player at it]: `crates/nf-game/src/nis.rs` `event()` case 6 snapshots `player_at`/`player_mode` (flags 0x100) and `free()` pushes `Request::PlayerEnable`. No code change was needed for the carry itself.
3. **SP #147 (0600085c) can never start via SP_Update** [M 0x1dbda8 + SP_Hit 0x1dc470:11-13 — the out-channel trigger bit reads `switch_channels[out]` only when `out != 0`; #147 has out 0, mask 1]: it is the intro cinema started by the level start (docs/systems/levels.md), not a route piece; the port's arrival stand-in (the substate-8 hold) covers it.
4. **The void crossing is the zero-G float, and its entry is now ported**: the level's script chain pulses ch 0x4b [D 0x06000971 streams 2/3 — played by 0x060008a3 stream 6 (SP#143) and by 0x060008f0 streams 8/9/11], and `C_CHCHZEROG_Handler` 0x216388 folds `Player_ChangeSubState(glb_players, 9)` on it [M 0x216388:10-17]. Ported into `scene.rs` (the round-1 block, with its [M] citations). The disc trigger chain ABOVE the pulse is unreachable — ch 19 (the launch order) has no writer in the level data [D scans; the level doc §6] — so the test drives the pulse as the documented stand-in, the same class as the ch 0x0a fight-gate drive. `[S] unverified`: the channel→control binding (no ACTION.ELF sender of the msg-0x4b control was found).
5. **The sealed shaft wall (the new blocker, probed)**: with substate 9 entered and steered (all real thruster input), Bond rounds the island south (reaches (5.00, 0.64, −3.44)) and is then capsule-stopped at **x ≈ 4.98 across the whole scanned band z −8..−0.5, y −2.4..+2.5** (world segment hits at t 0.079 from x 5.5 → face ≈ 4.43), while the **bullet segment crosses everywhere** — a glass wall, shootable but not floatable (probe1b_float::scan_the_x5_wall_openings). The route out is the shaft mouths at **z ±8.82** (SP#141/#142's looping mode-8 zero-G scripts 0x06000818, out ch 0x1c/0x18) — unprobed this round.
6. **The interactables test's blocker is explained [M]**: the spawner's min-distance screen [M 0x137cb0:30 — a PRE-selection usability test] refuses Drake's spot while Bond stands in the arena basin (the walk arrives ~1.5 from the spawn spot), so its arena-floor gate drive can never spawn; the game's own beat fires the gate from the deck (walkreal's green `the_fight_gate_spawns_drake_from_the_deck` pins it). The test's ignore message now carries this; the re-route needs the same float crossing.

## What runs (the rewritten walkreal chain, `#[ignore]`d at the wall)

Real-input proven up to the wall: the deck walk → ch 0x0a (documented stand-in) → Drake spawns+enables from the deck → the deck walk to the slit mouth → the 0x4b pulse (one-tick rise, the disc's own shape [D 06000971 s2/s3]) → **substate 9 grabbed (asserted)** → the steered thruster float south around the island → stopped by the wall. The steering code stays in the test body for the next round.

## Live checks for the main agent

1. **The 0x4b → zero-G entry binding** [S]: on a FRESH boot, during the slit crossing, read the zero-G substate byte (the port's `bond.substate` analog, the player object's substate field — the same address the p3==2 start reads substate 8 from) the moment the shuttle/fly-by passes: expect 9 (not 8) while Bond floats, and watch whether `switch_channels[75]` (0x4b) pulses coincide.
2. **The wall** [D/L]: any console/geometry dump that names the mesh at x ≈ 4.4, z −8..0 (the shaft glass?) — or just confirm on a fresh boot that Bond floats THROUGH where the port's capsule stops (i.e. the wall is port-added collision, not game collision).
3. **The shaft mouths** z ±8.82: whether the game's route enters the SP#141/#142 shafts from the deck level or from the float.

## What's left

- Map the shaft entries at z ±8.82 (extend probe1b_float's scan to z −9.8..−7.8 and the vertical columns there), port whatever set-piece the shaft ride needs (SP#141/#142's scripts are already primed loopers in the port), then the landing, the fight, and the un-ignore with the sisters.

## Verification (what ran)

- `cargo test --offline --release -p nf-view --test walkreal_0700001b_live --test interactables_0700001b_live --test probe1b_float --test probe1b_scripts` — 0 failed (walkreal 3 passed / 3 ignored incl. the chain test; the probes assertion-free).
- `--test fix_finale_live` (4 passed — it exercises 0x060008f0's chain), `--test walk_0700001b_live` (2), `--test simplescript_live` (1), `nf-game --lib` (522 passed).
- The background `tools/test/suite.py` died in its first minute (`cargo` off PATH in the nohup environment; the prior task documented the same sandbox limitation) — the foreground chunks above are the window's coverage. The full suite did not run on the final tree; the crates the change touches (nf-view scene + tests; nf-game src untouched this round) are green.

## Usage

not visible to me (no token, cost or context-window figures are exposed in this environment).
