
## ssys-wire

Branch `s/ssys-wire` from `origin/master` (22d5836b7). Task: wire the security
subsystems (SSys) into play. crates/nf-game/src/ssys.rs ports `SSys_Init`
0x1b01b0, `SSys_Monitor` 0x1ad998, `SSys_Msg` 0x1ad878 and `SSys_RamLoad`
0x1b0208, but nothing outside the file calls them, so the 240 s laser/camera
countdowns, their announcements and their channel writes never run. Confirm
that, then wire all four as the game does: `ResetMap_GameInit` 0x1d5080 calls
Init (respect `IsWarmReset`), `Game_Run` 0x1c9790 calls Monitor after the
`GS_IsPaused` gate and only while `ScriptCam == 0`, `Player_RamLoad` 0x1957a8
calls RamLoad. Port `Switch_Create` 0x1adcc0:29-31 (channel store when
placement +0x44 non-zero) and census every level for switches with non-zero
+0x44 [D]. Check the save side: what writes the SSYS block RamLoad reads, keep
it through checkpoint and Death-and-Replay. Msg sounds through the normal
sound path with level gates. Keep the game's tick order. Pin through the real
path on a census level (trip → announcements at 180/120/60/6 s → channel
drops at 0 → reactions). Entry started — work follows.

Progress (same session): confirmed `Ssys` had zero callers outside ssys.rs [M grep]. Wired
all four: `Switch_Create` 0x1adcc0:29-31's channel store (param 6 = `level_tag`+0x44 →
`SSysItems[idx].channel`, in `UseObjects::create` + `attach_level_id`, before the load
beat), `SSys_Monitor` 0x1ad998 into `tick_inner` at `Game_Run` 0x1c9790:32's slot (after
`Env_Update`, before the object-handler passes, `ScriptCam == 0` [M 0x1ad998:8]), the
`SSys_Msg` announcements through `Sounds::play` with the level gates, and `SSys_RamLoad`
0x1b0208 through `RamSave::ssys` (+ `with_ssys`) on `warm_exit_save` /
`entry_checkpoint_save` / the death serve, applied in `load_with_carry` (the 0a item-2
skip keeps the carried record; the death serve swaps the live record into the block's
record-2 slot). Disassembly checks: the monitor's channel writes carry NO
`switch_channels_time` stamp [M 0x1adaa0-0x1adacc, 0x1adc64] — the laser-2 arm writes the
array directly. Census [D]: only two switches on the disc carry param 6 — 07000009 #71
(item 1, ch 0x37) and 0700000a #643 (item 2, ch 0x3b, gate 0x3c); the 0a load snapshot
proves the store live [L ee_start_0700000a @0x2befd0: item 2 = {−1.0, 0x3b, 0}]. Real-path
pin on 09 passes: trip → arm 240 → announcements at 180/120/60/6 → drop at 240 s → the
show's AMBER reaction. Tests green (see the report). Commit abdd3b82a.
