# MP death effects (the local player's death presentation in multiplayer)

The user's report (2026-10-19): in an MP match the local player dies and respawns with none of the
SP death presentation. This doc pins what the GAME does on an MP death, what the port already ran,
and the getters the draw code reads. The sister doc for the SP sequence is
`docs/systems/mission.md` §8 + `docs/reports/death-animation-sp.md`; this doc covers the MP side
and the shared pieces.

## The game's MP death sequence

The MP death is the SP death's sim core with the SP-only pieces skipped. `MPSettings._384_4_`
(offset 0x180, the "this is an MP match" word, non-zero for every machine in a match — the port
keeps it as `Drones::mp_master`) is the branch everywhere.

1. **The death tick** — `Player_CheckForDeath` 0x190810, run on health ≤ 0 for the local player:
   - Common to SP and MP [M 0x190810:34-97]: `Car/GunImp/GT_PlayerHasDied`, `Player_ClearInertia`,
     `HUD_Update`, the death marker `obj+0xff = 0x12`, the death stamp `obj+0xec = GameState+0x52`,
     `Player_WeaponNone` (the fists 0x47), cam mode 0 (`Player_SetCamMode(BL, 0)` :64), health 0,
     `obj+0xf4 = 2`, substate 0xd (0xe in a vehicle, ch 0x60 bit 0x100 :89-94), and the death
     sound `Sound_Play3D(100.0, 1.0, 1.0, 0x89, obj+0x30, 0, 0, 0)` :88 — **0x89 plays in MP too**
     (it sits after the SP/MP arms).
   - SP-only [M 0x190810:39-42, the `MPSettings._384_4_ == 0` arm]: ch 0x5e + `Music_Event(6,1)`
     (the death sting). MP raises neither.
   - MP-only [M 0x190810:67-87, the `!= 0` arm]: the kill message — `Txt_BindLabel(0x2000037/38)`
     sprintf'd with the victim's (or killer's) name and `Text_AddMsg(slot, 0, 1, msg, 0, 0xb4)`
     ("X died" / "X was killed by Y" — the port's `kill_msgs`).
   - `MP_PlayerKilled(obj)` :95 (scoring; no-ops the SP path).
2. **The dead ticks** — `Player_Update` 0x193600:316-319 runs only `AnimObjectUpdate` +
   `Player_HandleDeath` while `obj+0xf4` is 2/3: no input, no collision — the first-person eye
   freezes at the death spot (cam mode 0 = `Camera_SetToPlayer` 0x1bc048 → `Player_GetHeadPos`
   0x195aa0; the death-animation-sp report's frozen-eye shape). The same freeze holds in MP.
3. **The blood pane** — `Player_HandleDeath` 0x190bb0, every dead tick:
   - SP arm [M :20-35, `_384_4_ == 0`]: `HUD_DisableAll` + `HUD_Enable(pane 12, 1, 0)` — the whole
     HUD hides and only the blood pane shows; once it is up, pane state non-zero or action 25 (✕)
     raises ch 0x62 (the fail-menu path).
   - **MP arm [M :37-55, `!= 0`]**: `HUD_Enable(pane 12, 1, 0)` **without DisableAll** — the HUD
     stays up under the blood. No ch 0x62, no fail menu. After
     `FRAME_RATE_INT * 5` ticks from the stamp (`(uint)(FRAME_RATE_INT * 5) <= GameState+0x52 -
     obj+0xec`, :41 — 300 ticks at the 60 Hz frame rate) and the scenario-0x10 lives gate
     (:42-48), `MP_ReSpawn` runs (:53) unless `obj+0xf4 == 3`.
   - The game draws **no respawn countdown and no prompt** — the only MP-death text is the kill
     message of step 1; the 5 s wait is silent.
4. **The pane's growth** — `HUD_UpdateBloodPane` 0x19a6a8 per tick while pane 12 is enabled
   (`pane+0x1c`): both blood sprites grow 1 px per tick (`FRAME_RATE_MUL`, :27-30) from
   `viewer+0xf8` toward `viewer+0x110`; when the grown height reaches the target the pane's state
   word `pane+0x1a` is set to 1 (:23-25) — the SP "covered" state. The MP arm (:31-38) caps the
   second sprite's counter at the remaining distance. The cover gate is `w − viewer.w ≥ viewer.h`
   with viewer.h = 448 [M 0x19f4e8:33] and 512-wide templates [L BloodSprInfo 0x2bcfd8] — 448
   ticks to full cover. **In MP the respawn at 300 < 448 ends the sequence first: the pane grows
   to 300 px (62.5% of full cover) and never reaches the covered state.** The pane data: pane 12,
   two sprites, layer 9, textures 0x0300005a / 0x03000002, template colour 0x007f7fff [L
   BloodSprInfo 0x2bcfd8] (the draw itself is the main agent's task).
5. **The respawn** — `MP_ReSpawn` 0x185368 (via the HandleDeath wait): the guard (:16-18 — no
   respawn in SP, after `MPGame+0x188`, or the scenario-0x10 lives-out arm), the policy's spawn
   pick (`MP_GetSpawnPoint` 0x185098, :23), the position copy (:30-54), the slot resets
   (`MPGame[slot+0x10/+0x20/+0x26]`, :55-57), and for a dead body (`obj+0xff` 3/0x12, :65-91):
   **`HUD_Enable(pane 12, 0, 0)` — the blood pane turns OFF** (:67), `Player_SetCamMode(BL, 0)`
   (:68), the view-punch words BL+0x8b0/0x8b4 = 0 (:69-70), health
   `MPSettings[slot*0x30+0x2c] + 100.0` (armor bonus + 100, :71-73), BL+0x160/0x15c/0x8a8 = 0,
   BL+0x8b8 = health (:74-77), `MP_EquipPlayer` (:78 — the weapons come back), the viewer snapped
   to the spawn (`Camera_Set(viewer, obj+0x30, obj+0x50)` :79-80), state 0 / substate 0 /
   `obj+0xff = 3` (:81-83), `Player_StandAtNewPosition` (:87) and a fresh stamp `obj+0xec` (:90).

## What the port ran before this task

The sim side was already game-faithful (`crates/nf-game/src/death.rs`, both MP arms):
`Drones::death_tick` runs the whole sequence for MP too — the pane data (`pane_on`, `ticks`), the
death script's events, the 0x89 sound — and skips exactly the SP-only pieces (ch 0x5e + sting at
`death.rs:214`, ch 0x62 + fail menu at `death.rs:234`). `Mission::update` stays off in MP
(`drone.rs:10976` [M 0x182c60:16-18]). The respawn: offline through `Sim::step_mp_death_respawn`
(scene.rs — `player_killed` scoring, the 300-tick gate, `mp_respawn` → `armor_bonus + 100.0`,
`Bond::stand_at`, `death = Death::default()`); online through `Sim::step_net_combat`'s dead wait
(`mp_dead_ticks`, the same 300 gate, the same respawn + `mp_equip_local`). The kill message and
the frozen camera were in.

What was missing was the VIEW surface: nothing read `death.pane_on/ticks/covered` (the SP red fade
is a documented undrawn gap — `docs/systems/mission.md` §7 known-gaps), so MP had no data to draw
either. This task adds the getters; the main agent draws the overlay.

## The getters (this task's delivery)

`crates/nf-game/src/death.rs` — on `Death` (the game state):
- `phase() -> u8` — 0 none, 1 the sequence is running, 2 covered (pane state word [M 0x19a6a8:24];
  SP-only — MP respawns first).
- `pane_px() -> u32` — the grown height in px: 1 px/tick from the enable [M 0x19a6a8:27-30],
  capped at 448 [M 0x19f4e8:33]; 0 while the pane is off.

`crates/nf-view/src/scene.rs` — `Sim::local_death_fx() -> Option<LocalDeathFx>` (None while
alive), the one call the draw code uses, offline MP, online MP and SP alike:
- `phase`, `pane_px` — as above.
- `fade_alpha: f32` — `pane_px / 448` clamped to 0..1 — the per-tick alpha source [S derived from
  the px rule; the draw maps it onto the pane templates].
- `hud_hidden: bool` — true only in SP (the DisableAll [M 0x190bb0:24]); MP keeps the HUD up
  [M :39].
- `respawn_ticks_left: Option<u32>` — MP only: the ticks left of the 5 s gate [M 0x190bb0:41-54];
  None in SP (SP waits for the cover/✕ instead).
- `camera_frozen: bool` — the frozen first-person eye [M 0x190810:64]; the pose source is
  `bond.head()` (the SP test `dead_bond_holds_the_death_camera_spot` pins the bit-freeze).

Clock note (online): the port's online death starts one tick after the kill tick (`mp_dead_ticks`
is set in `step_net_combat`, the local health mirror + `Drones::tick` run after it), so the getter
reads the pane clock from `mp_dead_ticks[slot] - 1` online (elapsed since the kill tick) and from
`death.ticks` offline (elapsed since the death tick) — both are the game's
`GameState+0x52 - obj+0xec` elapsed. The 300-tick respawn gate itself is unchanged (pre-existing,
both arms; changing it would move the pinned `mp_perf_16` digest for no game reason).

## Tests

`crates/nf-view/tests/mp_death_effects_live.rs` (ISO-gated):
1. Offline MP: the local player's death runs the phases with the game's numbers — pane 0 px on the
   death tick, +1 px/tick [M 0x19a6a8:27-30], hud up (MP arm [M 0x190bb0:39]), the 300-tick gate
   [M 0x190bb0:41], the respawn wipes the sequence (pane off [M 0x185368:67]) and re-arms the
   loadout; still dead at 299.
2. Online MP, two machines: the kill enters through the shared hurt path on both machines; every
   tick's `net_state_hash` agrees across the death and the respawn (the other machine's view of
   that player is unchanged — the hash sister), and the dying machine's getters run the same
   phases.
3. SP sister: the same getters on an SP death — hud_hidden true, no respawn gate, the growth
   reaches the 448 cover and phase 2 (what MP never reaches [M 0x190bb0:41 vs RED_FADE_TICKS]).
