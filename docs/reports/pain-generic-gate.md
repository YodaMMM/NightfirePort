# pain-generic-gate — the game's health gate on every player damage path

Branch `s/pain-generic-gate` from origin/master at 290dc4b03. Task: `Player_HandlePain`
0x1902c8 returns on health <= 0 before writing anything; the port's `pain_generic` path
lacked that gate. Every player damage entry now carries the game's gates in the game's
order, and the pre-fix leak is pinned red→green.

## What the game does (evidence)

`Player_HandlePain(obj, BL, damage, type, team)` 0x1902c8's gate order
[M decompile 001902c8 :14-26 + disasm 0x1902f8-0x190378]:

1. `switch_channels[0x5f]` set → return (`lbu` 0x26FCEF, `bnez` 0x190600);
2. health `BL+0x894` <= 0 → return (`lwc1 f0, 0x894($s1)` / `c.ole.s` / `bc1t 0x190604`);
3. damage <= 0 → return (`c.ole.s f20` / `bc1tl 0x190608`);
4. (MP) `MPSettings._388_4_ != 0 && GameFlow_GetState() != 2` → return (jal 0x1c9e80).

The body's FIRST store is `BL+0x8bc = 1.0` at 0x190378 — a gated pain writes NOTHING:
no `+0x967` direction bits (the generic store sits at decompile :121-125), no `+0x968`
glow (:126-130), no armour/health (0x1904ec MIN / 0x190510 MAX), no 1-in-4 grunt draw
(`Rand_Rand(4)` :110-119 is body code). `BL+0x8bc` is the pane fade the HUD reads
[M HUD_UpdateHealthPane 0x19b9c8:118-122; profile.rs:117].

Every damage entry reaches that one function:

| Game entry | Path | Pain type |
| --- | --- | --- |
| `Player_DealWithObjHit` 0x194458:99-101 (class 0x27) | hurt-volume contact | `Hurt_GetType` |
| `Player_DealWithObjHit` 0x194458:195 (class 5) | bullets + melee (record 0x65/0x66/99) | 0 |
| `Player_CollisionHandler` 0x1949a8:120 | falls | 6 |
| `Player_MonitorAir` 0x191a70:142 | drowning | 7 |
| `Player_Hurt` 0x196060 (type 0, team −1) | blasts (`Explode_Propagate` 0x17c518:125 case 3), live wires (`FuseBox_Update` 0x1adfe8:100), creature (`Creature_Hurt` 0x1352d0:17), sub (`Sub_Update` 0x1ad200:108) | 0 |
| scripted kills: `Player_Kill` 0x196098 / `Mission_Update` 0x182c60:40 | `Player_SetHealth(0)` DIRECTLY | — bypasses the pipeline by design; the port's mission-fail `Event::PlayerDead` (mission.rs:359) matches |

Two writers of `+0x967` exist and they differ [M 0x194458:195-226]: the bullet arm calls
`Player_HandlePain` FIRST, then writes the direction bits from the attacker's position
OUTSIDE HandlePain, unconditionally — a bullet on a dead body still writes the bits in the
game. Every other entry's `+0x967` store is HandlePain's own gated one.

## What the port did wrong (pre-fix)

- `PlayerHealth::pain_generic` (bullet.rs:447) set `dir_bits` BEFORE calling the gated
  `apply` — on a dead or zero-health player the port changed `BL+0x967` where the game
  writes nothing.
