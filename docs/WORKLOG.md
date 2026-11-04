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
- (14:35) sight_frustum_live 2/2 GREEN — census re-derived onto staying guards 158/161; 261 is
  deleted before tick 120 on the shared stream (prop 0x59 GO_TO_GOAL arrival tail [M 0x165640:47-82]).
- (14:35) hostage_killer_07000002_live 5/5 GREEN — the ch-0x19 opener pin now accepts the captor's
  silent-first 0x2E roll (no voice; step-1 advance sends to the hostage; her expected reply opens)
  [M 0x14a8b8 Rand(6) + D 0x2E.sfx=0 + M 0x141b00 + M 0x144340:10-24] — the same exception the
  repeat rule already carried, on the first turn.
- (14:35) fix_blast_scripts type-0 leg diagnosed (fix next message): the teleport spot
  (1.9717, -0.4, -2.256) is a VOID (no floor within 35u on any ray; Bond falls forever, grounded
  false); destroy[10] (#470) is a zero-extent box (lo==hi==origin, the boundless bigdoor01 leaf);
  the P2K record has spread 0.0 [L ee_snapshot weapon_data +0x24] so the shot dir is exactly
  `look` — and the test's aim_at treats pitch as radians while the game's pitch is half-turns
  [M player.rs look(): p = pitch*HALF_PI], so the shot dives -34.3 deg instead of the intended
  -21.8, passing 1.45u from the origin — outside the degenerate-box 0.1 tolerance [M
  breakable.rs seg_box_t lo==hi arm]. Needs a re-derived reachable spot (a real floor) or a
  compensated aim; geometry, not stream.

## gas-rng-draws-4 (started 2026-11-04 16:49 UTC)

Branch s/gas-rng-draws-4 cut from origin/s/gas-rng-draws-3 (b6dd3a47e), merged origin/master
(b21edd082) in. Task: finish the gas-init draws on the shared RNG stream [M 0x1aacc8:72-80] —
the report-3 list in order: enemy_accuracy's per-setup re-pin first (8 min), then the walk
re-routes (01 contact route + friendly-death stretch, 02 to_the_exit, 03 east-lane wedge +
friendly death, 14 the corridor-door chain at (30.3,35.4), 16 punch pin + defeat walk, 1b
Drake's defeat), fix_blast_scripts' type-0 leg (geometry-dead: the teleport spot is a void,
destroy[10] is a zero-extent box), fix_finale's 16 shelter pulse. Real input only, no
assertion weakened without a decompile address, helpers opt-in per level. Suite via
tools/test/suite.py (background from the start) + chunk_suite.py for the final tree. Sandbox
expires ~18:46 UTC; stop-new-work 18:26 UTC.
Round 2 (2026-11-04 17:14-18:40 UTC). enemy_accuracy re-pinned green (two 0x12 cells: d1 d5
8 was 15, d2 d5 25 was 17 [M 0x1aacc8]; binary 5/0+4i). fix_blast_scripts: aim_at pitch now
half-turns [M player.rs look()], the type-0 leg ignored with the measured geometry ledger
(point box at the origin, no floor within 56 u, P2K range 50 [L 0x2bf368]); binary 3/0+1i. 01:
the doorway lure play opened to d1 (AND fires 1974), lip/release/pre-clear gates opened, a
0x500003a far-field pre-clear added; three later reds carry fresh ledgers (7712 d1 west
corridor, 4162 d3 far pair, 8502 d2 stairwell). Not done: 02/03/14/16/1b re-routes (16 12/2+2i,
1b 15/2+2i, 03 4/2+1i fresh), the finale pulse leg (fresh ledger), the full walkreal sweep, the
chunk-suite total.
