# level-0c-yard-climb-real-health-4 — report

Task: finish 0700000c's yard climb on real health — continue `s/level-0c-yard-climb-real-health-3`,
take the -3 report's named next step, use the aimed act for the deck sentries, re-check every
remaining climb hit against the game's sight/accuracy/damage path, un-ignore the real-health sister.
Real input only; no health/ammo/channel writes; no teleports; never lower damage or health.

## What's done

**The task's code work is complete and verified green on a fresh sandbox.** The implementation landed
in commit `ea40c2ba` on this branch (the stretch before this session; the sandbox had reset since, so
this session rebuilt the environment and re-verified everything from scratch on the final tree — the
branch already contains the clean merge of `origin/master`, commit `5ade7dbe`).

### The fix (commit ea40c2ba)

The -3 blocker (the west sniper #13 killing the walk at tick 14308) was not the ledge lip. The shot
trace (`NF_RF_TRACE` + the player_impacts dump) showed the fight stands' rounds flying 2–3° off the
converged view and dying on the wall behind #13, because:

1. The stands fired with the aim bit false, so the weapon raise never ran — the game queues the raise
   only on the pressed edge from the idle machine [M 0x1a24c8:113-152] — and every round rode the hip
   cone (spread base 100.0 [L ee_start_05.bin @ 0x2c10b8 +0x24]).
2. The port scanned auto-aim per fire path and froze the walk legs' assist offset through the aim
   hold, steering the first aimed rounds −12° off (measured aoff −0.216 [L the NF_RF_TRACE impact
   trace]). The game eases the assist offsets toward the manual offsets 0.5/tick EVERY aim tick and
   runs the Check_AutoAim scan only on button-up ticks [M Player_AutoAim 0x1a7c98:50-59].

Port fixes, each citing the game address above:

- `crates/nf-view/src/scene.rs`: the per-tick aim-branch ease (in the `step_bond` path, right after
  the aim gate) and the raw-button scan gate in the fire path's dir pick (the bit-gated scan from
  fix-aim-raise2 is corrected to the decompile's raw-button gate).
- `crates/nf-view/tests/common/walkreal0c.rs` (0c-local only — used by exactly three binaries:
  `walkreal_0700000c_live`, `walkreal_0700000c_eastroute_live`, `upward_shot_0700000c_live`; no other
  level's walk includes it, so the per-level opt-in rule holds): the fight stands gain the raise loop
  (settle, then hold aim until the bit sets), the aim-bit fire gate, and cursor-aware convergence
  (the effective aim = the view + the eased cursor offsets).

### The sister

`walk_the_whole_mission_from_the_start_to_the_0d_handoff_real_damage` in
`crates/nf-view/tests/walkreal_0700000c_live.rs` is **un-ignored and passing on real health**: the
whole mission walked from the start (0c 19287 + 0d 838 ticks, real input only), Bond alive at 26.2 hp
at the 0d arrival. The west sniper #13 dies to the aimed stand from the spot whose gun rays clear;
the ledger at the handoff is green. No health, ammo or channel writes; no teleports; no damage or
health values lowered anywhere.

## Verification (this session, fresh sandbox, final tree already merged with origin/master)

Environment rebuilt from zero (clone, work-slim, ISO, toolchain), test binaries rebuilt release,
then run per binary with `--test-threads=1`. The fast runner (`tools/test/suite.py`) was used for
grouped chunks; because this sandbox kills background jobs between tool calls, the sweep ran in
foreground chunks sized to finish inside each tool call. **848 tests passed, 0 failed** (1 ignored,
pre-existing and intentional):

| Group | Result |
| --- | --- |
| `walkreal_0700000c_live` (whole binary: ladder495 + whole-mission + **real-health sister**) | 3 passed, 0 failed, 0 ignored |
| 0c users + climb/mantle group via suite runner: `walkreal_0700000c_eastroute_live`, `upward_shot_0700000c_live`, `fix_ladder_live`, `fix_ladder2_live`, `fix_ladder_water_live`, `ladder_02_live`, `ladder_underwater_02_live` | 24 passed, 0 failed, 1 ignored (`probe`, pre-existing) |
| Mission `walkreal_0700*_live` binaries, 23 of them directly (01:6 02:2 03:5 04:8 05:14 06:6 07:5 08:4 09:6 0a:10 0b:5 barrelrow:1 0d:1 11:2 11_route:1 12:3 13:3 14:9 15:6 16:14 41:11 4a:2 4a_satchel:2) | 126 passed, 0 failed |
| `nf_game` (lib + integration) | 656 passed, 0 failed |
| scene.rs-affected aim/fire binaries: `fix_aim_raise_live`:5 `fix_guard_misc_live`:6 `fix_autoaim_elevator_live`:1 `mouse_scope_live`:3 `scorpion_modes_live`:1 `drone_weapon_drop_live`:3 `civilian_no_shoot_live`:3 `abseil_damage_matrix`:4 `fix_viewmodel_zoom_live`:3 `fix_hud_zoom_tilt_live`:3 `fix_pickups_ammo_live`:4 `fix_aims20_modes_lights_live`:3 | 39 passed, 0 failed |

`walkreal_07000014_live` is simply slow (140 s); it passed with a 300 s per-binary limit after
timing out an earlier 120 s cap — noted so nobody reads the first line as a failure.

## What's left

- The full-suite single total line: this sandbox kills background jobs between tool calls, so the
  one-shot `nohup` sweep is impossible here; the sweep above covers every required group plus the
  lesson-10 targets for the touched code. This task's code delta vs its base is exactly three files
  (`scene.rs`, `walkreal0c.rs`, `walkreal_0700000c_live.rs`); the branch's full delta vs
  `origin/master` additionally carries the predecessor -1/-2/-3 changes (the difficulty3_01 and
  walkreal_01/02/03/04/08/09/0a/0b test files, WORKLOG, reports 2/3) — every level those files touch
  was re-run green in the table above. Not run this session: the ~53 `walkreal_probe*` 0c-local
  probe binaries, the remaining non-aim `fix_*`/probe binaries, and the `nf-formats`/`nf-audio`
  binaries (no code in those crates changed on this branch).
- No live PCSX2 render is needed: nothing visible changed (input-path logic only).

## Live checks for the main agent

None required. If cheap, one confirmation render would be nice-to-have: from a fresh 0c boot, at the
pre-climb fight stand, hold aim ~1 s before firing and confirm the rounds land on the deck sentry
rather than the wall behind it — the port now eases the assist offsets per aim tick
[Player_AutoAim 0x1a7c98:50-59], and a real-game confirmation of the eased impact point would upgrade
the [L]-trace inference to a direct measurement.

## Usage

Not visible to me: this environment exposes no token, cost or context-window figures for the session.
