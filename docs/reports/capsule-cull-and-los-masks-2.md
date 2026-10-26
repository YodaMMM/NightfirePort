# capsule-cull-and-los-masks-2 — the head-on cull and the per-caller LOS masks, round 2

## State: resumed the round-1 branch, merged origin/master, diagnosed every remaining group;
## the GT_Track ray-end fix landed; the turret deploy groups are blocked on the game's
## ext+0xd0 muzzle offset (the port's script-VM gap); the other five groups are reproduced
## with their causes localized but not fixed in this window.

## What's done

1. **The final tree exists and compiles.** `s/capsule-cull-and-los-masks-2` = origin/s/capsule-cull-and-los-masks
   (f0cda139) + origin/master, with the three merge collisions resolved in the cull's favour:
   the branch's corrected unconditional head-on cull arms `(nd > 0.999 ∧ f > 0.5) ∨ (nd < −0.999 ∧ f < 0.5)`
   [M Intersect_CylGeom 0x109da0:302-322] and the unconditional per-caller LOS masks
   [M Collide_Intersect 0x1ebaa8:300-319, Collide_Pick 0x1eb320:140-152] are kept; master's
   `LOS_TURRET_TRACK = 0x72f` const is kept (gt_los_live and the drone turret track read it); the
   master-side scratch gates the branch had deleted (`CAPS_GATE`, `caps_log`, the ref=bottom push-tail
   probe) are deleted in the merged tree too — the merge had auto-kept their uses after their
   definitions were gone.
2. **GT_Track's per-candidate LOS ray now ends at the candidate's obj+0x80 world-sphere centre**
   (`crates/nf-game/src/gt.rs` `TrackCand.sph`, `crates/nf-game/src/drone.rs` candidate fill):
   the decompile loads the ray end from `*piVar6 + 0x80` [M GT_Track 0x17f9c8:271-278], which is the
   object's world sphere [M/`docs/fields/obj-tag-player.md` +0x80..0x8C, Control_BuildWorldSph
   0x133070] — NOT the obj+0x30 feet the range gates read [M 0x17f9c8:99-118]. The port used the feet
   for both. Bond's candidate carries `sphere_centre()`; the guards keep the feet as the stand-in
   [S — the port's drone records build no obj+0x80]. The t-window is inclusive 0..1
   [M Intersect_RayGeom 0x10a8c8:171-172], so an exact endpoint touch counts — the distinction matters
   because a standing candidate's sphere centre rides at the deck top while the feet sit exactly on it.
   nf-game lib: 556/556 green.
3. **The turret deploy failures are diagnosed to the game's muzzle offset.** The 0x72f track ray's
   START is `barrel-transform pos + fire_dir · (ext+0xd0)` [M GT_Track 0x17f9c8:230-258], with
   `ext+0xd0` = the script-A union model-bbox max.z × 1.5 [M GT_Create 0x17ffa0:140] — and
   **[L] ee_start_0700000a.bin** has the live gun (ext 0x1b9e400: param4=18, facing=1, hp=0xffff,
   health=100.0) at **ext+0xd0 = 1.9865224**: the game's ray starts ≈0.29 above the deck, below the
   2.28 mount. The port's `GTurret::muzzle_offset` field exists (the struct comment already maps it to
   ext+0xd0 [M 0017ffa0:155-158]) but stays 0.0 because the port's script VM cannot enumerate the
   script objects' models — the tracked gap. Measured port-side consequence: the ray from t.pos (no
   push) to a candidate under the mount hits the deck's top face at the ray's endpoint
   (fraction 1.0, front-face, not dropped by 0x72f's bit 0x8), so the gun never arms.
4. **Reproduced every listed group on the final tree** (`NF_ISO` = the 1,061,754,880-byte
   Nightfire-onfoot.iso, `cargo test --offline --release`, test-threads default):
   - **bathhouse_shout_mixer_live — GREEN already** (2/2): nothing to do.
   - **gt_los_live 1/2, gun_turrets_live 1/2** — the deploy arms fail per item 3; the blocked-spot
     arms stay green (a wall-blocked candidate still holds the gun DORMANT).
   - **mp_net_slots_post_move_live 1/2** — `four_human_session_digest_is_pinned` (:285): the measured
     final digest 0xbc5e2155a3786e9c vs the pin 0xfb5e2155a3786e9c — ONLY the top byte moved, the low
     56 bits are identical, i.e. exactly one byte of the digested state changed with the masks/cull
     on; the digest's field map is the next window's first read.
   - **zoe_gondola_realpath_live 0/1** — `zoe_rides_the_carriage_and_never_targets_bond` (:248);
     consistent with round 1's ally-ray class (an aim/vision ray that now drops a 0x40-surface face
     shifts the scenario's RNG stream) but not yet instrumented.
   - **interact_07000009_live 15/16** — still the body-armour take (`:329`), Bond placed on the right
     floor, take never fires; the round-1 plan (print Bond's path + the `Pickup_Update` 0x18e148
     distance gate and the `Collide_LineOfSight(…, 0x721)` take ray) is unchanged.
   - **level_end_live 5/6** (`:270`, the 07000016 straddle door) and **walkreal_07000008_live 3/4**
     (`:164`, the gondola walk) — reproduced, not yet instrumented this window.
5. **The fast runner** (`tools/test/suite.py`, merged from master) was started per the task note and
   died with the sandbox's tool-call boundary (background processes do not survive here — measured
   twice, the nohup'd process vanishes between calls). Foreground chunks: the first 570 s slice ran
   **104 binaries, 0 failures** (the slice does not reach the nf-view failing groups); a full pass
   needs ~75 min at the measured rate and does not fit this window.

## How it was verified

- `cargo test --offline --release -p nf-game --lib` — 556 passed (the merged tree + the TrackCand change).
- The eight listed nf-view binaries, individually (numbers above).
- The suite runner's first foreground slice — 104 binaries, 0 FAILED/TIMEOUT (see its log lines in
  the report's suite transcript below; the run was killed by the 570 s chunk, not by a test).
- The [L] ext+0xd0 read: `work/ee_start_0700000a.bin` (file offset = EE address), object at
  0x1b9e2f0 (obj+0x30 = the placement translation 9.835809, 2.276048, 17.478699 [D placements dump]),
  obj+0xe0 → ext 0x1b9e400, ext+0xd0 = 1.9865224 f32.

## What's left

1. **The turret deploy groups** (gt_los_live `the_open_front_spot_still_deploys_the_gun`,
   gun_turrets_live `raising_ch18_deploys_the_wall_gun_and_its_shots_wound_bond`): plumb the game's
   `ext+0xd0 = script-A union bbox max.z × 1.5` [M 0x17ffa0:140] into `GTurret::muzzle_offset` and
   start the track ray at `t.pos + fire_dir·muzzle_offset` [M 0x17f9c8:230-258]. The port's script VM
   gap blocks the exact formula; candidate sources: the placement's own model bounds ([D], same gun
   model the script creates — verify the [L] 1.9865224 ≈ the placement bbox max.z × 1.5), or a
   per-level [L] table as the last resort. The candidate-side fix (TrackCand.sph) is already in.
