
## boot-loadout-qworm — started (in progress)

- 12:05 UTC branch s/boot-loadout-qworm created from origin/master d1b31fd3 (sandbox warm:
  repo, work/, ISO, rust, 5.3G target in place; ee_start_0700000b.bin NOT present). NEXT:
  read the level-0b-teleports-to-walks report + the Q-Worm finding (owned[0x58] load=1 on
  0700000b), find the disc's level-load inventory init [M], audit the port's boot loadout,
  fix every on-foot level's start gadgets/weapons to the disc rows, pin per level with a
  disc-rows test + a fails-on-missing-gadget sister.

- 12:40 the disc mechanism pinned [M]: `Player_InitWeapon` 0x1a7ef8 is a compiled per-level switch —
  `lw a0, 0x3774(v1)` (GameState+0xC) → `sltiu v1, a0, 0x4a` → the 74-word jump table at 0x2FC870 →
  the case arms call `Player_EquipWeapon` 0x1a8a58 with weapon/rounds immediates (the prologue's
  s-regs: s0=Upg(6)=2, s2=Upg(0x4a), s5=Upg(0x43), s6=Upg(0x1e), s7=Upg(0x56), s4=0x5b NTSC, 0(sp)=
  Upg(0x4e) the tail's laser); no loadout table in the level archives — the "rows" ARE the binary
  immediates. The port's `init_level_loadout` (player.rs :2278, fix-weapon-modes 1d8aa5fd +
  fix-start-loadouts 8b70ac2c) already matched — the 0b Q-Worm grant has been in since 1d8aa5fd.
- 12:45 crates/nf-view/tests/loadout_disc_rows_live.rs: a delay-slot-faithful MIPS decoder reads the
  rows straight out of /ACTION.ELF (the dispatch bytes asserted; the boot upgrade tables read from
  the ELF at 0x2bee08..; the case→arm mapping verified entry-by-entry: k=idx-1, k=9 → the 0a/0b arm
  0x1a8418 falling into the 09 kit 0x1a8448, k=0x13 → 14, k=0x49 → 4a with 12/13).
- 12:50 the pins: `every_on_foot_level_boots_with_the_discs_loadout_rows` — 27 on-foot archives
  through the real `nf_view::load`, owned set + switch request == the disc rows through the game's
  own refusal rules [M 0x1a8a58:20 + 0x1a97c8] — 2/0 green; the sister
  `the_discs_gadget_rows_survive_into_the_ports_kit` — every disc gadget row (id ≥ 0x48) asserted
  individually per level against the direct `init_level_loadout` call, the 09/0a/0b Q-Worm row
  hard-lined [M 0x1a8494] — 2/0 green. walkreal_0700000b_live's conditional 0x58 poke replaced by a
  hard boot-loadout assert — 3/0 green.
- 12:55 the targeted runs: loadout_disc_rows_live 2/0, fix_start_loadouts_live 6/0,
  gadget_carry_live 6/0, gadget_select_live 2/0, gadgets_live 4/0, gadget_grapple_live 1/0,
  gadget_laser_live 8/0, gadget_grapple_user_spot 2/0, walkreal_0700000b_live 3/0 — 34 passed,
  0 failed. The fast runner (started 12:07 on the base tree = origin/master d1b31fd3; my diff is
  tests-only) built 984 binaries and is partway through the run; the totals land in the report.
