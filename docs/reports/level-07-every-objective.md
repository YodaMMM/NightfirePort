# level-07-every-objective — 07000007 completable end to end on real input

Task: `walkreal_07000007_live.rs` walks the WHOLE mission 07000007 (the Exchange part 3, the
party) from a fresh load on real input (`step_bond` move/aim/fire/use; no teleports past a
blocker, no channel/state pokes): every objective the level's mission data lists [D the
objective records / Mission_MonitorObjectives 0x1827d8 M] in player order, each asserted
complete, ending on the level exit firing. First list what the current tests already cover
(and every stand-in/poke), add legs for the rest; every blocker is a port gap to fix [M/D] or
list (drawing-only). Full suite, quote its total line.

## The objective inventory — 2/2 already covered, nothing left to add

The disc's MissionData row for 07000007 (mission 0x07000005, part 2) carries EXACTLY TWO
objective records [L ee_start_05.bin @ 0x2a43a0 → table 0x2a3e58, count 2 — re-dumped byte for
byte this session against the port's `crates/nf-game/src/mission/c07000007.rs`; docs/formats/
mission-data.md record shape]:

| # | objective [D text id] | completes on | revealed by | asserted where (all on the real-input path) |
|---|---|---|---|---|
| 1 | "Rendezvous with undercover agents and maintain cover as party guest" (0x04000002, fail none, hint 0x0400005d) | ch 0x48 | ch 0x47 | `obj07().0 == 3` after spawn plate TouchOnce #411; `== 4` after the hall walk-in plate #412 (the existing `walk_the_party_start_to_finish`) |
| 2 | "Spy on secret meeting" (0x04000003, fail none, hint 0x0400005e) | ch 0x49 | ch 0x48 | armed-unannounced (`== 1`) from the load [NEW]; announced (`== 3`) by the same ch 0x48 [NEW]; `states[2][1] == 4` after 07000008's intro plate #237 through the game's own exit-carry (pre-existing) |

There is no third record — the wiki's "photographs" objective has no disc evidence (no camera
weapon in weapon_data, no photo channel; docs/levels/07000007.md §2 [S]). Both objectives are
covered END TO END by the pre-existing walk (spawn → party → hall ch 0x48 + the control-taking
cut-scene 0x06000073 → Drake's reveal ch 0x15 + the fly-through 0x06000072 → east stairs → E on
exit LOAD_LEVEL #418 → `level_exit == Some(0x07000008)` → the 07000008 carry → plate #237 →
ch 0x49), and the "maintain cover" half of objective 1 is additionally pinned by
`the_group_plate_wakes_the_walkers_and_the_guards_stay_calm` (the ch-6 group wakes onto its
kind-8 routes, and no bouncer enters combat while Bond crosses unarmed [M CivilianGuard
0x1603a8 case 3]).

## What's done (this session)

1. **The objective-2 reveal chain is now asserted on the walk** (commit 91927210): the two
   missing links of its state machine — armed-unannounced `states[2][1] == 1` straight after
   the load [M Mission_Init 0x182270:98-105 (port mission.rs init: enable != 0 → 1)] and
   announced `== 3` by the same ch 0x48 that completes objective 1 [M 0x1827d8:55-67 (port
   mission.rs monitor case 1: state 1 + enable channel on → announce)]. With the pre-existing
   `== 4` on 07000008 the chain 1 → 3 → 4 is fully pinned through real play.
2. **The inventory + stand-ins are written into the test's doc header** (commit 1a1e0aa1), per
   the task: which objectives the tests cover and every stand-in still carried.
3. **The stand-in audit** (nothing is a poke; the walk writes no channel/state directly):
   - the P2K auto-draw (`Upgrade_Weapon(6)` on the ch-0x48 rising edge [M Player_Weapon
     0x1a24c8:247]) is owned by the view loop, which the walk harness steps around — the walk
     asserts the LOCK LIFT through the game's own `weapon_lock_input` [M 0x1a24c8:243-246] and
     keeps Bond on the loadout fists; the draw itself is pinned through the view loop by
     `level_carry_live`;
   - the 07000007 → 07000008 hand-off is the game's own carry (the exit trigger's `level_exit`
     → RamSave + mission states [M 0x1b2318:111-116]), and Bond then WALKS from 07000008's
     start to plate #237 on real input — no teleport.
