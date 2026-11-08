# guard-standing-capsule — the standing body a firing guard owes Bond

## The task

A guard playing a fire animation had no standing body for Bond to collide with in the port; the
game keeps one (docs/reports/level-0d-wire-jump-grab.md, s/level-0d-wire-jump-grab — its "what's
left" item 1 and live check 1). Read the game's drone collision shape per state
(Drone_CollisionHandler 0x140968, the body capsule/sphere setup) and give the port the same body in
every state the game has it. Pin a bump into a firing guard on a real level, real input, no
teleport. No draw changes.

## The game's body, read from the decompile

- **The shape** [M `NDrone2_Collision` 0x140e90:91-121, written to drone+0x3f0/+0x400/+0x45c]:
  standing branch — top = obj.pos + up·r, bottom = obj.pos + up·(r − hip), r = 0.4 (0.55 when the
  kind word +0xd8 == 0xc), hip = the ANIMATED +0xa0 [M `NDrone2_DefaultInit` 0x14b300:48 default
  1.0; the sit writes 0.2 — the port's `hip_anim`]. A degenerate single sphere (r 0.3, centre
  pos + up·(0.3 − 0xa0)) when drone+0x2f != 0 [M 0x140e90:65-89 — the sitters/riders arm; not
  modelled — a live check item]. With +0x4F8 & 0x200 the arms come from the POSED BONES (5/0x37/
  0x33) [M 0x140e90:123-166] — the dead shape, unusable for the player push (next point).
- **When it is ON** [M `NDrone2_DoCollision` 0x149ec8]: only the dead bit 0x4F8&0x200 (set by
  `NDrone2_SetAsDead` 0x14a2f0:9, which also clears 0x20/0x80 — except level 0x700004a) or the
  anim-info word +0x570 bit 2 [M `DroneAnim_SetDAnimInternal` 0x13aee8:50 — the per-anim no-body
  flag; not modelled — a live check item] turn it off; `DroneFunc_NearPlayerCollision` 0x14b190
  turns it ON within 2.0 of a player otherwise. **No STATE test anywhere**: a firing guard's
  combat states set 0x4F8|0x20 [M `DroneFunc_SetAsAttacking` 0x14aed8, `DroneFunc_FirstAttack`
  0x148010, `NDrone2_SendAttackMessage` 0x14a410 — 40 writers] and stand inside the 2.0 band with
  the body on. `Drone_CollisionHandler` 0x140968 consumes the centres for the hit flags
  (obj+0xdc+0x60 bits 0x8/0x10/0x20 from hit+0x40 vs +0x3f0/+0x400) [M 0x140968:96-125].
- **The player bump** [M `Collide_Jointy` 0x1ec180:229-281]: the player's kind-0x800 query
  resolves against the bot's capsule hittest on EVERY contact tick — grounded included; SP
  (MPSettings+0x384 == 0) applies the FULL overlap (fVar12 = 1.0) to the class-3 player and
  returns 0 for a class-3 other (the bot never pushes itself); the class-3-vs-3 and class-2-vs-2
  gates don't fire on a mixed pair. No friend/foe test — allies push Bond too.

## The port's gap and the fix

The port already had the Jointy push (`player.rs`, the actor-actor capsule push) with the bot
capsules built in `scene.rs` — but:

1. the grounded-side push rode **only for CIVILIAN_DOOR_GUARD bots** (an [S] port choice whose own
   comment flagged the firing guards as a live-check item);
2. the capsule arms were the fixed `(pos+0.4, pos−0.6, 0.4)` — the hip-1.0 special case.

Fix (committed):

- `crates/nf-view/src/scene.rs`: every near bot (enabled, present, not a corpse — the 0x200 dead
  gate) contributes the game's standing capsule — top = pos + up·0.4, bottom = pos + up·(0.4 −
  hip_anim), r 0.4 (the 0.55 kind word and the +0x2f sit arm stay unmodelled live checks) — and
  `ground_bots` is every near bot, per 0x149ec8/0x1ec180 above.
- `crates/nf-game/src/player.rs`: the gate comment now states the game's rule (the old [S]
  airborne-only note superseded); the push code itself is unchanged.

## The proof (red on the old code, green on the new)

`crates/nf-view/tests/guard_capsule_live.rs` — `a_firing_guard_stands_between_bond_and_his_post`:
07000002 loaded, the intro plays out, the real route (spawn → ladder #114 → garden → west wing →
the ch-0x17 plate, its raise driven by the walkreal's own documented fallback) enables the geisha-1
captor group, `walk_nav_to` into the captor's room (3.0 arrive), then Bond is steered at the
nearest live captor's live position with real movement input every tick (no pos writes after the
spawn), 480 ticks.

