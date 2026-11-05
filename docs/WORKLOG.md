# WORKLOG

## regression-read-weapons (branch s/regression-read-weapons)

Started 2026-11-05 ~18:07 UTC. Task: read every commit since 2026-10-20 touching
crates/nf-game/src/weapon*.rs crates/nf-game/src/bullet*.rs crates/nf-game/src/grenade.rs;
for each, diff -> what a player would see -> check vs game [M]; prove suspects through the
real App path (HeadlessApp keys, 1/240 and 1/60); fix what is proven with a failing-first
test. Not casings, driving or gamepad. Name breaking commits and their first tag. Report
every commit range read and what was clean.

Progress log:
- Sandbox reused from previous task (repo, work/, ISO, nf-queue all present).
- Branch s/regression-read-weapons created from origin/master (2f0c88e2c) and pushed.
- Commit census complete: 30 non-merge commits since 2026-10-20 touch the census files
  (weapon*.rs does not exist; the weapon logic is gun.rs — census = gun.rs, bullet.rs,
  grenade.rs, hurt.rs for the damage half of the title) + 7 merges, whose deltas map 1:1
  onto the audited branch commits. Every diff read; each commit's player-visible claim
  checked vs the decomp/disasm ([M] with function+address, [L]/[D] for the data): all 30
  verified, no suspect. Highlights verified in the game binary itself:
  Bullet_init's unconditional 3-draw cone (0x12c7f4), the glass-pass arm record+0x6C bit
  0x800 -> HITTEST mask 0x20 (disasm 0x12f048-0x12f058), Bullet_handle_object_destruction's
  +0x68&0x4000/+0x70&0x80204 gate (0x12cd98), DoTrails' burn-out Rand_Rand(0x1518)+0x3c
  (0x12d890:424), Player_WeaponFiring's run-out return + state-9 refire gate
  (0x1a5460:601-648), Player_HandlePain's +0x968 byte glow (0x1902c8:126-130).
- Positive controls on the real path (all green): nf-game lib 679/0/5;
  weapon_handling_live 3/0; tripwire_shot_live 4/0; satchel_laser_scramble_4a_live 1/0;
  laser_watch_0c_live 3/0; bullet_range_live 3/0; fix_foliage_bullets_live 2/0;
  bullet_range_census 1/0; fix_06_fire_live 3/0; drone_weapon_drop_live 3/0.
- Full suite in progress; report next.
