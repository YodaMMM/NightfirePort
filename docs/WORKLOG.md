
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
