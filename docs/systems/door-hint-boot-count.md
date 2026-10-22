# door-hint-boot-count — the door hint's 3-per-BOOT counter

The aim-ray door hint ("Press Action to open door", label `0x200000b`) is capped at THREE shows per
power-on across every level. The counter is boot-scoped process memory, not level state and not
saved-game state. Docs first, then the port shape, then the tests.

## The game's rule

- `Check_Target` 0x1a7810 (the aim-ray tick, every 6th frame) — the class-0x1c (door) hit arm:
  the hint shows only while the .sdata global `DoorMsg` 0x30cd04 < 3, and every SHOWN hint
  increments it [M disasm 0x1a7b88 `lw r2, 0x8694(gp)` — the `DoorMsg < 3` gate; 0x1a7ba8
  `lw` + 0x1a7bc0 `sw` — the `DoorMsg + 1`; decompile 0x1a7810:186-191]. The gate short-circuits
  BEFORE the `Door_DoMsg` call, so an attempt the cap blocks never reaches the door and burns
  nothing.
- The full gp-relative scan (this task, over the whole ACTION.ELF, gp 0x314670 → immediate
  0x8694): exactly those THREE references exist, and no absolute `lui`-based reference to
  0x0030cd04 — no other reader, no other writer, no reset anywhere in the on-foot ELF
  [M the scan; decompile grep: `DoorMsg` appears only in Check_Target].
- `DoorMsg`'s image bytes in the ELF are `00 00 00 00` [D ACTION.ELF file offset 0x206c04] — a
  boot (a NEW process) starts at 0, and since the only writer is the +1, only a boot resets it.
- What does NOT reset it: a level load, the pause-menu Restart (`C_GCPAUSE_Handler` 0x204408:139 →
  `ResetMap_LevelToLoad(BaseMap, 0, 1)` — a plain cold level load), the campaign transition, a
  warm restart (`P_ENDMISSION_Handler` 0x216618:19-21). All of them are level loads; `.sdata`
  is process memory and outlives them all.
- It is NOT in any save: `Player_RamSave` 0x195670 saves the gameplay buffers (health, switch,
  inventory, channels); `.sdata` globals are not part of it [M].
- `Door_DoMsg` 0x135f18:15-33: the per-door `+0x74` latch (docs/fields/door.md) sets to 1 BEFORE
  any check — one attempt per door ever, even a locked door burns it [M 0x135f18:16-17] — then the
  lock-channel gate, then the state gate (state 4 = OPEN fails; grouped doors read
  `DoorGroupsStates[g]`), then `Txt_BindLabel(0x200000b, 0)`. The latch lives on the door's ext
  record, which the level rebuilds on every load — it is per-level in the game, and per-level in
  the port (`Doors::list[i].msg_shown`), by design on both sides.
- DRIVING.ELF has no `DoorMsg` symbol [D readelf] — the cap is on-foot only.

## The port

- The count lives in the per-level Sim's HUD: `HudState::door_msg` (crates/nf-view/src/scene.rs),
  with `load_msg` (`LoadMsg` 0x30cd08, the exit-hint cooldown — same boot scoping) beside it.
- The scene-to-scene handoff carries them: `HudState::carry_boot_globals` — called from
  `App::swap_in_scene` (the ONE scene-replacement path level loads, both Restart arms and the
  campaign transition take), the weapon-asset warm reload, and the MP late-join replay. No
  process-global static: tests stay isolated, and a fresh front-end session (the port's new
  process equivalent) starts at 0.
- It rides no save: `RamSave` carries health/switch/inventory/channels/mission only.
- The hint gate + increment: `scene.rs` hud_tick's class-0x1c arm — dist < 5.0 [M 0x1a7b6c-84],
  level != 0x700004a [M 0x1a7b58-64], `door_msg < 3` [M 0x1a7b88], then `Doors::do_msg`
  [M 0x135f18] → on a label, `door_msg += 1` + the 180-tick INFO message [M 0x1a7b98-0x1a7bc8].
- The per-door latch: `Doors::do_msg` (crates/nf-game/src/door.rs) sets `msg_shown` before the
  lock/state checks, exactly the game's order [M 0x135f18:15-33].
- The campaign transition survives in the port's real shape: a movie-wrapped load HOLDS the load
  request over the still-live gameplay scene while the briefing plays (lib.rs `pending_level`
  hold), and the phase-2 resolve re-issues it with the old scene still current — so the swap into
  the next level is one carrying swap (old sim Some → new sim Some), and the warm `RamSave` rides
  the same handoff. The mission-to-mission flow through the front end (results → mission list) is
  the port's session boundary — a new session starts at 0, the sanctioned "NEW process" case.

## Tests

- `crates/nf-view/tests/door_hint_boot_count_live.rs` (NF_ISO-gated): the real-path test — three
  hints burned through the aim ray on 07000001 doors 0/1/2, the next level (07000002) loaded
  through the app's shape (`load_with_carry` + the swap's carry line), a fresh closed unlocked
  door hints nothing and its latch stays clear; sister: a fresh session of the same level shows
  one. Plus the HUD-layer carry sister.
- `crates/nf-view/tests/door_text_labels_live.rs` (from door-text-labels): the latch, the locked
  door's burn, the open-door failure, the per-door one-shot, and the cap gate.

## Live checks for the main agent

1. Memory-watch `0x30cd04` (EE, data breakpoint) through play: expect +1 only on shown hints,
   no write at a level transition, none at the pause-menu Restart, none on death-retry.
2. Fresh boot → aim at three distinct closed unlocked doors (3 hints), then a fourth door → no
   hint — and after loading the next level, a fresh closed unlocked door still hints nothing.
   (The [M]/[D] chain above pins this; the live check confirms it end-to-end on hardware.)
