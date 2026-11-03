# Worklog

## reload-and-bond-hitboxes

- (10:24 UTC, 10-24) Branch created from origin/master (26543436). Task: verify the guards' reload animation (DroneWeap_* reload path, anim id, clip refill tick) and Bond's 16 hit boxes (player hit volumes, what a guard's round hits on Bond). Sandbox reused: repo, work/, Nightfire-onfoot.iso present. Setup next: suite in background, then trace the reload path and the player hitdata setup in the decompile.
- (11:30 UTC, 10-24) Reload path verified [M 0x1666d8/0x167370/0x15ff80/0x16a8b0 + disasm]: AimStandReload ENTER draws Rand_Rand(30) unconditionally (the port consumed NO draw — a determinism bug; the 0x4a alternate is gated by CanDoAnimState and dead for 07000005's guard sets, pinned), AlertStatusSet(1) added to stand + undercover (the crouch case has neither, pinned both ways); refill = +0xBBC = +0xBBE at ENTER (stand/crouch/undercover) and EXIT (sniper) — the port matched, now pinned. Bond's 16 hit boxes ported: HumanCollisionBones 0x2c6be8 [D] via AnimGetCollData 0x1c4ac8 + Collide_Jointy 0x1ec180 kind 0x201 [M], the [L] standing idle palette (three level-start snapshots agree ±0.011) riding Bond's yaw; the cylinder deviation is gone from the guard-round path; BulletEvent::Player carries HITDATA+0x52. SP damage has no per-part scale [M 0x1902c8:28-49] — pinned. Six new tests green (reload_and_bond_hitboxes_live); nf-game lib 546, ai_live 36, the copter/walkreal/bullet neighbours green. Report: docs/reports/reload-and-bond-hitboxes.md. Left: the anim-driven palette (the crouch pose), the astronaut table test, the MP multipliers.

## bond-hitbox-pose

- (11:38 UTC, 10-26) Branch created from origin/s/reload-and-bond-hitboxes (14eef65e). Task: make
  Bond's 16 hit boxes available on every level (the skin 0x05000007 table gap — no boxes = an
  unhittable Bond on levels without the skin), pose them by Bond's state (crouch vs stand), then
  the full suite with fix_bond_damage_live and anim_events_live green. Sandbox reused: repo, work/,
  Nightfire-onfoot.iso, target/ present.

## bond-hitbox-pose (session 2, continuation)

- (14:15 UTC, 10-26) Sandbox had reset; full setup redone (repo, work-slim, ISO, rustup 1.99.1,
  ALSA user prefix, queue worktree). Verified the three prior commits by running them:
  fix_bond_damage_live 7/7 (crouch-vs-stand round test in), anim_events_live 4+1ign (melee
  connects), bond_boxes_every_level_live 1/1 (16 boxes every story level).
- Suite chunks (foreground; background dies here) flagged mp_perf_16_live's digest: bisected over
  the branch with a throwaway worktree — breaks exactly at the crouch-pose commit (5da2b3aa);
  cause is the deliberate [L]-backed crouch root drop (0.4847) meeting the run's scripted crouch
  presses (i.crouch = ph % 307 == 0). Re-pinned 0xb270_68a8_2582_fec6 (kills 39), documented in
  the test; commit abb44f33.
- Triage: fix_0c_pylon_live + probe_pylon_render = no-GPU sandbox (wgpu adapter NotFound);
  fix_pickups_trace_live + walkreal_07000014_live fail identically at merge base 051c5240
  (verified by base-tree runs) — pre-existing, not this branch's.
- Coverage: 187/823 suite binaries run this session (0 new failures); the rest is the next
  session's first move. Report: docs/reports/bond-hitbox-pose.md.

## bond-hitbox-pose-2 (started 14:03 UTC 2026-10-29)

Task: the -pose branch's crouch toggle moves bond.pos[1] down 0.4847 (ROOT_HEIGHT −
CROUCH_ROOT_HEIGHT) INSTANTLY on top of the eased crouch_drop (0..0.45) head() already
subtracts — the camera reads head(), so a crouch drops the view ~0.93 in one tick instead of
master's eased 0.45. Keep the view EXACTLY as master (head() per tick), keep the crouched
hit-box posing, keep fix_bond_damage_live / anim_events_live / bond_boxes_every_level_live
green, and the mp_perf_16 digest pin must not need another re-pin. Add a head()-per-tick pin
test against master's values.

Plan: branch from origin/s/bond-hitbox-pose (f9bb4420) + origin/master (1b2309ea) merged
(82ccda59; bullet.rs conflict resolved: master's segment_bullet world trace [M 0x12e7b0] +
the -pose box walk; two rope_guard Player initializers gained crouched: false); build green.
Next: read the -pose crouch diff (5da2b3aa), pick the fix shape (root offset → bond_boxes
param, or the head() compensation fold), write the head() pin test, run the four named
binaries + the chunked full suite.

Progress (15:22 UTC): the fix landed — the toggle no longer moves pos (the -pose drop
removed), bond_boxes shifts its root-local frame by 0.4847 [L 1.086/0.607] so the boxes stay
crouched, head() is master's line unchanged. PROOF the view is master's: an identical probe
(flat floor, crouch → 70-tick hold → stand-up) run on a master 1b2309ea worktree and on this
tree produced BIT-IDENTICAL per-tick pos/cdrop/head/cc streams (111 samples ×2), and the
committed pin test pins those 110 per-tick values as f32-bit literals (==, no epsilon).
Greens: the pin test, fix_bond_damage_live 7/0 (the crouch round still misses the crouched
head — the boxes ride the palette offset), anim_events_live 4+1ign, bond_boxes_every_level_live
1/0, mp_perf_16_live 2/0 at the master-lineage digest 0x89cbfcb6ae45c2ac (the -pose 0xb270
encoded the reverted root drop; reverting it is not a view-only re-pin — the sim returns to
master's). Full suite: did not fit this window (setup + the two-worktree probe capture);
the next message resumes with the chunked runner on a fresh state file.