- The blast arms (drone.rs explode_propagate case 3 and spawn_blast case 3) wrote
  `pain_alpha = 1.0` (the port's `BL+0x8bc`) and pushed a `(frame, owner, Δ)` entry into
  `player_hits` even when `pain_generic` gated — a zero-damage ledger entry and a pane pop
  on a corpse the game never records.
- The live-wire arm (drone.rs FuseBox `Cut::LiveWireHurt`) did the same three writes plus
  `fusebox_hurts += 1`.
- The bullet arm pushed its `player_hits` entry unconditionally (zero entries for gated
  rounds). Its OTHER post-HandlePain writes (the direction bits, the 0x412/0x171 sounds,
  the melee hit-push) were already unconditional — those match the game and stay.

## The fix

- `apply` takes the generic `+0x967` value as `Option<u8>` and writes it INSIDE the gated
  body (after the health floor, with the glow) [M :121-125's position]. `pain_generic`
  passes `Some(0xC)`, or 2 for pain type 6; `pain` passes `None` so `pain_from`'s
  pre-write (the bullet arm's own unconditional store, [M 0x194458:203-226]) survives —
  the game's net effect for a bullet is the direction bits either way.
- The blast arms, the live-wire arm and the bullet arm now detect LANDED pain
  (`health != before || armour != armour_before` — exact: a gated pain moves neither
  store, and a landed pain always moves one, since type 0's fraction is 1.0 and types
  1..7 subtract the full amount) and gate the pane reset + ledger pushes on it.
- No draw changes: the 1-in-4 grunt draw sits inside the gated body exactly as the game's
  does; a gated pain consumes no draws on either side (the old code already gated the
  draw — the leak was the stores, not the stream).

## Verification

- **Red on the old code** (nf-game/tests/pain_gate_live.rs, run before the fix):
  3/4 tests failed — `pain_generic_writes_nothing_on_a_zero_health_player`,
  `the_pain_after_the_killing_pain_writes_nothing` (which also caught that the pin's kill
  needs 200 damage, not 100, at 05's difficulty-1 factor 0.6), and
  `a_zero_damage_pain_writes_nothing_on_a_live_player`. After the fix: 4/4 pass.
- **The real-input death walk** (nf-view/tests/pain_gate_dead16_live.rs, opt-in via NF_ISO):
  07000016 on the chain carry (14 → 15 warm saves, difficulty 1), walker allowances off
  (no top-up, no return fire, `die_panics = false` — this walker instance's own flags, no
  shared-helper change), real input only from the load: the corridor AR fire lands 28 real
  ledger hits, the slab ch-0x81 touch runs the launch chain, the pit pulse's 1000-damage
  contact kills through the real pipeline at t7005, and 600 post-death hold ticks pin
  health floored, `player_hits` frozen at 28, `pain_alpha` never rising, `dir_bits`
  frozen. **Caveat, stated plainly**: no further pain ARRIVED at the corpse on this route
  (the witness print says so), and the pulse's own post-death writes would be
  value-idempotent (0xC → 0xC) — so this walk is a regression pin through the real
  pipeline, while the pre-fix value difference is proven red by pain_gate_live. A route
  where a guard bullet/blast demonstrably reaches the corpse is listed below as left.
- **Test binaries run whole** (one level per call where level-bound):
  - nf-game lib 754 passed / 5 ignored; ai_live 37; progress_live 5; anim_events_live 1
    (the player_hits consumers);
  - walkreal_07000016_live 15/15; walkreal_07000005_live 16/16; walkreal_07000015_live
    7/7; walkreal_07000008_live 5/5; walkreal_07000001_live 7/7;
  - the death suite: fix_death_restart_live 2, death_anim_sp_live 2, camera_death_crouch_live 2,
    dead_contacts_live 8, fix_restart_health_live 6, fix_restart_target_live 2, fix_hud_live 2
    — all green.
- The full four-crate suite did not fit the window; the binaries above are the ones whose
  assertions touch the changed code (`grep -l pain_generic|player_hits|pain_alpha`).

## What's left

1. The remaining walkreal binaries: 03, 04, 06, 07, 09, 0a, 0b, 0c, 02/12/16-adjacent
   suites (fix_death_loop_07, fix_guest_reactions, mp_death_effects, pause_results_flows,
   fix_death_anim_events, fix_death_retry_boost were not run this window).
2. The `switch_channels[0x5f]` gate [M 0x1902f8-0x190310] is documented in `apply`'s
   comment but NOT wired: the port's `Channels` live on `Drones`, and wiring it into every
   pain call site needs either a channel arg or a Drones-level pre-check. No surveyed
   level evidence was gathered for channel 95 writes this window; wire it when a level
   that sets it is identified.
3. The MP gameflow gate [M 0x190344-0x190364: `MPSettings._388_4_ != 0 &&
   GameFlow_GetState() != 2`]: `mp_pain` (mp/equip.rs:152) is the pure multiplier block;
   the match-state gate is the MP world pass's job (scene.rs net_world_pain) — unchecked
   this window, SP-only task.
4. Flagged divergence, NOT changed (live-player behaviour, needs its own proof):
   `BL+0x8bc = 1.0` is written by EVERY pain that passes the gates [M 0x190378], but the
   port sets `pain_alpha` only on the blast/live-wire/pickup paths — the fall (type 6),
   drown (type 7) and hurt-volume paths never pop the pane fade. The fix above gates the
   existing writes; it does not add the missing ones.
5. A real-input route where a post-death guard bullet or blast demonstrably reaches the
   corpse (the corridor guards ceased fire at the death here), to turn the walk's pin from
   regression-grade to red-grade.

## Live checks for the main agent

Not required for the merge — the disassembly settled the gate order. Two optional checks
if a PCSX2 session is open anyway:

1. Address 0x1902c8 (Player_HandlePain), watchpoint on `BL+0x8bc` (BL = glb_players+0xe0)
   during a hurt-volume death (07000016's shuttle pulse): expect the write ONLY on ticks
   with health > 0 — after the health collapse the store must not re-fire every tick.
2. Address BL+0x967 (the direction byte) while a dead body takes guard rounds: expect the
   byte MOVING (the bullet arm's unconditional post-HandlePain store) while BL+0x8bc and
   BL+0x968 stay put — the two-writer split the fix models.

## Usage

not visible to me (no token, cost or context-window figures are exposed to this session).
