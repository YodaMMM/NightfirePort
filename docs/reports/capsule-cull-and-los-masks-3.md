# capsule-cull-and-los-masks-3 — the turret muzzle offset, the standing-height fixture class, and the opponent-identity pin

## State: three of the four round-2 leftovers are green (the turret deploy groups, the armour
## take, zoe + level_end from item 4; walkreal_07000008 is thread-order-flaky — green on the
## pre-merge tree, red at the same burn gate under `--test-threads=1` after the final
## origin/master merge). Item 3 (the mp digest pin) is untouched: the method and the first
## probe are below.

## What's done

1. **The turret deploy groups are green (gt_los_live 2/2, gun_turrets_live 2/2).** Three ports:
   - `GTurret::muzzle_offset` fills at attach time: script A's static `Entity` commands resolved
     through the level's glist bounds, union `max.z × 1.5` — `GT_Create` 0x17ffa0:118-140 unions
     the `Script_GetObj` 0x1db688 records' `obj+0xd8` celglist min +0x20 / max +0x30 and stores
     `fStack_a8` (max.z) × 1.5 at ext+0xd0 [M]. Verification is EXACT: the 0700000a wall gun's
     union max.z 1.3243483 (the `GunSamuri-MuzzleFlash` glist 0x020005ab [D]; BarrelFrame
     0x020005a9 and Barrels 0x020005a8 sit lower) × 1.5 = **1.9865224 = the live ext+0xd0**
     [L ee_start_0700000a.bin, ext 0x1b9e400+0xd0]. The port's script VM gap (the create-time
     object table) is bridged by the static Entity walk; a script that creates objects later
     would union them at the game's create beat only — none on the disc's 41 turret placements
     [S that general case].
   - The per-candidate track ray starts at `t.pos + fire_dir · muzzle_offset` [M GT_Track
     0x17f9c8:232-242 — the barrel transform pos + dir · ext+0xd0; the port's stand-in keeps
     t.pos and the create-time fire dir, the script-VM gap documented at gt.rs]. Measured: the
     unpushed ray clips the mount/deck (a straight-down ray from t.pos hits 0.4% along); with
     the 1.9865224 push the wall gun's ray starts ≈0.29 above the deck, below the mount.
   - The GT fire arm now runs `Bullet_init`'s in-body spread cone — 3 draws in the game's order
     (FRand_MVar2(2·spread) → FRand(2π) → FRand(π) → Vec_Spherical_2_Cartesian, added x/z/y,
     normalised twice) [M 0x12c5a0:160-180] — the same draws every other shooter arm makes; the
     GT arm skipped them, so turret shots never spent the game's RNG stream.
   - **The test fixtures teleported Bond 1.1 below the game's stance** — the same root cause
     class across three files: the game's standing root is the floor point + 1.1 [M
     Player_StandAtNewPosition 0x190d88:0x190ee0-0x190f18], and a teleport to the bare floor
     point (or floor+0.1) leaves the ground probe no floor to find — Bond falls out of the
     world. Measured: on 0700000a's clear spot y = −569.8 after 900 ticks; on 07000009's armour
     spot y = −372.9 after the 60 settle ticks (−488 after the walk), 4.98 from the armour,
     outside its 1.577 take gate forever. All the touched teleports now stand at floor + 1.1;
     gt_los_live's blocked spot samples the muzzle ray [M 0x17f9c8:232-242]; gun_turrets_live
     parks off the mount column (directly beneath, the game's own aim geometry puts the muzzle
     0.29 over the deck and its shots die in the deck top face [M 0x180498 case 4 Mat_LookAt
     0x180498:262-263]); the ammo pin reads the run's minimum (the facing-1 gun drops its 0x52
     pickup and DELETES itself after a lethal cycle [M 0x180498:246-257] — the old fixed index
     then named the next gun's untouched 0xffff).
2. **The armour take is green (interact_07000009_live 16/16).** The take never fired because of
   the standing-height class above: the 0x721 take ray was never cast (the distance gate failed
   at 4.98 vs 1.577 — radius 0.4767 [D probe] + 1.1 [M 0x18e148:203-210]). With the stance fixed
   the take fires through the real walk-over. The sister park moved to 2.2 units out (the old
   park sits 1.47 from the centre — INSIDE the gate at the real stance, so the game itself would
   take it). The port's 0x721 semantics were re-checked against the decompile and match
   (segment_take_scoped: the 0xd/0xe skip [M 0x1ebaa8:311-313], the cel scope, no 0x40 drop).
3. **zoe_gondola_realpath_live is green (1/1) — the TEST was wrong, fixed with the address.**
   The port's team-2 opponent model is in and correct: her recorded opp_targets on this harness
   are team-1 guards #19/#20/#24, never the player [M 0x1424e0:101-121 — team 2's opponent comes
   from the drone-list walk; 0x179d40:130 re-aims per shot]. The old `perp > 2.0` proxy
   contradicted the game's own geometry: the descent's guards ambush AT Bond, so a round
   correctly aimed at a guard passes within 2.0 of his head (t82: opp (100.87, 28.25, −122.15)
   vs his head (111.02, 28.33, −127.46) — 11 apart — while the line's perp is 0.99). The test
   now pins [M 0x1424e0:101] directly: every ally round's opponent is None (the game's
   SetOpponent(0) degenerate arm [M 0x1422d0:25-29]) or a team-1 drone.
