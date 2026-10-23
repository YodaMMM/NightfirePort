# mp-shot-gate-reload: the shared gate's auto-reload follows the game's trigger rule

The shared shot gate (`MpCombat::gate_tick`, docs/systems/mp-shot-gate.md) carries the reload
state machine that blocks a slot's shots while that slot reloads. 700y2
(docs/systems/mp-shot-effects.md) caught the gate and the shooter's own weapon
pipeline disagreeing at a clip boundary. This page pins the game's auto-reload rule and the
reload timing with addresses, and states the rule the gate (and the local pipeline) follow.

## The game's auto-reload rule ([M] `Player_WeaponFiring` 0x1a5460)

The auto-reload arm sits in the trigger's release path. The disassembly chain:

1. **The action bits** ([M] `Input_Update` 0x17ee88, disasm 0x17f084-0x17f100): the per-action
   state byte at `0x2a39cc + action + slot·0x158` is rewritten from the polled pad every frame —
   a fresh press writes `0x05` (bits `0x01 | 0x04`), a held press writes `0x01`, a release
   writes `0x00` (`daddu $a0, $zero, $zero` at 0x17f100). So in `Input_Action(slot, action,
   mask)` **mask 1 (bit 0x01) is the held LEVEL** (set on the press frame and every held frame,
   cleared on release) and **mask 4 (bit 0x04) is the fresh-press EDGE** (the press frame only —
   the next held frame overwrites the byte with `0x01`).
2. **The fire path vs the release path** ([M] 0x1a5460 disasm 0x1a5eb0/0x1a5fe8/0x1a6330):
   `Input_Action(slot, 9, 1)` (the level) nonzero → the fire path; zero → the reload-key check
   `Input_Action(slot, 0xe, 4)` (the fresh press) and then the release path, which re-reads
   `Input_Action(slot, 9, 1)` and branches **on zero to the auto-reload block at 0x1a61d0**.
3. **The auto-reload conditions** ([M] 0x1a61d0-0x1a6266): the trigger's level is 0, the dry
   `Player_RoundToFire(player, +0x91, spend=0)` fails (the clip cannot feed the `+0x91` need —
   0x1a18e0 checks `clip < (short)need` and spends only with the third argument), the view-anim
   state is 0 (`+0xf4`, disasm 0x1a61ec-0x1a61f4) and the switch state is not 10
   (0x1a61fc-0x1a6204), and `Player_ReloadAmmoType(player, 1)` succeeds.
   `Player_ReloadAmmoType` 0x1a1988 (called with `param_2 = 1`, the check mode — no spend):
   the carried rounds for the `+0x90` ammo type (`BLData+0x170 + type·2`) are > 0 AND the clip
   (`+0x1b8 + slot·0xc`, the twin's base slot per `+0x05`) has room under `+0x92` — clamped to
   1 shell per call for the one-at-a-time flag `+0x68 & 0x100`.
4. **On success** ([M] 0x1a6244-0x1a6266): the refire timer `+0x92c` zeroes, the shots-in-pull
   `+0x936` rearms to `weapon_data+0x28 + mode·2`, the view state `+0xf4` = 6 — or 5 when the
   reload-start script `+0xa8` exists (the script queues at 1.0); without a start script the
   reload script `+0xa4` queues at 1.35 (`AnimScriptAddSpeed 0x3faccccd`, disasm 0x1a6270+), or
   the `+0x68 & 0x200` twin cycle scripts `+0xac`/`+0xa4` alternate through `BLData+0x958`.
5. **The trigger must STAY released until the reload starts — holding the trigger with an empty
   clip reloads nothing.** The arm is re-evaluated every tick the level reads 0; a held trigger
   (level 1) takes the fire path, whose `Player_RoundToFire` spend simply fails every tick.
   Cross-check: `Player_SetWeaponAnimObj` 0x1a2e38 case 6 (disasm ~0x1a3600, decomp :360) reads
   `Input_Action(slot, 9, 1)` for the shell-loop interrupt — nonzero (held) aborts the reload to
   state 7 — the same mask used as "held". The mask-1-is-level reading is the only one under
   which the fresh-press re-arm (`Input_Action(9,4)` → 0x1a6358: the timer expired rearms
   `+0x92c = 1.0`, `+0x936 = shots_per_pull`) stays a per-press event; the caller
   `Player_Update` 0x193600:413 always passes `param_2 = 0`, so there is no per-tick re-arm.

## The reload timing

The reload is the view anim: state 5 plays `+0xa8` (start) once, state 6 plays `+0xa4` (the
shell loop — one refill per script stop through `Player_ReloadAmmoType(_, 0)`, the game's own
case-6 arm) and non-shell records go straight to idle at the first `+0xa4` stop (LAB_001a3678:
the `+0xa0` idle script, state 0, `Input_ClearAction(9)` — the cleared action byte makes a
still-held physical trigger read as a fresh press the tick after the reload, which re-fires).
State 7 (`+0xac`, the end script) exists only for records carrying it (the SPAS family [L
ee_start_05.bin]). The 1.35x speed on the auto-reload's `+0xa4` (no start script, `0x200`
clear) and the flag-0x1000 "from frame 6" head make the P2K's reload
`ceil((len − 6)/1.35)` ≈ 111 ticks — the port's `gun::reload_ticks` model, shared verbatim by
the gate (the same `GunParams`, the same `script_len`), so both machines count identically. The
script-player stop rule is not read from the decompile; ±1 tick is [S] and never asserted.