4. **No port gap blocks the walk** — every blocker listed in the task's examples was already
   either fixed and pinned (the story door #251 close/lock by `party_door_lock_07000007_live`,
   the urn SP #312 by `urn_break_live`, the bouncer rules by `fix_drone3_live`) or is inert
   boundary data (the ch-200 locked doors [D census — no writer on this level]). The known
   drawing-only gaps around this level stay documented in docs/levels/07000007-audit.md §4
   (GAP-3 dynobj draw-side, GAP-4 idle-guest face windows) and GAP-1 (guest hard-collision
   missing — Bond walks through the crowd; behavioural, but nothing in the objective path
   depends on it, and it only makes the walk EASIER, not wrong).

## How it was verified

- The mission-record dump re-taken from the EE snapshot this session (the python dump of
  ee_start_05.bin @ 0x2a43a0 → 0x2a3e58, 2 records, quoted in full above) matches the port's
  `c07000007.rs` byte for byte.
- `walkreal_07000007_live` on the pre-edit tree: **2 passed, 0 failed, 0 ignored; 11.11s**
  (the binary run directly with `--test-threads=1`), covering the full route including the
  exit firing and the 07000008 carry.
- The two new asserts (commit 91927210) re-verified on the REBUILT binary (the suite's cached
  build, timestamped after the commit; both new panic messages confirmed present in the binary):
  `walkreal_07000007_live` run directly with `--test-threads=1` — **2 passed, 0 failed; 14.40s**.
  The suite itself also re-ran the other half of the chain, `walkreal_07000008_live`: 3 passed,
  0 failed, 94.12s (the 07000008 side of the exit-carry and objective 2's `== 4`).

## The full suite

The fast runner was started in the background per the task (tools/test/suite.py, NF_ISO set,
NF_SILENT=1, the warm CARGO_TARGET_DIR from the prior sessions). This sandbox's 9.9 GB rootfs
is effectively full with that warm build cache, and the first runs died with ENOSPC during the
`cargo test --no-run` build phase; ~6.2 GB of stale per-session test binaries had to be pruned
from the cache (keeping every rlib/rmeta) before a full build of the four crates' ~888 test
binaries could complete. The runner then executed serially (`--test-threads=1`, 600 s per
binary).

**The 892-binary run needs longer than the sandbox window.** Started 04:45 UTC, it ran at
~3–6 binaries/min (many level-loading `*_live` tests take 10–100 s each); this sandbox expires
~07:28 UTC and work stops 07:08 UTC. At the last read (~05:34 UTC) it had completed **329 of
892 binaries with 1134 passed, 2 failed, 86 ignored** (interim sum of the per-binary
"test result" lines in `~/suite.log`), plus 2 CRASH binaries. If the run lands before expiry,
the final total line replaces this paragraph; the task's own test (`walkreal_07000007_live`)
was verified directly regardless (see above), and `walkreal_07000008_live` passed inside the
suite itself.

The 4 non-passing binaries so far, with evidence — all environment, none touching this diff
(`git diff origin/master --stat`: only `walkreal_07000007_live.rs` +36 lines and `WORKLOG.md`):

- `fix_0c_pylon_live` (level 0C, another task's test) — 2 FAILED: wgpu panics with
  `adapter: NotFound { requested_backends: NOOP | VULKAN | GL | METAL | DX12 | BROWSER_WEBGPU,
  supported_backends: VULKAN | GL ... }` — this headless sandbox exposes no GPU adapter for a
  render surface. Drawing-only, and not this level.
- `mp_stress_explosives_live`, `mp_online_failures` — CRASH (exit -9 = SIGKILL mid-test):
  the sandbox has 4 GB RAM and the kernel OOM-killer took them (mp_online_failures died 7 s in,
  after its first of 5 tests passed, mid `join_in_progress_from_the_menus_then_leave_and_reconnect`;
  mp_stress died 4 s in with no test output). Not code failures.

**origin/master moved past this branch's base** (e13d8a2c → f1791382, the level-13/level-15
merges) after the branch was cut. Merging now would invalidate the warm binary cache and force
a full rebuild of ~892 test binaries that this sandbox cannot fit before expiry; this diff
touches no shared source, so the numbers above are from this branch's tree as-is, and the merge
is left to the driver.

## Left (next session)

1. The full-suite FINAL total line (the run may outlive this sandbox; the interim line and the
   per-binary evidence above stand meanwhile, and the task's own test was verified directly).
2. One suite re-run on a tree merged with origin/master (f1791382) — needs the rebuild this
   sandbox could not fit; the warm cache makes it cheap for the next session.
3. Nothing on the task itself: both disc objectives are covered end to end on real input and
   asserted, the objective-2 state chain 1 → 3 → 4 is pinned on the walk, and no blocker remains.

## Live checks for the main agent

None — every assertion this task touches is Sim-side (mission states, channels, level_exit)
and is proven by the walk itself; nothing visual or address-only is left unverified.

## Usage

not visible to me
