# boot-loadout-qworm — report

Task: level-0b-teleports-to-walks found the disc's level load owns the Q-Worm (`owned[0x58]`,
load=1) on 0700000b while the port's boot loadout was believed to miss it. Find how the game
builds the level-start inventory, fix the port so every on-foot level starts with the disc's
gadgets and weapons, and pin it per level with a test that reads the disc rows (plus a sister
that fails on a missing gadget).

## What's done

1. **The mechanism, from the binary [M].** The disc has no loadout table in the level archives —
   the level-start inventory is `Player_InitWeapon` (0x1a7ef8), a compiled per-level switch:
   `lw a0, 0x3774(v1)` reads the level id (GameState+0xC @ 0x2a3774), `sltiu v1, a0, 0x4a` bounds
   it, and the 74-word jump table at **0x2FC870** (`lui v0, 0x30` + `addiu v0, v0, -0x3790` +
   `sll v1, a0, 2` + `jr a0` @ 001a8284) dispatches to the case arms. Each arm calls
   `Player_EquipWeapon` (0x1a8a58) with the weapon/rounds as immediates; the prologue precomputes
   the upgrade picks into s-registers (s0 = Upgrade_Weapon(6) = 2, s2 = Upg(0x4a), s5 = Upg(0x43),
   s6 = Upg(0x1e), s7 = Upg(0x56), s4 = 0x5b on the 60 Hz NTSC build — the `movz s4, v0, a0`
   PAL-50 Hz pick at 001a8250 does not fire on this disc [D identity]), spills Upg(0x4e) to 0(sp)
   for the shared tail, and the tail equips Upg(0x4e) @999 = 0x4e and the fists 1 @0 [M
   001a7ef8:318-319]. "load=1" has no per-row flag: on 0700000b the owned[0x58]=1 measurement is
   the 09 kit's `EquipWeapon(BL, 0x58, 0)` at **001a8494** (the 0a/0b arms RamLoad, then fall into
   the 09 kit at 0x1a8448 when the handgun is unowned — the cold-boot path). The boot upgrade
   tables were read from the ELF bytes [D]: UpgradedHandguns 0x2bee08 = [2,4,6,8], Lasers
   0x2bee18 = [0x4e,...], Snipers 0x2bee28 = [0x1e,...], Tasers 0x2bee48, PDAs 0x2bee58,
   DartGuns 0x2bee68, Upgrades 0x2beea8 = 0 [L ee_start_05.bin].
   The table indexing was verified entry-by-entry: entry k serves the decompile's case
   0x700000(k+1) (k=9 → the 0a/0b arm 0x1a8418; k=0x13 → the fresh 14 arm 0x1a8618; k=0x49 →
   case 0x700004a sharing 12/13's arm 0x1a84dc; the 15/16/17 grouping matches k=0x14/15/16 →
   0x1a85e8), reconciling the decompile's `case 0x7000015:` labels with the raw table.
