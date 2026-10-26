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
