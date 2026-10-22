# MP bot fell-out-of-map recovery — `BOT_fellOutMap` 0x1259d0

Ported by unported-mp-bots-1 (2026-10-21). Evidence tags: [M] decompile in
`work/decomp/ACTION.ELF/` (`:NN` = line; disasm quoted from `work/elf/ACTION.ELF`),
[S] suggested.

## The game path

- `control_movement_object_handler` 0x1336f8 (the per-tick type-2/drone movement pass)
  → `control_handle_cel_change` 0x133288:71,181 — when a drone-class obj ends up in no
  cel (`build_LinkToRoom` 0x1c9680 fails, `obj+0xFF == 2`), it calls
  `Drone_FellOutMap__FP7obj_tag` 0x13a878. [M]
- `Drone_FellOutMap` 0x13a878:9-19 splits on the MP master copy (`MPSettings` byte 0x184,
  Ghidra decimal-388): 0 (SP) → `obj+0xFE |= 1` (the delete pass consumes the flag), so
  SP drones that leave the map are deleted; non-0 (MP, incl. bot matches) →
  `BOT_fellOutMap(obj)`, so MP bots are teleported home instead. [M]
- `BOT_initSys` 0x1262b0 is NOT part of this path — it is an empty stub
  (`jr $ra; nop`, 8 bytes, verified in the shipped ELF; next symbol
  `BOT_postLoadInit` 0x1262b8) that `MP_Start` 0x183a50:71 calls before its master gate
  on every level load. The real per-bot init is `BOT_init` 0x1249e8. [M]

## `BOT_fellOutMap` 0x1259d0 (348 bytes)

`p1 = obj+0xE0` (the `Drone_tag`), `p2 = *(p1+0xD1C)` (the bot's `BOT_vars` slot,
wired by `BOT_init` 0x1249e8:153). [M]

1. The spawn pick runs under a forced policy: save `MPSettings` byte 0x1C0 (Ghidra
   decimal-448 — the port's `MpSettings::spawn_policy`), set 1 (farthest from the
   enemies), `MP_GetSpawnPoint(team, obj)`, restore. `team` is read fresh from
   `MPSettings + slot*0x30 + 0x20` with `slot = BOT_vars+0x75C`. Disasm
   0x125a1c `lw $s7,0x1c0($s2)` / 0x125a24 `sw $v0,0x1c0($s2)` (`$v0 = 1`) /
   0x125a48 `sw $s7,0x1c0($s2)`. [M]
2. Teleport: `obj+0x30/0x40 := SpawnPoints[idx] pos (+0x00)`,
   `obj+0x50/0x60 := the entry facing (+0x10)` — the decompile's second `Vec_Copy2`
   source `idx*0x30 + 0x2a7360` is the disasm's first source (`SpawnPoints`, ELF symbol
   @0x2a7350) plus 0x10 (0x125a5c `addiu $s0,$s0,0x10`), i.e. the SAME entry's facing,
   not a second table. [M]
3. `Drone+0x480 := 0` — the velocity vector (docs/fields/drone.md:31). [M]
4. `build_LinkToRoom(obj, 0, glb_world)` — re-attach to the world graph. The port has
   no per-bot room link (its `world.rs` cel relink covers the player only); the port's
   `Bond::prev_pos` pin makes the next move pass see a zero segment instead. [M + S]
5. The `BOT_vars` wipe: +0x730 := 0xF9, +0x766 := 9 (`BOTSTATE_getStateType(0xF9)`
   [M 0x1291f0]), +0x734 := 0, +0x760 := 0xFFFF (no nav node), +0x765 := 0xFF (no
   active goal), memset +0xB0 (0x80, the per-player-info block), +0x762 := 0xF9 (the
   state mirror), +0x770 := 0, +0x771 := 0, memset +0x6DC (0x40, the opponent-history
   ring), +0x76C := 0, +0x76D := 0. Field names: docs/fields/bot-vars.md. [M]
6. `NDrone2_SetOpponent(drone, 0)` — the target clears. [M]
7. `Drone_SM_SetState(drone+0x108, 0xF9, 0)` — back to `BotIdle`; the wipe plus the
   fresh spawn is a full goal-machine reset. [M]

The game does NOT touch: the stats block +0xA0..0xAD (the +0xB0 memset starts above
it), the grudge +0x76B, the weapon +0x768, the armour +0x769, the goal slots
themselves. [M 0x1259d0 — none of these offsets appear]

## The port

- `MpMatch::bot_fell_out_map(bot_index, rng)` (crates/nf-game/src/mp/mod.rs) — the
  policy save/force/restore around `MpMatch::get_spawn_point` (the port of
  `MP_GetSpawnPoint` 0x185098), the slot-pos mirror update, then the brain half.
- `BotBrain::fell_out_map(pos, yaw)` (crates/nf-game/src/mp/bots.rs) — the teleport
  (pos/yaw/body/vel), the mapped `BotVars` wipe (+0x730/0x766/0x734/0x760/0x765/0x770/
  0x771), `opponent = None`, and the `set_state(BOT_IDLE)` pending write committed by
  the next tick's dispatch. The +0xB0 block, +0x6DC ring and +0x762 mirror have no port
  state (the port recomputes +0xB0 from the slots every tick) — documented at the wipe.
- The detection caller (`control_handle_cel_change`'s drone branch) is not ported yet —
  queued in docs/systems/uncited-audit.md; this function is the recovery half.

## Tests (crates/nf-game/src/mp/bots.rs, `mod fell_out_tests`)

- `fell_out_map_forces_farthest_spawn_and_wipes_the_goal_machine` — through the real
  match path: policy 0 control pick takes the least-far spawn (the sister that proves
  the forced override is load-bearing); the fell-out pick takes the farthest, the
  policy byte restores, pos/yaw/vel/wipe/grudge/weapon/stats assertions per the [M]
  lines above.
- `fell_out_map_commits_bot_idle_on_the_next_tick_and_spares_other_bots` — the state
  commit through the real `BotBrain::step` dispatch (trace 0xC3 → 0xF9), the +0x766
  type, and the untouched second bot.