2. **interact_07000009_live's armour take** — the round-1 instrumentation plan stands; add the
   0x721-mask drop trace (the take ray may now pass a face the game's mask drops, or lose one it
   keeps — print the ray's hit faces with their surface bytes).
3. **mp_net_slots_post_move_live** — map the digest's fields (the top byte moved, 0xfb→0xbc; find
   which state byte feeds it), then re-derive the pin from the game's behaviour per the task note.
4. **zoe_gondola_realpath_live, level_end_live, walkreal_07000008_live** — instrument the shifted
   rays (the [los] env-gated probe from round 1 is the template), re-derive the moved pins from the
   game's behaviour, no re-seeding until green.
5. **The full-suite total line** on the final tree (the runner needs a sandbox where background
   processes survive, or ~8 foreground chunks).

## Live checks for the main agent

1. **The 0700000a wall gun's deploy reach** [verifies the ext+0xd0 story]: fresh boot 0700000a, raise
   ch 18, stand Bond on the lower floor directly under gun #636 (9.84, ~0, 17.48) — watch whether
   `GT_Track`'s per-candidate `Collide_LineOfSight` returns non-zero for him (the gun deploys into
   COMBAT and shoots). Expected with ext+0xd0 = 1.9865224: the ray starts at y ≈ 0.29, under the
   mount — the game deploys. If the game does NOT deploy there, the muzzle push is bigger than the
   measured value implies and the ray start needs a second look (disasm 0x17f9c8:230-258).
2. **The standing player's obj+0x30 vs the floor top** [settles the sphere-centre geometry]:
   fresh boot 07000005, Bond standing still — read obj+0x30 y, obj+0x80 (world-sphere centre) y and
   the floor top under him. The ee_start_05 read is pos 24.97 / centre 24.837 (−0.1267); the floor
   top under that spot decides whether the game's centre rides above the deck (the port settles
   pos exactly ON the deck top, so its centre digs 0.127 under — if the game's pos is floor+0.14,
   the port's standing height is the real gap and it explains BOTH turret deploy rays and possibly
   the armour take).
3. **The armour take** on 07000009: fresh boot, walk Bond over the body armour — watch
   `Pickup_Update` 0x18e148's take branch and its `Collide_LineOfSight(…, 0x721)` return (does the
   real game's take ray get dropped by any face on this path?).

## Usage

Token, context-window, cost and credit figures for this session: not visible to me (no such counters
are exposed in this environment; nothing to copy exactly).

## ntfy failed

All three fallbacks failed (2026-10-26 ~12:32Z):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "capsule-cull-and-los-masks-2: ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds`
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'capsule-cull-and-los-masks-2: done (report on branch 3e4721ca; 5 groups diagnosed, next window continues)', headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "capsule-cull-and-los-masks-2: done (report on branch 3e4721ca; 5 groups diagnosed, next window continues)" https://ntfy.sh/nightfire-yoda-8k3x`
   → no output; the call was killed at the 30 s tool timeout (network to ntfy.sh unreachable from this sandbox).
