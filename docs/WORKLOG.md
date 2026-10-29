## level-15-every-objective (2026-10-29)

Fresh sandbox (reset). Re-verified the level's mission data byte-for-byte from the disc ELF:
MissionData vaddr 0x2a4350 (PT_LOAD vaddr 0x107100 → file 0x1000; row 21 file offset 0x19e250),
row 21 = level 0x07000015 mission 0x07000014 part 1, obj table @ 0x2a42d8, **count 1**: obj 0
text 0x0400004d, fail 0xffffffff, hint 0x04000054, ch 0x8e, init 0, enable 0x4b, flags 0 —
byte-identical to crates/nf-game/src/mission/c07000015.rs. 07000016 (row 22) lists 2 objectives
(ch 0x8f/0x90, enables 0x8e/0x8f). Mission_MonitorObjectives 0x1827d8 re-read: states 0/1/2/3/5,
complete fold 3 → 4 on the record channel (case 3), alarm fail switch_channels[0x61].

walkreal_07000015_live.rs already covers the single listed objective end-to-end (announce on the
spawn reveal plate 0x4b → walk the silo → USE exit #445 → carried channels → 07000016 spawn plate
#252 fires 0x8e → all states[21] == 4). Added the [D] row-21 citation and a
states[21].len() == 1 shape pin. Suite + report to follow.

Suite (this session): build clean (exit 0); 235/888 binaries recorded 825 passed / 2 failed
(fix_0c_pylon_live ×2 = wgpu adapter NotFound, no GPU in sandbox) / 34 ignored; net/mp binaries
that false-failed under parallel contention all pass in isolation. walkreal_07000015_live green
(1 passed, 11.1 s). Report: docs/reports/level-15-every-objective.md.