4. **level_end_live is green (6/6).** `step_at` stands at the game's stance [M 0x190d88]: the
   raw teleport left the root ~1.0 under the 07000016 arrival-walkway stance (measured: walkway
   top −0.197, teleported root −0.103); the corrected head-on cull [M Intersect_CylGeom
   0x109da0:302-322] no longer pushes a flat top crossed in the spine's top half, so the sunk
   head's use sphere fell 0.38 short of the open leaf's live sphere (dist 2.670 vs 1+r 2.292)
   and the E could never reach. At the game's stance |to| = 3.006 → dist 2.006 ≤ 2.292. The
   closing asserts are unchanged.

## How it was verified

- `cargo test --offline --release -p nf-game --lib` — 558 passed (after the muzzle/LOS/spread
  change), then the whole `nf-game` + `nf-formats` set single-threaded on the final tree: only
  the 3 `disc_golden` tests fail (the slim ISO's documented allowance — BIGF headers the slim
  extraction truncated).
- The eight touched nf-view binaries on the final tree, `--test-threads=1`: gt_los_live 2/2,
  gun_turrets_live 2/2, interact_07000009_live 16/16, level_end_live 6/6,
  zoe_gondola_realpath_live 1/1, **walkreal_07000008_live 3/4** (below), walkreal_0700000a_live
  and walkreal_0700000b_live green.
- The fast runner (`tools/test/suite.py`) was started in the background twice per the task note
  (PATH-fixed): it died at the tool-call boundary both times (the third sandbox to measure
  this). Foreground slices: the first ran the build (394 s) and its --only set (zoe and
  level_end still red then, everything else green); the final tree's slice spent its whole
  window in the post-merge rebuild (481 s of a 520 s limit — no run phase). The full-suite
  total line did not fit this window; the touched-binaries table above is the final-tree
  evidence.

## What's left

1. **walkreal_07000008_live** (3/4 on the final tree, `--test-threads=1`): the laser burn
   (`walk_the_gondola_to_the_briefcase_and_back` :164, ch 134 never written) is
   thread/order-flaky — the SAME binary was green in the earlier slice (default threads) and
   red single-threaded. The burn is the deterministic contact path (weapon 0x4E reach 4.0 [L
   ee_start_05 weapon_data] vs the bolts ~3.1 out [D]; the contact ignores world occlusion
   [M 0x1eae80:92-108]) — so the flake is the WALK's stance, not the burn: an env-gated
   `[burn]` trace at the contact site (drone.rs near :13725) printing per held tick (muzzle,
   bolt pos, distance, radius) plus `NF_WALK_TRACE` (exists) on the fusebox→safe legs will
   separate the stance stall from the burn. If the game walks those legs and burns (PCSX2),
   the port pushes a face the game does not (the push-tail gate is the suspect — level-0a-
   office-hatch-10's [L] finding); if the game also stalls, re-derive the leg.
2. **mp_net_slots_post_move_live `four_human_session_digest_is_pinned`** (:285, the ONLY red in
   its binary otherwise): tick0 matches (0x5c0bb60a8cdf8938 ✓), the final digest reads
   0xbc5e2155a3786e9c vs the pin 0xfb7b58f73e340ec3 (round-2's "only the top byte moved" note
   is wrong — the whole u64 moved, consistent with FNV-1a's avalanche: ONE component diverged
   mid-run). Method for the next window: a baseline tree at the pin's commit (a7afd219, the
   test's own commit, where the pin was measured), its own CARGO_TARGET_DIR, a scratch dump of
   the per-tick component hashes (frame_rng / bodies / vitals / combat / match / bots / …) on
   both trees, diff to the FIRST divergent component+tick, then `NF_NET_HASHDBG=first-last` on
   both to name the state byte; re-derive the pin from the game's rule once the byte is known.
   The shared RNG stream (the cull/masks change guard sight/aim timings → draw counts) is the
   prime suspect; the bodies/vitals components follow it.
3. **A full-suite total line** on the final tree (the runner needs ~75 min of run window; this
   sandbox kills background processes at the tool-call boundary, measured three times now).

## Live checks for the main agent

1. **The standing root vs the floor, one clean spot** [settles the fixture class everywhere]:
   fresh boot 07000009, walk Bond onto the body-armour floor (−22, −368.5, 24.86), stand still —
   read `obj+0x30.y` and `obj+0x80.y` and the floor top under him. Expected per the port's rule:
   obj+0x30 ≈ floor + 1.1 (−367.43), obj+0x80 ≈ floor + 0.97. If the game's root differs, the
   three fixture fixes here need the real number.
2. **The armour take ray** [round-2's check, kept]: fresh boot, walk over the armour — watch
   `Pickup_Update` 0x18e148's take branch and the 0x721 `Collide_LineOfSight` return (expected
   clear → take; the armour's obj+0x8c radius for the gate math).
3. **The wall gun's deploy reach** [round-2's check, now with the [L]-verified offset]: fresh
   boot 0700000a, raise ch 18, stand Bond on the lower floor directly under gun #636 (9.84, ~0,
   17.48) — the gun should deploy into COMBAT and shoot (the muzzle ray starts y ≈ 0.29). Note
   whether the game's rounds can wound him there at all (the port's geometry says the shots die
   in the deck when the aim is straight down).

## Usage

Token, context-window, cost and credit figures for this session: not visible to me (no such
counters are exposed in this environment; nothing to copy exactly).