- Pre-fix: the captor stands in his fire family the whole approach (states 0x74/0x68/0x73) and
  Bond walked **through** him — **min XZ distance 0.189**. RED (assert min ≥ 0.8).
- Post-fix: Bond presses against the body — **min XZ 0.942**, exactly the game's spine gap
  (0.55 + 0.4, the Jointy overlap resolved). GREEN.

## What ran (each binary whole, one per tool call)

| binary | result |
|---|---|
| `guard_capsule_live` | 1 passed (the new pin; red pre-fix) |
| `walkreal_07000007_live` | 5 passed, 1 ignored — the door-guard level, unchanged |
| `walkreal_07000001_live` | 6 passed, 1 failed, 1 ignored — see the residual below |
| `walkreal_0700000a_live` | 12 passed, 1 failed, 2 ignored — see the residual below |

The full suite did not fit this window (the 108 walkreal binaries alone exceed it); the change is
global (every level's near bots), so the two residuals and the unrun binaries are the next work.

## The residuals (the next session's first items)

1. **walkreal_07000001 `following_mayhew_down_completes_the_level`**: the contact (drone #1, team
   2) died at t15517 — guard #15's weapon-28 burst, 45.0 to the head, `by_player false`
   [L NF_HEALTH_TRACE run]. The push moved Bond's follow spots (game-accurate), the guards' lines
   shifted, and the contact ate a round. This is the contact-in-the-line-of-fire issue the
   s/level-01-start-guard-contact task owns ("the guard's rounds can kill the contact from 0.04 u
   off the line"); the port's contact death under shifted player lines needs that task's guard-fire
   rules, not a walk hack. The other six 01 tests pass after the pre-stairwell sweep learned the
   block's own pocket rule as a flank bound (`flank_hold_x = Some(-27.0)`) — the push had drawn a
   flank into the stairwell mouth and a wedge sidestep carried Bond down onto the contact's plate
   (the exit fired mid-fight).
2. **walkreal_0700000a `west_doorway_repro`**: the mission failed during the ring walk at
   "the ai node 34 south" (level_exit armed; the walk was at (−15.08, 1.12, 3.99)) — the named
   west-doorway ally zone; the ring's shifted path needs the same triage (what fired, at which
   spot) next session.
3. Every other `walkreal_*_live` binary (the change is global), plus `npc_doors_live` /
   `fix_guard_misc_live` (drone.rs untouched this time, but the collision feeds them).
4. The unmodelled game arms, if wanted: the 0.55 kind-0xc word, the +0x2f sit sphere, the
   +0x570-bit-2 no-body anims, and the posed-bone (0x200) shape.

## Live checks for the main agent

1. **The bump on hardware.** In PCSX2 on 07000002, walk to the geisha-1 room (−45.4, 5.9, 11.7),
   let a captor open fire (states 0x70/0x71/0x73/0x74), and walk INTO him. Expected: Bond stops
   ~0.9-1.0 from the guard's origin and slides along the body — the port now matches (min 0.942);
   the pre-fix port walked to 0.19 (through him).
2. **The ally push.** On 07000001, stand beside Mayhew within 1 u: a grounded Bond is nudged off
   him (Jointy, no friend/foe gate [M 0x1ec180:229-233]). If hardware does NOT nudge, the class
   byte at hittest+0xff for the ally lead is not 2 — a gdbwatch on the query's +0xff bytes during
   contact settles it.
3. **The anim-info OFF bit.** Any anim whose +0x570 word carries bit 2 (knock-down? rope?) should
   present NO body. A gdbwatch on 0x149ec8's `+0x570 & 2` branch during a guard's knock-down pin
   names which anims; the port then models the flag.

## ntfy failed

All three fallbacks, in order, with their output:

1. `curl -sS -m 15 -H "Title: NightfirePort" -d "guard-standing-capsule: blocked …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 15001 milliseconds`
2. python3 urllib (the driver's fallback line, timeout 15)
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -O- --post-data "guard-standing-capsule: blocked …" https://ntfy.sh/nightfire-yoda-8k3x`
   → DNS resolved (159.203.148.75), the TCP connect printed, then the TLS handshake stalled past
   the 20 s timeout with no HTTP response — the delivery could not be confirmed.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session; nothing to copy.
