# agent-music-merged

One `music.rs` with exactly one port of each `UpdateMFX_*` level half. Built on origin/master
with s/agent-musical-events (which carries s/agent-music-machines) merged first, then
s/agent-sound, then the music parts of s/agent-lvl-01-02 and s/agent-lvl-03-04 (their
`music.rs` unit tests and 01's live intro test; their non-music drone/walk changes stay on
their own branches).

## Which version won

| Updater | Level | Kept | Why |
|---|---|---|---|
| `UpdateMFX_02MayhewA` 0x111ed8 | 07000001 | agent-sound's body (fields renamed `may_a_*`) | Both versions match the decompile line for line (:40-170: the 3/500-hold attack arm, the 5 bridge, the 7/9 and 0xb/0xe round-robins, `MusicTemp3` falling every frame). Both gate on `MusicWaitForEnd` [M :40]. No behavioural difference; one copy kept. |
| `UpdateMFX_02MayhewD` 0x112bf8 | 07000004 | agent-sound | The bodies are the same, but agent-lvl-03-04 called it outside the end-wait gate. The game checks `MusicWaitForEnd` after the shared exit block [M :38-40], so on the exit frame the 0x172 trigger must not jump marker 3/4 over the end marker. New test `mayhew_d_stops_on_the_exit_frame` fails with the ungated call, passes gated. |
| `UpdateMFX_03TowerA` 0x113558 | 07000009 | agent-music-machines | Same logic in both; the version already headed for master kept. |
| `UpdateMFX_03TowerB` 0x1137b0 | 0700000a | agent-music-machines | Same logic in both (e[40] zone, `MusicTemp1` latch). |
| `UpdateMFX_PowerStationA1` 0x113c18 | 0700000c | agent-music-machines | Same logic in both; checked against the decompile's flow (:45-92 end jump, :93 stop latch, calm 0xf / stealth holds / 0x14 instant, creep 0x38/0x22). |
| `UpdateMFX_PowerStationA2` 0x113fc0 | 0700000d | agent-music-machines | Same logic in both (0x2b instant + 600 hold, 0x3a latch, trigger not cleared). |
| Attack gate in `UpdateMusicalEvents` 0x114e70 | all | agent-musical-events (`GateWrite::Alarm`, `AttackGate::apply`) | agent-sound's `AlarmRow` made the same writes but missed the restart's re-seat of the level row [M 0x114e70:33-82]; the events version has it. agent-sound's tests now use `Alarm`. |

Kept from agent-sound unchanged: the combat-music triggers and the 2-D objective/hint chimes
(`sound.rs`, `scene.rs`, the drone.rs trigger path) and its unit and live tests.

## Tests
- nf-game `--lib music`: 37 passed, plus `mayhew_d_stops_on_the_exit_frame` (fails with the
  ungated call, passes gated).
- nf-view music binaries, all green: audit_07000004_live, cutscene_skip_audio_live,
  fix_06_alarm_live, fix_party_mission_live, menu_audio_flows, menu_music_flow, menu_sfx_flow,
  music_level_updaters_live, music_machines_live, music_triggers_live,
  music_unported_consumers_live (with 01's intro-end test), musical_events_gate,
  placement_tick_arms_live (83 passed, 6 ignored by their own marks).
- Level walks, all green: walkreal_07000001..06, 09, 0a, 0b, 0c (main, barrelrow, eastroute), 0d.

## Unproven
Nothing new is claimed beyond the decompile reads above. No live capture was taken.
