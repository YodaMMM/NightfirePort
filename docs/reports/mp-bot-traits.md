# mp-bot-traits — MP bots use their 360-degree vision and health regen traits

Branch `s/mp-bot-traits` (from origin/master 22d5836b7). Code: commit 55c7be28c (the traits)
+ this report.

## What's done

**The flag-byte audit** ([M] every read of the bot record's stats flags byte — the copy at
`BOT_vars+0xac` = record byte 0x0C, the record copy pointer `Drone+0xd1c` [M BOT_init
0x1249e8:134-141]):

| bit | site | what it does |
|-----|------|--------------|
| 0x01 (bit0) | `NDrone2_SetOpponentAimPos` 0x141fe8:146 | the aim-noise roll: with the bit the miss denominator is fixed 4 (miss 1/4); without it `aim_byte*4` (miss 1/(aim*4)). NOT ported (see What's left). |
| 0x02 (bit1) | `NDrone2_DoHitEffects` 0x1458e0:49, `Player_DealWithObjHit` 0x194458:133 | the shooter bot deals ×1.25 damage with any weapon outside the fists family. PORTED. |
| 0x04 (bit2) | `NDrone2_DoHitEffects` 0x1458e0:43, `Player_DealWithObjHit` 0x194458:127 | the shooter bot deals ×1.5 with the fists family (weapon ids 1, 100, 101, 102). PORTED (damage). The punch-selection arm was already ported (combat_weapon_block, 1-in-200 vs always). |
| 0x04 (bit2) | `NDrone2_DSTATE_BotGlobal` 0x16e060:317/376 | up close and facing, the bot skips the 1-in-200 dice and takes the punch at once. Already ported. |
| 0x08 (bit3) | `NDrone2_FindOpponent` 0x1424e0:288 | the acquisition limit becomes 90000.0 (=300², else `Drone+0xE8²`=576.0²) and the cone drops. Was half-ported; COMPLETED (see below). |
| 0x08 (bit3) | `NDrone2_DSTATE_BotGlobal` 0x16e060:187-218 | the sight ack: a held player opponent with the raw seen state sets `Drone+0x228` bit 2 the same tick — the reaction-time delay (HaveOpponentSight 0x175ef0, the port's seen_ticks>=10) does not apply. PORTED. |
| 0x10 (bit4) | `BOT_init` 0x1249e8:165-167 | arms `BOT_vars+0x738 = GameState+52 + FRAME_RATE_INT`. Already ported. |
| 0x10 (bit4) | `NDrone2_DSTATE_BotGlobal` 0x16e060:209-233 | the regen tick: +5.0 health, clamp to the stats' max HP (`BOT_vars+0xa4` u16), `BOT_SetHealth` 0x126758, re-arm +60. Was a stub — only the deadline re-arm existed, the +5.0/clamp were missing. PORTED. |

Key evidence details:
- `FRAME_RATE_INT` = 60 (`GameState+52` = the frame counter at 0x2A379C [L ee_snapshot 55165;
  docs/fields/drone-statemachine.md:66]; FRAME_RATE_INT global, bots.rs:634, drone.rs:4775
  audit note) — the regen is **+5 HP per second**, clamped to the stats' max.
- The damage perk scales `HITDATA+0x08` BEFORE HitDamage/HandlePain and keys on the SHOOTER
  bot's flags + current weapon (`BLData+0xe0` control → `+0x30` obj → `+0xe0` → `+0xd1c` →
  `+0xac` [M 0x194458:120-140]; docs/fields/hitdata-spawner-ally.md:32 "MP perk scaling").
- The wide arm's score gate is `(row & 4) != 0 || vision_bit` and the cone-reject tail is
  `else if (vision_bit == 0)` [M 0x1424e0:364-366] — a 360 bot needs NO seen bit and NO cone:
  any candidate within 300² scores, including directly behind and never-seen.

**The port changes** (`crates/nf-game/src/mp/bots.rs`, `crates/nf-view/src/scene.rs`):
1. The regen heal: the fired tick now computes `(ctx.health + 5.0).min(ctx.max_health)` and
   rides the new `BotBrain::regen_heal` to the scene, which folds it into `MpBot::health`
   right after `step` (the game writes `Drone+0xac` via BOT_SetHealth; the port's float lives
   on MpBot — mp/mod.rs:586). Deterministic: no RNG, fixed order.
2. The 360 fast-ack in `bot_global_msg3` [M 0x16e060:186-218]: flag bit 0x08 + held player
   opponent (< OBJ_ROW; the game's Control_Plr2Ind gate) + LOS segment clear →
   `vars.sighted |= 4` the same tick (the game's row bit 2 is the raw LOS-only CanSeeObject
   [M standins-scene-929]; the port's player rows carry only the candidate bit under lazy
   sight, so the segment IS the row's seen bit). No +0x238 stamp, matching the arm.
3. `find_opponent`'s wide arm completed [M 0x1424e0:288-299, 364-366]: the seen-bit pre-gate
   and the lazy sight chain (segment + OpponentVisibility ≥ 0.7, whose cone would have
   defeated the wide arm) now exempt a wide bot; the candidate bit (row & 2) still wraps both
   arms.
4. `bot_damage_trait_scale(flags, weapon, dmg)` — the ×1.5/×1.25 shooter perk as a pure
   function, applied at both bot-shot damage sites in scene.rs before `mp_pain`
   (bot→human and bot→bot). The MpShot fx event keeps the raw weapon damage (draw/audio only).

## The flag census [L]

The four Bot Setup records at 0x2DEEAA (identical in all EE snapshots) all read flags 0x00
[L INITIAL_BOT_RECORDS, bots.rs]. The 29 `default_bot_stats` presets at 0x26D2F0 [L, in
bots.rs] — the ones the bot-choose screen copies into a record via `BOT_getDefaultStats(list
index)` → `amemcpy(0x2deeaa + i*0x12, stats, 0xe)` [M C_SBMPBTCHOOSE_Handler 0x210f08:49-62]:

| preset row | flags | meaning | max HP |
|---|---|---|---|
| 17 | 0x01 | bit0 fixed-4 aim roll | 150 |
| 20 | 0x02 | bit1 damage ×1.25 | 200 |
| 21 | 0x02 | bit1 damage ×1.25 | 200 |
| 24 | 0x04 | bit2 melee ×1.5 + always-punch | 300 |
| 25 | 0x18 | bit3 360 vision + bit4 HP regen | 150 |
| 26 | 0x04 | bit2 melee ×1.5 + always-punch | 150 |

Rows 0-16, 18, 19, 22, 23, 27, 28 read 0x00 (23 of 29 rows flag-less). The bot NAMES for
these rows are the disc's bot-choose list labels (Txt_BindLabel ids, not in the decompile) —
row N of the list maps to preset row N [M 0x210f08]; the names need a disc-text pass (live
check below).

## How it was verified

- Five new nf-game lib tests (`bot_trait_*`, all pass): the regen rate (no heal before tick
  60, +5.0 at it, re-arm, clamp at the stats' max 150, holds), the no-bit sister (never
  heals), the 360 acquisition (an unseen target directly behind: the wide bot takes it, the
  narrow bot never does; 301² stays out of the wide limit), the fast sight-ack (the wide rig
  acks on the acquisition tick; the narrow rig doesn't), and the damage-scale truth table
  (fists family ×1.5 with bit2, ×1.25 with bit1 on other weapons, the else-if order, no
  flags no scale).
- nf-game lib whole: **689 passed, 2 failed, 5 ignored** — the 2 failures
  (`fell_out_map_*`) panic at bot_init's pre-existing `(slot - 4)` with slot 1; reproduced on
  PRISTINE origin/master in a scratch worktree (same panic at its bots.rs:1368) — a master
  bug, not this branch's regression (found while running the suite; not fixed here to keep
  the diff to the task).
- Suite (tools/test/suite.py, release, --test-threads=1): the build is green; the run got
  through 24 binaries (bank_live … frustum_cull_live) all passing before this sandbox's
  window cut it — **the full-suite run is NOT complete** (see What's left).
- The task-critical mp binaries, run whole on the final tree: mp_bots_live 3/0,
  mp_bots_moving_live 3/0, mp_bot_weapons_live 2/0, mp_bot_objectives_live 10/0,
  mp_world_sync_live 8/0, mp_perf_16_live 2/0, mp_16_slots_live 3/0 (passed/failed).

## What's left

1. **The full suite run** — 24/1142 binaries done (all green); the mp_*_live set the task
   names is covered by the manual runs above, but the rest of the four-crate suite needs a
   longer window. No failure seen anywhere so far.
2. **bit0 (0x01, the aim roll)** — not ported: `SetOpponentAimPos` 0x141fe8:146-157 makes a
   `Rand_Rand(denominator)` draw per call while aiming; adding it shifts the shared match
   RNG stream for every bot, so it wants its own task with a stream audit (the wobble at
   +0x210 rides the same function).
3. The hit pipelines' +0x58 identity on mirrored nodes (the shooter-side reading used here is
   the docs' "MP perk scaling" + both doc annotations; a live check below settles it).
4. The `mp-bots` feature-gated dispatch skeleton note in bots.rs's header predates the wired
   brain — stale comment, next doc pass.

## Live checks for the main agent

1. Regen rate: poke the flags byte of a bot's record copy (`BOT_vars+0xac`, block base
   0x26d660 + i*0x780, i = bot slot − 4) to 0x10 mid-match, damage the bot (Drone+0xac
   float), gdbwatch BOT_vars+0x738 (write at 0x16e060:232) — expect one write per 60 frames
   and +5.0 on Drone+0xac per write, clamped at the stats max (BOT_vars+0xa4).
2. 360 vision: clear bit 0x08 → stand behind a bot at < 24u with LOS; Drone+0x170 (opponent)
   should stay 0; set the bit → 0x170 takes the player obj within a tick, and Drone+0x228
   bit 2 (0x4) sets the same frame (no 10-frame reaction delay).
3. Damage perk: set bit 0x02 on a bot's flags byte, let it shoot you, gdbwatch HITDATA+0x08
   between the hit and Player_HandlePain — expect the ×1.25 multiply before the pain call;
   repeat with bit 0x04 + fists for ×1.5.
4. Bot-choose names: walk the MP bot-choose list on the disc build and note each entry's name
   + the stats row it copies (the wheel at 0x210f08 sends the list index to
   BOT_getDefaultStats) — pins the census table's row↔name mapping.

## Usage

not visible to me (no token, credit or context-window figures are shown in this environment;
nothing to copy verbatim).