2. **The port was already correct — no source change needed.** `init_level_loadout`
   (crates/nf-game/src/player.rs:2278, landed in fix-weapon-modes 1d8aa5fd "the island loadout
   grants the Q-Worm" + fix-start-loadouts 8b70ac2c) models the whole switch 1:1, Q-Worm
   included. The task's premise ("the port's boot loadout misses it") was stale: the walkreal_0b
   conditional poke beside the shot was a dead-in-practice safety net, never the live path. What
   was missing was the per-level proof — this task pins it against the disc bytes.
3. **The disc-rows pin: crates/nf-view/tests/loadout_disc_rows_live.rs.** A delay-slot-faithful
   MIPS abstract decoder reads the rows straight out of `/ACTION.ELF` on the disc (the ISO's own
   ELF, hash-pinned build [D disc.rs ACTION_ELF_SHA1]): it walks the prologue for the s-register
   map, reads the jump table per level, and collects every `Player_EquipWeapon` immediate on the
   cold path (branches explore both arms — the owned-gate's fall-through IS the cold kit; the
   warm path jumps straight to the tail), plus the held-weapon writes (`sb ?, 0x63`) and the
   wipes. The dispatch bytes are asserted before decoding (a build drift fails loudly).
   - `every_on_foot_level_boots_with_the_discs_loadout_rows` — boots every on-foot archive
     (27, from FILES.BIN via `matrix::on_foot_levels`) through the real `nf_view::load` and
     asserts the owned set and the switch request equal the disc rows folded through the game's
     own rules: the equip refusal when the first-person model is not loaded [M 0x1a8a58:20] and
     the `Player_CheckWeaponsLoaded` revocation [M 0x1a97c8]. The per-level skin refusals the
     port shows are the disc's own censuses (15 lacks 0x50 where 14 has it, 16 lacks 0x56,
     0700000c/0d refuse the listed 0x58 because the Q-Worm fp skin 0x050005b rides only
     09/0a/0b [D archive scan] — qworm-not-on-0c's finding, now reproduced from the rows).
   - The sister `the_discs_gadget_rows_survive_into_the_ports_kit` — calls
     `init_level_loadout` directly per level with the refusal disabled (`loaded = |_| true`, the
     raw kit list, the dev panel's levelkit call) and asserts EACH disc gadget row (id ≥ 0x48)
     individually, naming the level and the gadget on failure; the 09/0a/0b Q-Worm row is
     hard-lined. This is the test that fails on a missing gadget.
4. **walkreal_0700000b_live strengthened:** the conditional 0x58 poke before the SCOMP shot is
   now a hard boot-loadout assert (`owned[0x58]` must be true at boot, cited to 001a8494) — a
   loadout regression fails the walkthrough instead of silently granting.

## Verification (the runs I made, this tree)

- `loadout_disc_rows_live`: **2 passed, 0 failed** (both tests above; 27 levels each).
- `walkreal_0700000b_live`: **3 passed, 0 failed** (the fresh walk boots owning 0x58 and the
  Q-Worm install lands through the real trigger/bullet/PCQWorm path).
- `fix_start_loadouts_live` 6/0, `gadget_carry_live` 6/0, `gadget_select_live` 2/0,
  `gadgets_live` 4/0, `gadget_grapple_live` 1/0, `gadget_laser_live` 8/0,
  `gadget_grapple_user_spot` 2/0, `fix_weapon_modes_live` 6/0 (incl.
  `the_island_loadout_grants_the_q_worm`), `qworm_not_on_0c_live` 3/0, `level_carry_live` 8/0
  (incl. `cold_07000005_loadout_matches_the_snapshot`), `fix_dev_panel_live` 6/0 (incl.
  `level_kit_grants_the_games_loadout`), `portal_live` 4/0 (incl.
  `cold_loadout_arms_bond_on_every_story_level`). Targeted total: **61 passed, 0 failed**
  across 14 binaries.
- No *inventory*-named binary exists; the inventory surface is covered by the above
  (fix_start_loadouts' warm-carry test + gadget_carry's loadout rows). nf-game itself: the
  crate's lib unit tests (the gadget families, the pickup/inventory arithmetic, the draw
  machines) — **588 passed, 0 failed, 5 ignored**; the crate's ~90 integration binaries ride
  the fast runner below.
- The fast runner (`tools/test/suite.py`, started 12:07 UTC on the base tree = origin/master
  d1b31fd3; my diff is tests-only, so the pre-existing binaries' results carry over): `build
  ok: 984 test binaries in 1166 s`, then the sweep — at the time of writing 6 FAILED lines, all
  the expected environment set: 3 `disc_golden` + `identity_disc` (the slim-ISO cross-check
  dumps) and 2 `fix_0c_pylon_live` (headless GPU) — the same set previous sessions saw. The
  full 984-binary sweep does not fit the sandbox window; the runner stays alive writing
  suite.out and the next message can read its total line.

## What's left

- The suite's final total line (hours; partial as above — the runner is still writing suite.out).
- Optional [L] confirmation of the decode on real hardware: the live checks below.

## Live checks for the main agent

1. Fresh-boot 0700000b (no save-state resume), read the player BLData's owned byte for the
   Q-Worm: `obj+0xE0` → BL, then **BL+0x5DA** (= 0x1BA + 0x58·0xC [M docs/fields/bldata.md;
   Player_EquipWeapon 0x1a8a58's owned store]). Expected: 1 (the disc's 09-kit grant).
2. Fresh-boot 0700000c, same address. Expected: 0 (the equip is listed but refused — the fp
   skin 0x050005b is not in 0700000c's containers [D archive scan; M 0x1a8a58:20]).
3. Fresh-boot 0700000b, read the held-weapon byte the arms write (`sb reg, 0x63(a1)` with
   a1 = *(s3+0xdc) — the control record — e.g. `sb s5, 99(v1)` at 001a84d8): expected 0x43 (the
   dart gun); and BLData+0x934 (the persisted held weapon the RamLoad arm restores): expected
   0x43 after the kit runs.

## Usage

- The session's own input/output token totals, context used and cost: not visible to me.
- The two research subagents' Task-tool results printed their usage (read from their return
  messages): the game-side inventory research — `Tokens -> prompt:167182 completion:5010
  total:172192`; the port-side loadout research — `Tokens -> prompt:123012 completion:6548
  total:129560`. No other credit/context figures are shown in this environment.
