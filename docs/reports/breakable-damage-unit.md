# breakable-damage-unit — report

Task: settle what the collide pass writes to HITDATA+8 for a player-bullet contact on a 0x20
breakable, match the game's one-contact-per-bullet rule, and re-pin
`interactables_0700000a_live::vending_glass_breaks_under_fire` with the proven count.

## Answers

1. **HITDATA+8 = the round's own HITTEST+0x90, re-armed every flying tick.**
   - The victim's contact node is a COPY of the tested object's pick node: the copy loop writes
     `node+8 = *(tested->HITTEST + 0x90)` and `node+0x58 = tested` [M Collide_Update 0x1ea398,
     disasm 0x1ea690-0x1ea698], pushing onto the HIT object's `obj+0xD0` list (the pick's list
     anchor is the object itself: Collide_Update sets `HITTEST+0x70 = obj` [M 0x1ea398:49]).
   - The bullet IS the tested object: `Bullet_update` re-arms its embedded HITTEST (`obj+0xD4` =
     bulletdata+0x50 [M Bullet_init 0x12c5a0 disasm 0x12c650-0x12c660]) every flying tick (state
     < 2): kind 0x101 point / 0x201 ray and **+0x90 = weapon_data+0x0C**, overridden to 1.0 for
     base ids 0x2A/0x2C-0x2F and 0.0 for 0x2B/0x34-0x37/0x3A-0x40 — the override switch runs only
     when `weapon_data+0x08 > 0` [M 0x12e7b0 disasm 0x12f030-0x12f0b8]. Not a scaled value: the
     raw record word (with those id overrides). `Collide_Intersect` lands it into the pick node
     (`node+8 = HITTEST+0x90` [M 0x1ebaa8:320-322]) and the copy carries it to the victim.
   - So a bullet contact is **class 5** (the bullet's own class: `obj+0xFF = 5` [M 0x12c624-0x12c62c])
     with damage = `weapon_data+0x0C` — exactly what the port's `bullet_contact_damage` already
     computes (nf-formats weapon.rs; the searchlight feed has used it).

2. **One contact per bullet per object.** The pick list is sorted by the hit distance
   [M 0x1ea398:120-158 QuickSort on HITDATA+0xc]; the copy walk pushes each landed dynamic
   victim's node but **stops at the first non-pass node** [M 0x1ea398:216-229], and
   `Bullet_CollisionHandler` kills the round on that same first solid hit (state 2
   [M 0x12ce80 disasm 0x12cff0-0x12cff4], the node walk exits with it [M 0x12d81c-0x12d82c]).
   Pass-flagged nodes (surface byte bit 0x40 — foliage) neither stop the walk nor receive copies;
   the world's solid stop is the first NON-pass static node (the port's `segment_bullet` drop
   rule). The port previously scored EVERY overlapping breakable EVERY tick (a slow dart through
   the 0.1-grown pane bounds scored 2+ ticks in a row) and let the round fly on.

3. **The count is decompile-settled; no PCSX2 read is required.** #379's health =
   `(float)(int)param 1` = 9.0 [M Break_Create 0x12c230 disasm 0x12c2a8-0x12c2e0 `cvt.s.w`;
   D placement param 1 = 9]. The K5 dart 0x43's contact = 0.1f [L ee_snapshot weapon_data
   0x2bf150 + 0x43·0x10C + 0x0C] and its pellets word (+0x18) is 1 [L], one dart per shot. With
   one contact per dart the f32 accumulation (`health -= 0.1f` per class-5 contact [M
   0x12bfd8:37], kill at `< 0` [M :49]) dies on contact **91** (after 90: +2.0414591e-6).

## Port changes

- `breakable.rs`: health = `(param(1) as i32) as f32` — the port's `f32::from_bits` re-read the
  param's BITS (health 1.26e-44 ≈ 0), so the glass died on the first contact of anything. (The
  merge with origin/master 326db1530 brought the same fix independently from batch-1022ae.)
