## level-02-every-objective-2 (started 22:00 UTC 2026-10-29)

Task: list every stand-in, position write, health top-up or channel poke `walkreal_07000002_live`
(and the shared walker helpers it uses) still carries on 07000002, and replace each with the
real-input leg, fixing any port gap the real leg hits [M/D]. The walk must stay green. FULL suite
+ total line. Branch `s/level-02-every-objective-2`, from origin/master 531c08fb.

Progress log:
- 22:00 setup started; branch pushed with this entry.
- 22:05 setup: repo + work/ + slim ISO + queue worktree + rustup (ALSA headers needed a user prefix:
  libasound2-dev extracted to /home/z/alsa with a fixed alsa.pc prefix + a libasound.so symlink to the
  system runtime — the stock dev .pc points at /usr and the .so symlink was dangling).
- 22:45 full four-crate release test build green (target-cache ~3.6 GB; 3.7 GB disk left).
- 22:50 baseline walkreal_07000002_live green in 50.66 s / 32107 ticks. Stand-in inventory written up.
- 23:15 probe02quads: the three quads' tris + SEG/NAV + TELE — the port fires ALL THREE when Bond
  really stands on them; the pokes were harness routing (wedge / arrive-radius short), not port gaps.
- 23:20 real legs landed: no channel pokes, no bond.pos writes, top_up_health=false. Walk now FAILS
  honestly: Bond (real health) dies to the garden guards' fire during the south-hall-west nav-10 wedge
  (1836 ticks exposed) → fail restart → level_exit. The next window adds the fight-through legs.
- 23:00 fight-through legs added (clear_pursuers: real walk+fire vs the 0x15 pursuers, the 0x81
  ambush, resupply on dry clips). Root cause of the remaining failure found with NF_ALARM_TRACE:
  the nav-10 wedge is guard g3's BODY-BLOCK (d 0.8, 0x74→0x68→0x60 at t4137-4184); the fail chain
  auto-restarts ~+1462 into it. Test ignored with the exact blocker; pokes stay gone.