## The port's rule (this fix)

Both the gate and the local pipeline take the auto-reload arm from the trigger's **level**,
never from the edge:

- `MpCombat::gate_tick`: the arm runs when `!fire` (the wire's fire byte — the live loop fills
  it from the same trigger level the pipeline sees, `lib.rs` `fire: self.trigger`), the shared
  clip cannot feed the `+0x91` need, and `Player_ReloadAmmoType`'s gates hold (the carried > 0,
  the clip has room) [M 0x1a5fe8 → 0x1a61d0, 0x1a1988]. The old `!fire || fire_prev` (a
  ¬fresh-press) reloaded while the trigger was HELD — the game does not (the level must read 0).
- `Gun::update_with` (the shooter's own pipeline): the same fix, `!pressed` — the pipeline and
  the shared gate must agree or the boundary loses legitimate shots (the pipeline reloading
  while held would re-fire through the state-6 clear into a shared clip the gate never refilled).
- The manual reload key keeps its own arm (the wire's `reload` lane is the game's fresh-press
  edge, `Input_Action(0xe, 4)` [M 0x1a5ec8]) and does not zero the timer (branch A leaves
  `+0x92c`, disasm 0x1a5f50-0x1a5f84); the auto arm zeroes it [M 0x1a6244].
- The gate has no view-anim state to gate the arm on (the view is per-machine, the gate is
  shared): the stand-in is no check at all. Harmless for the filter — a shot description only
  exists when the sender's pipeline actually fired, and that pipeline blocks its own fires while
  its reload runs; the gate starting its reload a few ticks earlier (no busy wait) only ever
  means the shared clip is ready before a description can arrive. [S — port design, the PS2 had
  no online play; the tests pin the observable behaviour.]

## Tests

`crates/nf-view/tests/mp_shot_gate_live.rs` (NF_ISO-gated, two machines per test, the hashes
must agree every tick — the sister):

- `p2k_through_two_clips_keeps_every_shot_and_agrees` — the natural spawn loadout (nothing
  topped: the equip's `clip_size·2` = 7 clip + 7 carried [M mp_equip_player]), a 2-of-12 tap
  cadence, the shot description riding the next tick's row exactly as the live session's
  `net_shot_pending` does. Both clips' worth (14) fire and are accepted, none lost at the
  boundary, no two accepted closer than the 8-tick interval [D +0x40], the 7th→8th gap spans the
  ~111-tick reload, and the shared state ends clip 0 / carried 0.
- `held_trigger_through_the_boundary_reloads_only_on_release` — the repro: the 7th tap's press
  never releases (the trigger held ~300 ticks through the boundary). The game's rule says the
  held trigger reloads nothing and fires nothing [M 0x1a5fe8]; the old rule (¬fresh-press)
  reloaded while held and re-fired through the state-6 clear mid-hold. After one released tick
  both machines reload, the still-held trigger re-fires through the state-6 clear
  [M 0x1a3678], and the remaining taps bring the total to 14 with nothing lost.
- `rocket_spam_fires_at_the_launcher_rate_and_stops_when_dry` (existing, the sister family) — a
  rocket-every-tick sender still fires only at the real rate.
