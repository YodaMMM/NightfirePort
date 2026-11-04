# WORKLOG

## audit-mp-panes
Started 2026-11-04 21:28 UTC. Branch s/audit-mp-panes (cut from origin/master 8c9f02b03).
Task: sweep MPPaneList and every HUD_MP* update/create function against the decompile and
ee_start_05.bin: each pane's sprites, positions, colours, enable conditions, and what writes
them. Include the per-scenario objective icons, MP_setPlayerStatus's status bits and what
reads them. Fix what is missing or wrong where the draw path exists, pin each with a test,
list anything needing new drawing as a table with the template values.
Work log (audit-mp-panes):
- (21:30-22:10) setup redone (sandbox was fresh): clone, branch s/audit-mp-panes off
  origin/master 8c9f02b03, work/, ISO, ALSA 1.2.11 user prefix (1.2.16 needs glibc 2.43
  symbols this sandbox lacks), queue worktree pending. Swept MPPaneList 0x2be9e0 (22 slots,
  13 live) from the ELF + ee_start_05.bin: [0]=MPAmmoPane 0x2be698 → HUD_CreateAmmoPane
  0x1a0ed8 / update 0x19a7f0; [1]=MPHealthPane 0x2be818 → 0x19b390 / 0x19b630; [4]=
  MPMsgInfoStatusPane 0x2be868 → 0x1a10b8 / 0x19ca88; [6]=SightPane, [9]=RedeemerPane,
  [10]=RCCarPane, [12]=BloodPane, [13]=MPScorePane 0x2be9c0 → 0x19ed10 / 0x19f0a0,
  [14]=RadarPane, [17]=OICWPane, [18]=RoninPane, [19]=LaserPane; Sight/Ronin/Laser share the
  EMPTY update 0x1a12d0 (enable-byte toggles only). HUD_Init 0x199e88 picks MPPaneList when
  MPSettings+0x180 != 0; HUD_Reset 0x1a0d38 leaves 0/1/2/3/4/13/14 enabled; MP_ReSpawn
  0x185368:67 disables the blood pane. Template dumps [L ee_start_05.bin] in the report.
- (22:10-22:45) fixes: the blueprint status bit was the FLAG's — the port set/cleared 0x1 on
  touch/deliver/drop; the game writes 0x2 [M 0x1882d8:81-84,126-129, 0x188970:27-29]. The GE
  item bits 0x4/0x8 were never written; added per MP_GoldenEyeUpdate 0x187568:123-126,231-236.
  drop_carried now clears each dropped object's own bit. MpSlot::status_bits doc carries the
  full map. The MP health pane now hides under the Ronin remote view (pane 18 is in the
  update's hide bytes [M 0x19b630:54-61]) — the port drew the bar over the remote view.
- (22:30-22:45) pins: crates/nf-game/tests/mp_status_bits.rs (4 tests, all pass), the
  Ronin-hide test in mp_hud_real_live.rs (4/4). Per binary (alphabetical cargo order):
  mp_hud_data 4/4, mp_hud_pane 5/5, mp_match_rules 18/18, mp_never_mentioned 6/6,
  mp_status_bits 3/3; nf-view: mp_hud_real_live 4/4, mp_clock_gate_live+mp_bot_objectives_live
  10/10 in one run, mp_scenarios_live 10/10, hud2_live 5/5, hud_radar_live 4/4. Checkpoint
  pushed edc31a4e5.
