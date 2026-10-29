# level-01-every-objective-2 — land the 07000001 objective walk

Branch `s/level-01-every-objective-2` = `origin/s/level-01-every-objective` (f468735c) merged with
`origin/master` (1b2309ea, batch-1029a). One test change; no game-code changes.

## What was done

**The 0x47 assert demoted.** `walkreal_07000001_live` no longer asserts
`!channels.on(0x47)` ("07000001 never raises the obj0 completion channel — no disc writer on this
level"). An assertion may not rest on a missing writer — the port may simply lack it — so it is now
an `// [S] unverified` eprintln log, kept out of pass/fail (crates/nf-view/tests/walkreal_07000001_live.rs,
the obj0 block after the walk). Every other assert kept: obj0 announced-and-active (states01[m01][0]==3),
07000002's three objectives still waiting on their enable channels, mission-success ch 100 off — and on
the 02 walk the cross-level fold stays: `walkreal_07000002_live` still asserts 07000001's escort
objective folds (states01[m01][0]==4) when 07000002's intro cut-scene stream 7 writes ch 0x47
[D 07800002 0x06000084] through the real monitor [M 0x1827d8 case 3].

**What was searched for the 0x47 writer** (recorded here per the task):

1. **07000001 level data** [D census, walkreal_probe01]: every placement, trigger, mux, lock, script
   and pickup scanned — the only script-0xD writes on this level are **0xB and 0xE**; no
   trigger/mux/lock/pickup carries 0x47.
2. **ACTION.ELF code, disassembly census** [M-dis work/elf/ACTION.ELF]: every `switch_channels`
   (0x26fc90) store (lui 0x27 / -0x370 base idiom, tracked through addiu/addu to sb/sh/sw) — the only
   immediately-written channels are **0x2d, 0x5e, 0x62, 0x63, 0x64, 0x6f, 0x97, 0x98, 0x9b, 0xfd**.
   Every other writer (Trigger/SP/Script_EventHandler/DroneFunc_HostageSaved/Pickup/Door/…) takes its
   channel from disc data.
3. **Decompile callers of switch_channels** [M work/decomp/ACTION.ELF]: Script_EventHandler (0x1d94c0),
   FuseBox_Update (0x1adfe8), SSys_Monitor (0x1ad998), P_CHEATMEDAL_Handler (0x2086b0),
   Sensor_Update (0x1af618), DroneFunc_HostageSaved (0x1441c0), Door_IsLocked (0x136810),
   MP_CheckForEndCondition (0x184268), Player_HandlePain (0x1902c8), Sub_DeployTorpedoes (0x1acf68),
   control_handle_cel_change (0x133288), Player_Move (0x191dd0) — none is a 07000001 story path to 0x47.
4. **Port source, this session** [M]: `channels.set` across crates/nf-game/src — no production writer of
   0x47. The only 0x47 `set` (mission.rs:515) is inside a `#[test]` (cold-init test setup), not a game
   path — which is exactly why the assert had to go: today the port simply lacks the writer.
5. **The real writer is the next level's story hand-off**: 07000002's level-start cut-scene, SP #400,
   script 0x06000084 (pack 07800002, mode 1), stream 7 writes ch 0x47 = 1 [D 07800002 0x06000084 stream 7]
   — the same one-part-late shape as 0x4D (07000003's TouchOnce #308 writing 07000002's obj2).

The [S] expectation itself ("07000001 never raises ch 0x47 during the level") stays recorded as the
log line, with both branches spelled out, out of pass/fail.

## How it was verified

- `walkreal_07000001_live` on the final merged tree: **4 passed / 0 failed (27.5s)** — solo cargo run,
  real input path, NF_ISO on the slim ISO.
- `walkreal_07000002_live`: **1 passed / 0 failed (52.2s)** — the cross-level fold pinned live.
- Both binaries also ran again inside the full-suite chunks below (same tree) with the same results.

## FULL suite

Sequential chunks of `tools/test/suite.py` (per-binary cap 260s, --test-threads=1) on the final tree
(merged with origin/master). This sandbox kills background jobs between tool calls (verified twice:
nohup'ed runner died within 60s), so the "background in the first minutes" path is not available here;
chunks ran in the foreground instead.

**Total line: `3454 passed, 20 failed, 213 ignored`** across **876 / 894** test binaries.

All 20 failures are environmental (sandbox: 2 cores, 3 GB RAM, no GPU, slim ISO):

- 3 `disc_golden` — the allowed slim-ISO failures (retail-only golden manifest).
- `identity_disc` — slim ISO sector count (518,435 vs retail 1,191,072); volume id/serial/vmode/version match.
- `probe_pylon_render`, `fix_0c_pylon_live` ×2 — wgpu `adapter: NotFound` (no GPU).
- 11 exit -9 SIGKILL OOM CRASHes: `mp_death_hands_online`, `mp_online_failures`, `mp_remote_fx_live`,
  `mp_shot_effects_live`, `mp_stress_explosives_live`, `net_desync_report_live`, `net_join_live`,
  `net_match_end_clients_live`, `net_resync_live`, `net_soak_faults_live`, `oddjob_hat_live`.
- 2 TIMEOUTs at the 260s runner cap: `campaign_flow_live`, `walkreal_07000003_live` — both known to
  need ~8 min; campaign_flow passed solo at limit 570 in the previous session (5 passed).

Not run (18 binaries, all slow walkreal_* live tests, each minutes long): walkreal_07000013_live,
walkreal_07000014_live, walkreal_07000015_live, walkreal_07000016_live, walkreal_0700001b_live,
walkreal_07000041_live, walkreal_0700004a_live, walkreal_0700004a_satchel_live, walkreal_celprobe_live,
walkreal_dump_live, walkreal_probe0c2_live, walkreal_probe0c3_live, walkreal_probe0c_door_live,
walkreal_probe0c_live, walkreal_probe11_48_live, walkreal_probe2_live, walkreal_probe3_live,
walkreal_probe81_live. None touched by this task; the task's own two binaries are inside the total.

## What's left

- The 18 unrun walkreal_* binaries above (next session's window; nothing in this task depends on them).
- The 13 environmental failures to re-check on hardware with more RAM/GPU and a retail ISO.
- `net_soak_faults_live` remains unverified beyond its OOM (unchanged from the previous session).

## Live checks for the main agent

None required for merging (no visual work). Optional, to settle the [S]: during a real 07000001
playthrough in PCSX2, watch the Drones channel store the mission monitor reads (the game-side channel
table behind `MissionData` row ch 0x47, text 0x04000006) for any write of 0x47 — expected: never rises
before the level exit; the port currently has no writer, matching the census above.

## Usage

not visible to me