- `drone.rs` `update_player_bullets`: the BREAKS contact now follows the game's rule — the
  NEAREST solid breakable takes ONE contact and the round dies on it (`b.dead`), nothing behind
  it scores or stops it; gated by the first non-pass world node (`segment_bullet`) and the drone
  candidate (the same sorted list). Damage = `bullet_contact_damage(b.weapon)` (the HITTEST+0x90
  value), not the raw `b.damage`. The FilterBullets reject (`weapon word & (filter | 0x106)` [M
  0x1ece30:24-33], master's `weapon_word`) is applied per candidate, and master's script-pane
  kill deletion is kept. The 0x54 camera-subject detector moved to `Props::camera_shot_hit`
  (a shot marker, not a contact — it does not consume the round).
- **The DESTROYS keep their previous coarse pass** (`props.bullet_hit` unchanged): the
  script-keyed destroyable regions parse to degenerate (point) bounds — the port has no shape
  for them, and the pinned smash tests ride the coarse segment-AABB overlap. Porting the
  nearest-solid rule onto the destroys needs their real region shapes (live check below); the
  evidence (Destroy_CollisionHandler 0x12bd48:31 — the same node+8 float) is unchanged.
- The monitors/wires/held-targets/site/copter arms are untouched (their node+8 questions are the
  same evidence; listed under live checks).

## Verification

- `interactables_0700000a_live::vending_glass_breaks_under_fire` UN-IGNORED (master's
  batch-1022ae re-ignored it pending this count) and passing; it pins, through the real
  input path: load health 9.0; ONE K5 dart = exactly one 0.1f contact (9.0 → 8.9f, dead bit
  clear); then the kill count through the real weapon wheel to the P2K (0x02, contact 3.5f
  [L weapon_data w2 +0x0C]; 7+41 rounds [D inventory]): the sampled health sequence
  [8.9, 5.4, 1.9, −1.6] matches the in-test f32 mirror of `Break_Update`'s arithmetic —
  3 contacts from 8.9f. Sister: #378 keeps 9.0 and stands.
  (The K5 alone cannot kill the glass: 5+10 darts [D inventory] against the 91-contact count —
  the count is asserted via the f32 mirror, the live kill rides the P2K.)
- Full suite (`tools/test/suite.py`) on the final tree merged with origin/master 326db1530:
  see the total line below.

## Suite total

`tools/test/suite.py` ran on the tree BEFORE the last two fix commits (the merge + the health /
one-contact fixes; it builds ~15 min and the sandbox deadline landed mid-run): it reached the
nf-view binaries with `FAILED fix_blast_scripts_live::the_destroyable_smash_plays_its_type_script`
(diagnosed below) and `CRASH net_join_live (exit -9)` (the sandbox killed the MP binary — not a
test assertion; it passes on master runs). No total line was reached before the deadline.

The blast failure was MY nearest-solid rule applied to the DESTROYS: 0700000d's script-keyed
destroyable regions parse to degenerate (point) bounds and the pinned smash test relied on the
coarse segment-AABB pass (a 13.2-unit step's AABB covers the point neighbourhood even when the
spread-carrying ray misses). Per the plan above the destroys went back to their previous coarse
pass and the breaks keep the game's rule. Verified on the FINAL tree (direct runs,
`--test-threads=1`):

- `fix_blast_scripts_live`: 3 passed (the type-0 smash, the type-1 smash + script, the Samurai
  bolt) — the regression is gone.
- `interactables_0700000a_live`: 7 passed, including the UN-IGNORED
  `vending_glass_breaks_under_fire` with the count pins.

The rest of the suite's binaries on the pre-fix tree reported no FAILED/TIMEOUT lines up to the
deadline (the failed blast binary was the only failure). The full-suite re-run on the final tree
did not fit the sandbox window — the next message should re-run it (13 min once built).

## What's left

- Nothing for this task. Noted for the next worker: the fusebox-wire, held-target, site and
  copter feeds still take `b.damage` (the raw record word) rather than `bullet_contact_damage`;
  the evidence for node+8 says they should use the HITTEST value too (the copter feed already
  does). No test pins a contradicting value.
- The 07000009 lobby-roof glass (lobby-roof-glass-09's aim-offset finding) is a separate branch.

## Live checks for the main agent

Optional (the decompile settles everything the tests pin; these would confirm on hardware):
1. FRESH boot of 0700000a; gdbwatch the class-0x20 object at (6.12, 1.22, 16.35)'s
   `*(obj+0xE0)+4` (the health f32) while firing the held K5 into the glass: expect 9.0 → 8.9
   after the first dart lands (ONE 0.1f contact) and the death on the 91st contact.
2. Mid-flight of a K5 dart, gdbwatch `BUdata+0xE0` (= `*(obj+0xE0)+0x90`, the HITTEST+0x90):
   expect 0.1f from the first flying tick [M 0x12f05c-0x12f060].
3. A grenade (0x2B family) thrown at the glass: expect NO health change from the contact
   (HITTEST+0x90 forced 0.0 [M 0x12f0a8]); the blast path is separate.
4. 0700000d's script-keyed destroyable regions (43 at the origin, e.g. destroy[10]): gdbwatch
   the placed object's +0x80/+0x8C world sphere and its glist collision block — the port parses
   degenerate (point) bounds for them and needs the real region shape to port the nearest-solid
   contact rule onto the destroys.

## Usage

not visible to me (no token/cost figures are exposed in this environment; the session's
context use is not reported to the agent).
