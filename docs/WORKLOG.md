Plan: branch from origin/master 8de76e31; fast suite daemon started in the first minutes; read
SP_Hit/SP_Update decompile, census mask-bit-2 SPs on disc, port the bullet arm, real-input tests;
touched crates + suite; report.

### level-07-urn-break (started 08:26 UTC, 2026-10-25)

Task: placement #312 OBJECT_UrnBreak (script 0x0600007A, mode 64, mask 2) on 07000007 is a script
player that should start when shot. SP_Hit 0x1dc470 bit 2 fires on class-5 HITDATA against the
current stream objects' +0xD0 chains; the port's held_hit_targets skips hit_points-0 players so a
gunshot can never start the urn. Port the bullet arm for every mask-bit-2 SP, census the disc,
pin with real-input tests (urn shot -> state 1->2, unshot stays primed, 07000012 sky bridge, 07000006 radio).

- (08:26) Branch from origin/master 8de76e31, pushed. Suite daemon started in the background.
- (08:30-09:00) The [M] map: SP_Hit 0x1dc470's bit-2 walk (Script_GetObj kinds 1/2, the +0x37
  weapon filter, the mask==2 FilterBullets 0x106 reorder — trigger-neutral, the OR is
  position-independent); Collide_Update runs AFTER the walk (0x1336f8:238) — one-frame order.
  The [D] census (dump_sp_hit_census): exactly 3 mask-2 SPs — 07 #312 urn (0x0600007A, modes
  [64,0], hp 0), 12 #660 sky bridge (hp 120), 06 #807 radio (modes [8,1]) — NONE sets +0x37.
- (09:00-10:20) The port: sp_bullet_targets (no hp/hold gate) + add_bullet_hit (the weapon_data
  +0x00 id) + the bit-2 arm in Player::update (countdown 0, the +0x37 filter) + the pass-head
  rotation (tick_bullet_hits -> bullet_hits, the game's Collide-after-walk beat). Tests
  urn_break_live 4/4 GREEN: the urn shot -> state 1 holds on the record tick -> 2 on the next
  walk + debris; unshot stays primed 300 ticks; the radio's real fire swaps to script B
  (state 6->2, cur 0->1); the bridge's real 3-tick hold triggers (hp 120 [D]). The intact-urn
  EntityEnd-at-length semantics left UNPINNED (the port keeps SP_Delete objects; needs a live
  boot). The 07 headless fire spawns from a stale camera — the urn test injects the fired round
  (the pass/walk chain real; live check filed).
