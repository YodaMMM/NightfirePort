
## mp-bot-site-scan
Started 2026-11-11 07:05 UTC. Branch s/mp-bot-site-scan cut from origin/s/mp-bot-backoff
9bf508137, merged origin/master. Task: the mp-bot-backoff report's last "What's left" item —
the site-ack chain. The port's find_opponent returns early for a bot holding a site, but the
game's scan runs unconditionally [M-dis 0x142770/0x1427b8 -> 0x1427c4]. Read the scan and the
site-ack chain in full, with addresses, and port them in the game's order. Test red on the old
code, pin through a real bot match (a site-holding bot turns to an opponent who comes into
view); MP state equal on every machine, no shared Rand draws; run every mp_*_live binary and
mp_world_sync_live, one per tool call.

Work log:
- (07:05) branch created, WORKLOG entry, checkpoint pushed.

## mp-bot-backoff
Started 2026-11-10 23:21 UTC. Branch s/mp-bot-backoff cut from origin/master 0ee8478fd.
Task: check the landed CanBackoff port (4c2203cc6, crates/nf-game/src/mp/bots.rs) against the
decompile; add the two halves it lacks — the DroneAnim_CanDoAnimState(anim 4) gate and the
behind-room NDrone2_MoveTest (-0.75 local-Z probe, cel-checked) — in the game's order
[M 0x1501a0:31-77]; pin via a real bot match; make mp_bot_objectives_live::
demolition_bots_shoot_the_site green again with the game's 4.0 too-close band; keep MP state
equal on every machine, no shared Rand draws; run every mp_*_live binary and mp_world_sync_live.

Work log:
- (23:21) branch created from origin/master 0ee8478fd, WORKLOG entry, checkpoint pushed.
  Next: fetch work-slim.7z for the decompile (work/decomp missing in sandbox), read
  NDrone2_CanBackoff 0x1501a0 in full, audit the landed port.
- (23:25-23:50) decompile sweep: NDrone2_CanBackoff 0x1501a0 disassembled in full (the EE
  MULT-writes-rd quirk recovered the dropped mflos); the two missing halves read in full —
  DroneAnim_CanDoAnimState 0x13dfc8 (DroneAnimStates[4].u16=14 → Drone_AnimTables+1512 block
  [D], char = drone+0xda) and NDrone2_MoveTest 0x155330 (range chain + AINetwork_BoundsTest
  mode 0). Bot char = the weapon's anim set [M 0x12a028:47]; bot drone mode 0 (no DIVARS+0x48
  write) → +0xc5 = 0x00 → NO speedmul: the landed "1.8 × mul" probe read the wrong rows —
  the game's probe is 0.75 straight back.
- (23:50-00:20) can_backoff rewritten in the game's order (prop 6 implicit [M 0x14d528:2] →
  anim gate over the [D] block → the lock → the 0.75 probe → nav::Network::move_test); rig
  tests reworked to the decompile (the old mul pin contradicted it); the engage-band wall case
  moved inside the probe; anim_state4_block_matches_the_disc added.
- (00:20-01:00) demolition red with the faithful probe — instrumented (fine timelines, master
  vs branch): the freeze was the sight chain, not the backoff. Two landed stand-in bugs fixed
  with addresses: opponent_visibility's bearing atan2(dx,dz) copied the game's +0x54 order into
  the port's (cos,sin) yaw convention (mirrored cone), and the site pseudo-opponent's seen test
  used a point-centre ray the site's own geometry blocks at 2 u (the game's CanSeeObject(obj)
  is surface-aware). find_opponent's site-hold early return investigated and left as landed
  (the scan-skip deviation documented, M-dis cited in the report). All 10 mp_bot_objectives_live
  green again; mp_perf_16's PINNED_DIGEST re-pinned (39 kills) with the reason on the constant.
- (01:00-01:10) the mp battery + rig tests run (47 rig + 45 binaries green; mp_stress_explosives
  and mp_online_failures SIGKILL'd by the sandbox memory both ways); suite started on the final
  tree, killed early by the window — re-run before merge. Report written, branch pushed.
- (continued) merged current origin/master: master's reviewed CanBackoff (1.8 x mul, the
  0x1249e8:98/0x14c600/0x14c938-case-0x10 chain) taken in every conflicted hunk; the auto-merge
  had left the branch's old 0.75 in the shared `let to` block — fixed to master's `* reach`.
  nf-game --lib 794/0/4 on the merged tree.
- (final) the battery: every mp_*_live binary one per tool call on the merged tree — 40 nf-view
  binaries (159 tests) + nf-game mp_arenas_live 1 + mp_map_objects_live 4, all green; the site
  pin demolition_site_holder_turns_to_an_opponent passes; mp_perf_16 re-pinned once for the
  merged tree (42 kills, 0x4b0f0d0e0dbee72b). mp_stress_explosives_live not attempted (sandbox
  memory SIGKILL precedent from the base branch and the last two sessions).
