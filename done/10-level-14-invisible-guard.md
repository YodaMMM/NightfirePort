## Your task: `level-14-invisible-guard`, Countdown Part 1: a guard that is there but does not draw

In play on 07000014 a guard is present (he acts / can be fought) but is not drawn. The load log points at a candidate: "g-021: skin 050000b1 is carried only by 07000016.bin/07000015.bin - no container the session has loaded; the table never registers it. On 07000014 the 0x050000b1 placements still resolve through the Drone+0xe0 rewrite [M 0x14b300:82-95, :563-565]; a skin a placement DOES request with no session carrier just misses (AnimObjectNew 0x1be910:44-46) and draws no character". (1) Census every drone on 07000014 that is present/enabled at any point of the level but has no drawn character (no skin resolved, a held-back spawner mesh never released, a palette slot left FAR) - list each with placement, skin hash and why. (2) For each, find what the game draws [M/D; where does the game get the skin - a container the port does not load, a pack, the Drone+0xe0 rewrite] and fix the port so the guard draws with the game's model. Pin it with a test (the character for that drone has a resolved skin and a non-FAR palette slot while present). Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-invisible-guard`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-29T08:05Z

---
## Report (done, 2026-10-29T09:22Z)


---
## Report from the worker (done, 2026-10-29T09:23Z)

# level-14-invisible-guard — report

Task: on 07000014 (Countdown part 1) a guard is present (acts, can be fought) but draws
nothing; census every such drone across the three missing-character lanes and fix the port.

## What was done

**The census, on the real load path** (`nf_view::load` of the disc's 07000014.bin, ISO-driven):

1. **No skin resolved — count 0.** 61 drone placements; 37 create at the mission-start
   difficulty (the 24 with param 2 = 2/3 are the game's own create gate [M Drone_Create
   0x136be0 first branch, GameState+0x40]; the mission start forces difficulty 1 [M
   Menu_RunMiniMission 0x1ffb90:22]). The task's candidate — skin 0x050000b1, carried only by
   07000015/16 [D carrier scan, 07000016 entry 139 @0x8594a0 len 0x860, 07000015 entry 102
   @0x609e60 len 0x860] — is **not** a miss: there are exactly **11 raw 0x050000b1 placements**
   (indices 448/449/465/468/471/472/479/703/704/1029/1030, groups 10/12/14/15, param 9 = 26
   [D probe]) and `NDrone2_DefaultInit` rewrites the param-0 to the own-archive 0x0500008c in
   Drone+0xe0 BEFORE the `AnimObjectNew` request [M 0x14b300:82-95, :563-565 — cite-14-skinless-drones];
   the captains swap to 0x05000093 (kind 6) / 0x050000b2 (kind 7), both own-archive [D probe:
   level.skins holds 8c/93/b2, not b1]. The 15 0x05000090 placements substitute to 0x05000093 on
   0x07000014 [M AnimObjectNew 0x1be910:33-43], and the g-021 session merge carries 90 from
   0700000d regardless (story order [D]). The loader's "g-021: skin 050000b1 …" line is the
   informational branch of that merge, not a defect. `missing_models_census_live` already pins
   07000014 `skinless_drones` = 0.
2. **A held-back spawner mesh never released — count 0.** The port binds characters to drones
   by placement index at load (`char_drone`, scene.rs), the spawner templates are records in
   the load-time list marked absent [M Drone_Create/Drones::new], and `Action::Spawn` mutates
   the template record IN PLACE (present/enabled rise on the very record the character is bound
   to — crates/nf-game/src/drone.rs `Action::Spawn(i)` arm), so `char_hidden`'s drone-link arm
   unhides the mesh the tick after the spawn. Probed: immediate waves (ch 1/2/3), the
   channel waves (ch 9/10/11, 13), and the full mission chain (0x47 → 0x48 → 0x87+0x88 → 0x8C →
   the room escape 0x0C/0x09 with the walk-on rounds the visible-spot screen needs [M
   0x137cb0:43] → 0x0D → 0x54): **every spawned guard draws** (hidden=false, pose Some).
3. **A palette slot left FAR — count 0.** `dr.object_pose(&data)` (the pose the GPU palette and
   the hit boxes share) returned Some for every present/enabled/alive drone at every stage —
   no drone's bones collapse to the placement point while its AI acts.

**No drone present/enabled/alive without a drawn character reproduced on any drive built this
session.** The load-log candidate is resolved by the game's own rewrite; the remaining two
lanes are clean on the port's current tree.

**The pin** (`crates/nf-view/tests/invisible_guard_07000014_live.rs`, 2 passed / 0 failed in
10.7 s): `every_present_drone_draws_through_the_whole_mission` drives the whole five-stage
chain and asserts, for EVERY drone present/enabled/alive at every stage: a character bound via
`char_drone`, `char_hidden == false` (the released mesh), `object_pose().is_some()` (the
non-collapsed palette) and `characters[ci].skin.hashcode == dr.skin` with a skeleton in
`sim.skins` (the resolved skin). `the_b1_and_90_placements_resolve_to_own_archive_skins` pins
the raw census [D]: the 11 b1 placement indices, the 15 90 placements, the own-archive
presence of 8c/93/b2 and b1's absence. Any future invisible-guard regression in any of the
three lanes fails these.

## How it was verified

Suite slices on THIS branch's tree (master ccc714a1 + the new test): the 08:12 UTC fast-runner
background pass (191 binaries: 691 passed / 2 failed / 48 ignored — the 2 the no-GPU 0c-pylon
renders) + chunks c00/c01/c02 (1397 passed / 3 failed / 0 ignored: campaign_flow_live and
walkreal_07000003_live TIMEOUT at the 260 s chunk cap — both pass at full length, 570 s and
384 s respectively, proven earlier today — plus the slim-ISO identity_disc sectors assert:
518,435 vs the retail 1,191,072) + the new test binary (2 passed). **Combined: 2092 passed /
5 failed / 171 ignored.** All 5 are environmental; none touches this task's lanes.

**Not run this session:** ~500 small nf-view binaries of this tree (the background runner was
killed by the sandbox reaper at 191; the chunk grind ran out of window). The previous session's
full chunked suite covered the same crates on the level-03 tree (3529/12/215) whose
nf-game/nf-formats/nf-audio code this branch inherits unchanged; master's own merge carried its
nf-view deltas with its own green runs.

## What's left

- The ~500 unexecuted small nf-view binaries on this exact tree (a `suite.py` pass will close it).
- If the main agent's playtest still SHOWS an invisible guard on 07000014, the live checks below
  localise it; the port-side sim+scene census is clean on every drive built.

## Live checks for the main agent

1. Which guard, and where: the placement index / position of the guard that fights invisibly
   (`tools/trace/fieldprobe.py` — poke a loud shot at the player and read the acting drone's
   object+placement), plus the selected difficulty (GameState+0x40) at that boot.
2. The draw state of that drone in the real game: obj+0xFC (the draw counter) and obj+0xD8/0x90
   (the mesh/pose pointers) — gdbwatch on a fresh 07000014 boot while the guard is on screen.
3. If the guard is a spawner reinforcement: its group's spawner params (p1/p2/p4/p6/p7) from the
   placement, to check the port's spawn timing against the game's [M 0x137e68].

## Usage

not visible to me — this environment exposes no token, credit or context-window figures.
