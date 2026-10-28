# level-0d-c5-door-laser — Chain Reaction C5 door vs the deck laser

Branch `s/level-0d-c5-door-laser` (base master 494f74fd). **Status: the machinery is ported and
the handheld-filter + beam-ray behaviour is pinned green; the sustained-fire drain path needs one
more diagnosis round** — the mount lands ~5 contacts through the test harness and then stops
taking effect, so the end-state drain test is `#[ignore]`d with the reason on the branch. The
sandbox expired mid-task; the next message continues from the branch.

## Done

- **The 0x20 gate** [M Explode_Propagate 0x17c518:168-181]: `explode.rs`'s case 0x21 now smashes
  only when the leaf's tag+0x14 mask clears bit 0x20 (`Targets::destroys` carries the mask;
  `drone.rs run_beam_explodes` feeds `x.mask`). The C5 doors' tag+0x14 is 0xFFEF
  [M Destroy_Create 0x12c318:47-52 `~p4`; D #93/#96 p4 0x10] — no blast ever smashes them. Unit
  test `destroy_case_respects_the_0x20_mask_bit`.
- **The bullet mask filter** [M Collide_FilterBullets 0x1ece30:26-33]: `Props::bullet_hit` drops
  every class-5 contact whose `weapon_data+0x10` word ANDs non-zero against tag+0x14. The
  category words are [D] (`probe_0d_destroy_lod`): M6 0x01 → 0x0001, 0x08 → 0x0008, 0x2e →
  0x0020 — all dropped; the mounted 0x61/0x67/0x70 → 0x0010 — kept. The caller passes the same
  word the fusebox path already read (one read, two consumers). End-to-end green:
  `bond_own_gun_does_nothing_to_the_c5_door` — Bond dismounts, draws the M6, 120 held-fire ticks
  at the door: health 10000, colour white, no channel.
- **The hit-colour pair** [M Destroy_CollisionHandler 0x12bd48:61-95]: `Destroy::colour`
  (obj+0x104/0x105) — a weapon-0x67 contact tick drops both bytes 5 (clamp 0), any other tick
  recovers 1 (clamp 255), the death return freezes the pair; `Props::destroy_colour_tick` runs
  once per sim tick beside `channel_kills`. Exposed `pub` for the draw. Unit test
  `the_colour_pair_drops_under_the_beam_and_recovers_after` (255→0 in 51 contact ticks, +1
  recovery to 0x3b [L 00ff3b3b], the death freeze, the non-0x67-contact recovery).
- **One 50-damage contact per round** [D the doors' collision block; M Bullet_CollisionHandler
  0x12ce80's solid death]: the destroy leaves now stop the round (`destroy_stop` after the
  drain registers in the props pass). The beam's 0x67 round carries 50 damage
  [D weapon_data+0x67·0x10c+0x0c; the record: range 500, speed 16.67/tick, spread 0].
  A per-tick trace through the real mount shows exactly −50.0 per landed contact with ~3 rounds
  in flight — the measured rate [L ~50/frame].
- **The merged test corrected**: `0700000d_gunimp_door_live.rs`'s one-contact-smash test
  contradicted [L PCSX2 2026-10-27 (10000 → 7100 → 4050 over two 60-frame bursts, ~200 contact
  frames)] and [M 0x17c518:170] — rewritten per the rules (the TEST was wrong).
- **asm findings for the record** (the decompile's dropped store/args, per the rules):
  - `Destroy_Smash`'s `Control_SetGList` argument is `*(tag+0x00)` [M 0x12bbb8-0x12bbc8]:
    `Destroy_Create` fills it with `*(glist+0x40)` — the block-0x27 LOD pointer [M 0x12c318:23-26]
    — or, when the disc word is −1 (nothing written [M 0x1d1f10:14-17]), the parsemap fallback
    `pCelList + glistIndex·0x50 + 0x50` [M 0x1d1b68's loop head] — the brief's "glist + 0x50
    destroyed entry" exactly. The C5 leaves carry no LOD variant [D probe: lod_word −1 for every
    0xe1 on 0d], so the swap lands on an undrawable slot — the doorway ends open [L].
  - The 0x67 fire arm's `Explode_Propagate(4.0, 5.0, hit, cel, imp)` [M 0x1320d8:433-436] runs
    every beam hit; the gate above makes it non-lethal to the C5 leaves — the drain is the death.

## What's left (the next message continues)

1. **The sustained-fire quirk**: through the harness, ~5 contacts land and then the health
   freezes while the trigger stays held (the beam quad stays up, the heat climbs — hot at 100
   shots; the bursts show `fired 30` with zero new contacts). The first burst drains 10000 →
   9750 + one flash drop, then nothing. The trace variant (firing while the aim's look inputs
   are still held each tick) drains −50/tick steadily, so the defect is in the sustained
   no-input fire path (aim easing? the fire arm's re-trigger condition?), not in the drain
   itself. Diagnose, then un-`#[ignore]`
   `the_beam_drains_the_c5_door_at_50_a_tick_and_the_smash_fires_at_zero` — its asserts are
   already written: the death catch at exactly −50 [L], bigdoor01 dead the SAME tick at full
   health 1000 [M 0x12c4f8:29-33 + L], ch 0x1f [M 0x12bb60:40], objective 6 → state 4
   [M 0x1827d8:78-85].
2. **The draw-side destroyed-glist hide**: the sim models the swap as `Destroy::dead` [M], but
   the port has no per-frame static-instance visibility hook (the same shape as the 0x3b
   channel-show gap) — scene.rs's 0xe1 comment documents the swap's real semantics. Needs the
   hook, then `--bond` renders at the doorway before/after the smash.
3. **p6 > 0 deferred channel writes**: `Destroy_Update`'s countdown arm — the decompile's
   create-stores-negative/handler-negates/Smash-negates sequence never reaches the countdown;
   needs the live check before porting. The C5 doors are p6 = 0 [D] — the immediate-write path
   the port has is exact there.
4. The full suite sweep on the final tree (see below for what ran).

## Verification (what ran)

- `cargo test -p nf-game --lib`: **581 passed, 0 failed** (includes the new props/explode units).
- `0700000d_gunimp_door_live`: **2 passed, 1 ignored** (the handheld filter + the beam stop; the
  drain test carries the ignore reason above).
- Not run: the full four-crate sweep on the final tree — the sandbox window closed during the
  sustained-fire diagnosis. The earlier background baseline died between tool calls (this
  sandbox kills background processes), and the chunked foreground sweep did not fit. **Next
  message: run the full suite before merge.**

## Live checks for the main agent

1. Sustained beam on bigdoor02's face: confirm the health drain pauses when the heat byte
   (ext+0x9c) saturates — is the hot state the PS2's fire gate (`GunImp_Update` 0x1320d8's
   cooldown arm under the hot byte), and does the door take contacts while hot?
2. While the beam is on the door, does obj+0x104/0x105 step −5 per frame or per contact (the
   recovery measured ~3/display-frame vs the [M] +1 per handler call — the call rate needs one
   watch)?
3. After the smash, confirm both door meshes vanish (the glist+0x50 swap target) and the
   objective-6 record completes the tick after the channel write.

## Usage

not visible to me
