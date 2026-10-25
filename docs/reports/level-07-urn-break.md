# level-07-urn-break — shooting the urn starts its break script

Branch `s/level-07-urn-break` (from origin/master 8de76e31; tips 5a825a0a → 6641b3c4 → 395ebdd9).

## What was done

**The port (crates/nf-game/src/nis.rs, drone.rs, useobj.rs).** `SP_Hit` 0x1dc470's bullet arm is
in:

1. `Scripts::sp_bullet_targets` — the bit-2 read set: the CURRENT script's stream objects of every
   script player with a loaded script (`Script_GetObj` 0x1db688 walks the kind-1/2 records), with
   NO hit-point or hold gate — the game fills every object's `+0xD0` chain and `SP_Hit` reads it
   while the countdown is 0 [M 0x1dc470:15-87, 0x1dbda8:66]. The old `held_hit_targets` (hp ≠ 0 +
   held only) stays for the damage tail — that gate is why the urn (hp 0) could never react.
2. The bullet pass (`update_player_bullets`) records each player-bullet contact per player
   (`add_bullet_hit`, the weapon id = `weapon_data+0x00` u16 [M; docs/fields/weapon_data.md]) into
   `Player.tick_bullet_hits`.
3. The pass-head rotation `tick_bullet_hits → bullet_hits` models the game's one-frame order:
   `Collide_Update` runs AFTER the control walk [M `control_movement_object_handler` 0x1336f8:238],
   so a contact recorded at tick N is read by `SP_Hit` at tick N+1.
4. `Player::update`'s trigger block ORs bit 2 when the countdown is 0 and a contact is in the read
   set, filtered by `+0x37` (placement param 8): a set filter only arms for that weapon's bullets
   [M 0x1dc470:64-70]. The mask == 2 special case's `Collide_FilterBullets(obj+0xD0, 0x106)`
   [M :31-34] reorders the chain and stamps `+0xC` = 1e8, but `SP_Hit`'s own walk visits every
   node either way [M :36-76] — trigger-neutral; documented, no port analog (no per-object chains).

**The census [D]** (`crates/nf-view/examples/dump_sp_hit_census.rs`): exactly three mask-bit-2
script players ship on the disc — 07000007 #312 (OBJECT_UrnBreak, script 0x0600007A, modes [64, 0],
stay 1, out 0, hp 0), 07000012 #660 (the sky bridge, 0x06000530, modes [64, 0], **hp 120**) and
07000006 #807 (the radio, 0x06000055+0x06000056, modes [8, 1]). **None sets `+0x37`** — the filter
arm has no disc user; it is pinned from the decompile by a unit test. `dump_sp_script.rs` dumps any
level script; the urn's: stream 9 = the INTACT urn (glist 0x020004EE, model sphere r 0.911, lives
t0→t23), stream 11 a 1-tick piece, streams 1-8 the debris (stream 1 = 0x020004EA), stream 12 the
crash sound — the mode-64 prime spawns exactly the Wait(0) streams (9/11), which are the shot
targets.

**The tests** (`crates/nf-view/tests/urn_break_live.rs`, 4/4 green; unit test in nis.rs):

- `shooting_the_urn_starts_its_break_script` — a P2K round through the intact urn's
  `Control_BuildWorldSph` sphere records on BOTH stream objects at tick N with the SP **still
  state 1** (the one-frame order [M 0x1336f8:238]); the next walk consumes it → **state 2**
  [M SP_Update case 1] and the debris spawns. The 07 headless harness has no first-person camera
  feed at Bond's spawn (the round spawns from a stale camera — see Live checks), so this test
  hands the pass the fired `PlayerBullet` directly; the pass, the chain rotation and the SP walk
  are the real path. The radio and bridge tests fire Bond's REAL trigger.
- `an_unshot_urn_stays_primed` — 300 ticks: state 1, no contacts, no debris, the intact urn alive.
- `shooting_the_radio_swaps_it_to_its_b_script` — real fire beside the radio: state 6 → 2, cur
  0 → 1 (script A freed, script B 0x06000056 plays) [M SP_Update case 6].
- `shooting_the_sky_bridge_starts_its_break_script` — hp 120 [D]; the primed stream-0 object is a
  class-0 entity with NO glist record [D script dump] — the game's no-record sphere is the
  translation with radius 0.1 [M 0x133070:19-27, :61-66]; Bond's real 3-tick trigger hold starts
  the break (state 1 → 2).
- `the_037_bullet_filter_arms_only_the_matching_weapon` — a weapon-2 bullet does not arm a
  filter-5 SP; the matching weapon's does [M 0x1dc470:64-70].

## Verification

- `cargo test -p nf-view --test urn_break_live` → 4 passed, 0 failed (before the suite restart;
  a later rerun under the suite's concurrent build was SIGKILLed by memory, not a failure).
- `cargo test -p nf-game --lib` on the final tree → **548 passed, 0 failed**, 3 ignored.
- `cargo test -p nf-view --test walk_07000007_live` → 2 passed, 0 failed (the urn/radio/bridge
  changes must not move the party walk).
- The full suite (`tools/test/suite.py`) was started in the background in the first minutes on
  the base tree; it was restarted on the final tree at 09:35 UTC and had reached ~30 binaries by
  09:53 (2 CRASH = the OOM SIGKILL family from the concurrent builds; `exit -9`, the documented
  environmental family). **The 90-minute full run cannot fit the remaining window — the final
  tree's total line is NOT available; the touched crates above ran green on the final tree.**

## What's left

- The full suite on the final tree (the next session: `NF_ISO=… nohup python3 tools/test/suite.py &`).
- Live checks for the main agent (below).

## Live checks for the main agent

1. **07000007, fresh boot, the urn at (49.26, 43.01, −53.46) [D #312]**: shoot it with the P2K —
   the break script should start on the NEXT frame after the bullet's contact frame (state 1 → 2,
   `SP+0x24` at obj 0x…312's SP; watch `tick_bullet_hits`'s game analog: the class-5 record lands
   on the stream objects' `+0xD0` chains during Collide). Also confirm the intact urn art
   VANISHES at the break's end (stream 9's EntityEnd sits at the script's last tick — the port's
   SP_Delete keeps the objects; whether the game runs that EntityEnd before the length stop is
   unpinned).
2. **07000007 headless fire camera**: why does a test-spawned round on 07 leave from a stale
   camera position while 05/06/12 fire from Bond's eye? `Sim::run`'s camera is
   `self.bond.head()` [M scene.rs], so the 07-specific divergence is worth one look — the urn
   test currently injects the round.
3. **07000012 sky bridge reachability**: the primed SP's only shot target is a 0.1-radius
   no-glist ball at ≈(0.015, −0.415, −0.045) world [M the BuildWorldSph no-record path; D the
   script dump]. Confirm in the game what actually breaks the bridge in normal play (a bullet
   there is unreachable) — likely the blast proxy path (the destroyable → `Explode_Create`) or a
   trigger; the SP's reaction to a bullet ON that ball is what the port pins.

## Usage

Input tokens: not visible to me. Output tokens: not visible to me. Context used: not visible to
me. Cost: not visible to me. (No token/credit figures are shown in this environment; the sandbox
clock read 09:53 UTC at the report.)
