# WORKLOG

## gas-rng-draws-2
Started 2026-11-03 05:10 UTC. Branch s/gas-rng-draws-2 cut from origin/master 6f16146e, then
origin/s/gas-rng-draws merged in (the task's base). Task: land the gas RNG draws without
breaking the level runs — prove which RNG the game's gas init draws from [M the gas init
decompile]: the shared drone stream or a separate one; if separate, give the port's gas pool
its own Rng (most of the 20 reds should clear); if shared, re-check every red (a walk that no
longer completes is a real regression; only legitimately-moved digests/counts may re-pin,
citing the decompile). Run every listed red file + goggles_render_live; all green; then the
full suite.

Work log:
- (05:10) branch start, WORKLOG entry; merging the base branch next.
- (05:12) the re-land: reverted the two reverts (2856f627 + 0ebfee9a) — the gas draws and the
  goggles fix are back; auto-merge clean.
- (05:15) the RNG provenance PROVEN: the game's gas init draws from THE SHARED STREAM —
  Rand_FRand 0x1e3780 / Rand_Rand 0x1e3738 advance the one global SEED_X/SEED_Y pair, and the
  same pair serves DroneWeap_DoBulletAccuracy 0x179740, Bullet_init 0x12c5a0 and every AI roll;
  the port's drone::Rng implements exactly that pair (18000/30903). Task option 2 applies:
  re-check the reds, no blind re-pins.
- (05:20) the red sweep: 17 failures across the 16 listed binaries + goggles (goggles 8/8
  green). Triage so far:
  - mp_perf_16 digest: re-pinned 0x89cbfcb6ae45c2ac -> 0xfce3e6fa7c122c43 with the [M] cite
    (the bots' Gas_Init sites draw from the shared stream now).
  - hold_npcs: the LAW is intact (held exactly 0.0000 at 240/600/1200/2400, re-measured); the
    unheld window 240->600 (240 landed inside a standing pause of the shifted wander phase —
    0.003 u at 240, 0.63 u by 600). GREEN.
  - sight_frustum x2: placement 261's record is present=false from the intro on — he carries
    prop 0x59 and the GO_TO_GOAL arrival tail deletes him [M 0x165640:47-82]; the new stream
    makes him finish his route during the intro. The game's own law; the test's census must be
    re-derived (the test's own note says so) — left for the next message.
  - walkreal_01/03 real-damage deaths: FRIENDLY NPCs (skin 0x5000036) kill Bond point-blank
    (d 0.6-1.0) — forced hits under 3 u are the game's law [M DroneWeap_DoBulletAccuracy
    0x179740:113-115]; the fixed walk routes get unlucky on the correct stream. Routes need
    re-tuning — left for the next message.
- (06:05) nf-game lib green; ai_live 37/0 green (the base branch's re-pins hold).

## gas-rng-draws-3
Started 2026-11-04 13:16 UTC. Branch s/gas-rng-draws-3 cut from origin/s/gas-rng-draws-2
3083a2ca, then origin/master merged in. Task: finish landing the gas RNG draws — turn the
remaining reds green (sight_frustum_live 2, walkreal_01/02/03/14/16/1b, fix_blast_scripts_live,
fix_finale_live, hostage_killer_07000002_live, enemy_accuracy_live), re-routing walks the way a
player would adapt (never forcing state), re-pinning only values the stream legitimately moves.
- (13:18) branch created, master merged clean, initial checkpoint pushed.
