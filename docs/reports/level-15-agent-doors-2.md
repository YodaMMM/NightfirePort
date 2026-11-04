# level-15-agent-doors-2 — the 15 Agent walk finished

Branch `s/level-15-agent-doors-2`, cut from `origin/s/level-15-agent-doors` and merged
origin/master (commit cd5e314f). **Status: DONE —
`walkreal_07000015_live::walk_spawn_to_the_omega_sector_exit_real_damage_agent` is GREEN and
UN-IGNORED; the whole `walkreal_07000015_live` binary runs 6 passed / 0 failed / 1 ignored** (the
1 is the d3 00 Agent sister, ignored for its own documented reason — that route's budget exceeds
100 health).

## What was done (both of the previous session's leads, inside the walk — no sim-code change, no helper change)

### 1. The #447/#448/#449 fence pops: the law pinned, the shot moved out of the blasts

The previous report's lead 2 asked to pin the pops' law (the −4.8/−4.3 at ~9.5 u "don't fit
radius 8"). A run probe printed the three mines' records straight from the sim (`Mine::damage` /
`Mine::radius`, the placement params at inst+0x1C/0x20 [M worldobj.rs's Mine]):
**all three carry damage 80, radius 10** [L the run probe 2026-11-04].

That settles every pop exactly: at the old threshold stand (9.49 / 8.61 / 8.71 u off the three
mines) the sweeps cost 80·(1−9/10)·0.6 = 4.8 and 80·(1−8/10)·0.6 = 9.6 twice = **24.0 together**
[M `Explode_Propagate` 0x17c518's sweep `damage · (1 − (int)dist / radius)` with the (int)
truncation at 0x17c61c-0x17c628, no LOS]. The walk now takes the fence shot from a mark 2.2 u
north (real walk input, no teleport): **11.13 u off #447 and ≥ 10 u off all three — the pops
drop to zero** (the truncation floor at (int)d ≥ radius), the assert's range rule (d > 8) holds
with room, and the shot still runs down the open doorway's x 0 aperture.

### 2. The west-room grenadiers: answered by proximity, not by placement spot

The previous session's lead 1: #10 (hp 20 — one head hit) survived because the rim mark's
placement-spot pre-shoots missed him (`agent_drone_near(8.0)` found none — the door routes walk
the guards off their spots) and his return-fire stands cost 21.0. The walk now sweeps at the
southwest jog — **the nearest live hostile within 14 u of Bond, and the sweep repeats while one
stands**: the measured green run took #11 at 12.5 u (hp 20, exposed), #10's residue fell to 8.4
(his own fire during the jog leg's return fire), and the door[52]-through re-fight is gone.
Net saving on the pair: 12.6.

### 3. The green run (real input only, no top-ups)

Exit USE #445 reached on the doors route; **Bond crosses at 26.2 health** (73.8 taken of 100.0 —
the previous session's ledger sat at exactly 100.0). The run's remaining world damage is the
level's own toll the report already evidenced: #526's rim-shot blast 41.6 (the geometric floor —
every stand x ≥ 14 is blocked by the rim [L probe15e]) and #462's lane shot 6.4.

## How it was verified (the binaries that ran, with their pass counts)

- `walkreal_07000015_live` whole: **6 passed, 0 failed, 1 ignored** (the Operative real-damage
  walk, the top-up walk, the mine sisters, and the Agent sister now un-ignored; the ignored one
  is the d3 00 Agent budget case).
- **Every `walkreal_*_live` binary on this branch: 28 binaries, 170 passed, 0 failed, 15
  ignored, 0 red** — 01 (8/0/0), 02 (3/0/1), 03 (6/0/1), 04 (10/0/0), 05 (14/0/0), 06 (6/0/1),
  07 (5/0/0), 08 (5/0/1), 09 (8/0/0), 0a (12/0/3), 0b (7/0/0), 0c (1/0/0), 0c_barrelrow
  (1/0/0), 0c_eastroute (3/0/0), 0d (1/0/1), 11 (3/0/1), 11_route (1/0/0), 12 (3/0/0),
  13 (3/0/0), 14 (9/0/1), 15 (6/0/1), 16 (14/0/2), 1b (17/0/2), 41 (15/0/0), 4a (4/0/0),
  4a_satchel (2/0/0), celprobe (2/0/0), dump (1/0/0).
- `nf-game` (lib + integration, `--test-threads=1`): **1048 passed, 0 failed, 8 ignored**.
- `nf-formats` + `nf-audio`: **136 passed, 3 failed** — the 3 are the slim-ISO `disc_golden`
  allowance (`bigf_header_fields_behave_as_documented`,
  `bigf_tables_match_the_golden_manifest`, `directory_tree_matches_the_golden_manifest`; the
  logged cause is `/DRIVING/MIS01.MUS: too small for a BIGF header` — the slim ISO's cut file).
- **The full suite total line**: nf-game 1048/0/8 + nf-formats 136/3 + nf-audio (inside the
  same run) + nf-view's walkreal set 170/0/15; **the rest of nf-view's non-walkreal set did not
  fit this sandbox** — the ~75 test-binary links alone outlasted the window and the disk filled
  to 89 % again mid-build. Nothing under `crates/*/src/` changed on this branch (the only task
  change is the level-15 test file), so the non-walkreal set's exposure is the merge's master
  delta, already covered by nf-game's 1048.

## What is left

- The rest of nf-view's non-walkreal binaries (time/disk-bound; the next session can run
  `cargo test -p nf-view -- --skip walkreal` after freeing `target/release/deps`).
- The d3 00 Agent sister of this walk stays ignored (its own documented budget case — not this
  task's ask).
- Live checks for the main agent: none needed — the fence law matched the measured pops at
  three distances and the walk asserts the whole route on real input.

## Usage

not visible to me — no token, credit, cost or context-window figures are exposed anywhere in
this environment (no status bar, no session summary, no API dashboard access).
