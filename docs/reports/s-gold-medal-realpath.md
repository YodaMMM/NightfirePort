# s/gold-medal-realpath: gold-medal weapon upgrades on the menu path

No game logic changed. No change to the shared random stream.

## What was checked

The G-029 upgrades (`Upgrade_Weapon`, `Menu_SetBonus`, docs/systems/pickups.md) reach Bond's kit when a
codename with gold medals is loaded from the memory card and a mission is started from the menus. The
headless `--shot` path was not used for this.

- Card: `nf_game::profile` writes `<config dir>/<NAME>.profile` (PLRS/MSSN/MPSG/GSET/CHET/BNUS chain).
  `NF_CONFIG_DIR` overrides the config dir (`settings::config_dir`), so every run uses a fresh temp dir.
- Medals: `Menu_SetLevelBonus` 0x1ffa10 [M] sets reward counters `medal..1` and then calls `Menu_SetBonus`.
  Gold is medal 3, and counter 3 is the kind-4 descriptor of each `RewardsTable` row. The throwaway codename
  GOLDALL has gold on all 12 missions: bonus word `0x7f000f7ffd24964f`, upgrade row `[3, 1, 1, 2, 1, 1, 1, 1, 1]`.
- Menu walk: title, Main, Select Codename (row 0 New, row 1 Play without saving, row 2+ card), the card
  row, Difficulty, then the mission list. The list opens on the highest unlocked row, and the wheel wraps.

## Results

Real exe (`NightfirePort.exe --game`, built with movies), driven by `NF_MENU_SCRIPT`:

    loadout: level 07000005 switch request 8 (6 owned weapons, health 100, armour 0) owned [0x1 0x8 0x4c 0x4f 0x50 0x54] upgrades [3, 1, 1, 2, 1, 1, 1, 1, 1]
    [spot] saved .\nf-shots\001-07000005.png (07000005 --bond 104.95,24.97,108.83,-2.147,0.000 frame 240)

The capture shows The Exchange's start with the gold handgun drawn, its laser sight on, and a 16-32 ammo
readout. The base kit is handgun 0x2, stunner 0x4a and laser 0x4e. Here the kit holds 0x8, 0x4c and 0x4f.

In-process menu walk (`gold_medal_realpath_live`, the same `FrontEnd` actions and the same `load_front_end`
the window uses):

- 07000005: request 0x8, owned [1, 8, 4c, 4f, 50, 54].
- 0700000c: request 0x22 (upgraded sniper), owned [1, 8, 22, 4c, 4f, 50, 54].
- 07000009: request 0x44 (upgraded dart gun), owned [1, 8, 44, 4c, 4f, 50, 57, 58, 5b] (decryptor 0x57).
- Control, Play without saving on the same card: request 0x2, owned [1, 2, 4a, 4e, 50, 54].

Fail check: with the level load's upgrade row forced to zero, `gold_codename_walk_upgrades_the_kit` fails
(request 2, expected 8). After restoring the load code, it passes.

## Changes

- `crates/nf-view/src/lib.rs`: the `loadout:` log line also prints the owned ids and the upgrade row. The
  line still starts with the same prefix, so the smoke greps still match.
- `crates/nf-view/tests/gold_medal_realpath_live.rs`: two menu-walk tests that run with NF_ISO. It also has
  two ignored tests: `write_gold_profile` (`NF_GOLD_DIR`), the card writer the script uses, and
  `real_exe_gold_kit` (`NF_REAL_EXE`), which runs the exe in a window.
- Not in the repo: the repeatable runner is `audits/tools-wip/gold-medal-realpath.sh <exe>`.

## Unproven

- The 0x54 zoom doubling (row[2]) was not checked on this path.
- Only one difficulty was walked: the wheel's first row.
- The PS2 was not compared live for this check (no PCSX2 run).
