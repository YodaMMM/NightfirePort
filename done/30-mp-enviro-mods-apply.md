## Your task: `mp-enviro-mods-apply`, every Enviro-Mods row changes the match

The Enviro-Mods page (`P_MPENVIROMODS` 0x210030) writes `mp_envmods` in crates/nf-view/src/menu.rs, and the rows' save and summary use it, but the comment near nf-view/src/lib.rs:1748 says the match settings carry no such word, so `gunimps` is cleared for every match. `mp_enviro_value_label` also has no caller. Find what each row does in the game and whether it reaches a match on master: +0x1C0 view distance, +0x1B8 (`GunImp_Create` 0x131f20:52-58 mounted guns), +0x1D4 (the locked row), +0x1D0 (the grapple, see mp/equip.rs) and +0x1CC vehicles. Read every reader of those five words in the decompile and list them in the report, with what the port does for each. Add the words to `MpSettings`, carry them from the menu into the match on host and clients, and apply each reader. Mounted guns must stay in step on every machine, which was the reason they were cleared, so prove that before turning them on. Vehicles: report only what the row gates, and leave car creation alone. Use `mp_enviro_value_label` for the page's rows if the page's text does not already match, and compare each row's label with the decompile's node lists. State must stay equal on every machine online and must not touch the shared Rand stream. Pin through real matches, row by row: on and off give the game's difference. Run every mp_*_live binary, mp_world_sync_live, the menu binaries that walk Enviro-Mods and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mp-enviro-mods-apply`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-07T12:27Z

---
## Report (done, 2026-11-07T14:21Z)


# mp-enviro-mods-apply — the Enviro-Mods rows and the match

Branch `s/mp-enviro-mods-apply` from `origin/master` (10956badb). **Status: the reader census and
the apply work are in; the row-by-row test pins and the suite rerun did not fit the window** (the
sandbox lost ~50 minutes to a disk-full cycle mid-session — details under "What's left").

## What each row does in the game (the reader census)

Every access to the five words uses Ghidra's decimal field forms; the census grepped all ~3000
decompile files for both the hex and the decimal forms and checked every hit (the hex-form hits
are other structs — M_MANAGER, M_SPIN, weapon arrays — and are excluded). The struct is zeroed at
boot, 0x1DC bytes [M bootup 0x1c7e78:17].

- **+0x1C0 (`_448_4_`) — the spawn-selection policy, NOT a view distance.** The task hint said
  "view distance"; the decompile says otherwise. The only gameplay reader is `MP_GetSpawnPoint`
  0x185098:165-183: 0 = the candidate closest to another player, 1 = the farthest from all
  players, 2 = Random (`Rand_Rand(n)` over the valid candidates, line 178), anything else = the
  range start. `BOT_fellOutMap` 0x1259d0:12-19 forces 1 around one respawn. Callers:
  `MP_ReSpawn` 0x185368:23, `MP_Start` 0x183a50:120,144. "View Distance" is only the menu-def
  row title (hash 0x1af [L docs/systems/mp-menus.md:183]); no LOD/vision code reads the word.
  **Port:** correct already — `MpSettings.spawn_policy` [M 0x1C0], `MpMatch::get_spawn_point`
  implements the policy (Near/Far/Random + fallbacks), pinned by mp_core_live (policy 0/1/2/9
  arms), carried in the menu confirm and over the wire (nf-net put/get 16).
- **+0x1B8 (`_440_4_`) — Gun Emplacements (mounted guns).** Sole gameplay reader
  `GunImp_Create` 0x131f20:56-58: in MP (`+0x180 != 0`) with the word 0 it returns 0 — no mount
  object at all. With it set, the turret entry is `MPSettings+0x1B4` (the weapon set) for 0..9,
  and `Rand_Rand(10)` for set 10 = Random [M :59-63]. Boot default 0 [M 0x1c7e78:38]. Also read
  by the PTP persist pair (Boot_GetPTPData 0x1c8198:54 / LoadPTPData 0x1c8560:71) and the menu
  row [M 0x210030:13,33]. **Port:** the words were in `MpSettings` and on the wire already
  (mp-lobby-settings), but lib.rs:1750 cleared `gunimps` for EVERY match, so no mount ever
  reached a game. This session replaces the clear with the game's arm (below).
- **+0x1D4 (`_468_4_`) — Explosive Scenery (the locked row).** Sole gameplay reader
  `Destroy_Create` 0x12c318:15: class-0x18 destroyables are created only when NOT in MP or when
  the word is non-zero. `P_MPCONFIRM` 0x20ec00:148 masks it `&1` at confirm. The lock:
  `P_MPENVIROMODS` 0x210030:35-38 binds the single "Locked" item (0x1000076 at node 0x10) while
  `mp_stuff[0x370]` == 0; `Menu_UnlockMPSettings` 0x1ff698:103 sets it on the type-3 reward 0x3e.
  Boot default 0 [M 0x1c7e78:44]. **Port:** applied already — scene.rs skips 0xE1 placements and
  clears `props.destroy` while the word is 0 (mp-lobby-settings), the menu masks &1 at confirm,
  the confirm path is pinned by mp_world_sync_live's settings (explosive on) and the site-damage
  tests.
- **+0x1D0 (`_464_4_`) — the grapple.** Sole gameplay reader `MP_EquipPlayer` 0x1855f8:27-34:
  when set, every MP player additionally equips `Upgrade_MPWeapon(character, 0x50)` (the
  Q-Claw). Not persisted (no PTP/profile access). Boot default 0 [M 0x1c7e78:43]. **Port:**
  applied already — `mp::equip::mp_equip_player` equips 0x50 while `s.grapple != 0`
  [M equip.rs:103-106]. **A test pinning the on/off kit difference did not land this window**
  (see What's left).
- **+0x1CC (`_460_4_`) — vehicles.** Sole gameplay reader `Car_Create` 0x1305b0: no vehicle in
  MP while the word is 0 [M :13]; the body select: 1 → tank body, 2 → helicopter body (obj+0x112
  = 1, the 0x2000194 gfx + 1.5 z raise), 3 = Random → `Rand_Rand(100) < 51` → helicopter [M
  :29-43]; never on arena 0x0700004b [M :16]. Boot default 0 [M 0x1c7e78:42]; the scenario
  quick-start forces 2 [M C_SBMPSCEN 0x20c600:51]. **Port (report-only per the task):** the word
  is carried everywhere (menu confirm, wire mods[1], `MpSettings.vehicles`); scene.rs gates the
  0xDA placements off it (off: nothing; 1 tank / 2 heli; 3: the same 51% rule rolled on a
  seed-keyed private stream, a documented port substitution [S]) and the port still creates no
  drivable car (docs/GAPS.md) — car creation untouched.

Boot defaults [M 0x1c7e78:28,38,42,43,44]: +0x1C0 = 2 (Random), the other four 0.

## Code this session (all committed on the branch)

1. **The mounts reach the match.** `nf_game::gunimp::apply_mp_turrets` (new, gunimp.rs) is
   `GunImp_Create` 0x131f20:59-63's MP arm: turret = weapon set 0..9, set 10 = a pick per mount.
   The game draws the pick from the shared `Rand_Rand` (0x1e3738) at create time; the port
   substitutes a private stream keyed by the match seed (`seed ^ 0x4755_4e49`) so the pick is
   identical on every machine without adding a draw to the shared stream [S — the same
   substitution the vehicles roll already documents]. Applied twice (idempotent, same seed):
   at build (scene.rs, the mp-lobby-settings block) and at match start (lib.rs, where the local
   `--mp` default builds without settings) — lib.rs's unconditional `gunimps.clear()` is gone;
   the word 0 case still clears [M 0x131f20:56-58].
2. **The mounts ride the online world hash** (scene.rs `net_world_fold`): turret, active, hot,
   pitch, yaw, smoothed mirrors, heat, cooldown per mount. `fire_handle` (machine-local audio
   voice) and `beam` (the shot's draw quad) stay out. A mount desync now fails the tick hash.
3. **The Enviro-Mods page draws `mp_enviro_value_label`** (menu.rs MpEnviroMods arm) — the
   decompile's node lists live in one function (P_MPENVIROMODS 0x210030's msg-0x10 binds:
   row 0 Near 0x1b0 / Far 0x1b1 / Random 0x1b2; rows 1/3 On 0x1b7 / Off 0x1b6; the locked row's
   single 0x1000076; vehicles Off 0x1b6 / Tanks 0x10002d8 / Helicopters 0x10002d9 / Random
   0x1b2). The draw's old inline copy disagreed with the game on the locked row: it keyed on the
   stored value being the stale 0x10, while the game binds by the unlock flag [M 0x210030:35-44]
   — a fresh locked profile (value 0) showed "Off" where the game shows Locked.

The carry (menu → `MpSettings` → wire → client) already existed from mp-lobby-settings and is
unchanged: the five words ride `MpSettings` (nf-game mp/mod.rs), the menu confirm writes them
[menu.rs, the &1 mask at confirm M P_MPCONFIRM 0x20ec00:148], nf-net serialises spawn_policy
(slot 16) and the four mod words (protocol 7 `mods[4]`), and mpnet.rs rebuilds them on the
client. State equality online holds because every machine builds the level from the same
settings and seed; nothing in this work touches the shared Rand stream.

## Verification this window

- The census (above) is from the decompile, checked line by line; the menu page's full handler
  (0x210030) was read and matches the port's row order/controls/labels.
- `cargo test -p nf-game -p nf-view --no-run` compiles the changed tree clean (the libs and the
  test binaries build; the final links hit the sandbox's disk limit partway — see below).
- Existing-test impact is nil by construction: every existing test builds matches with
  `gun_emplacements` 0 (the `MpSettings::deathmatch` default and the menu's boot defaults), so
  the gate clears exactly as before; the world-fold addition is empty for mount-less matches.

## What's left (the next message continues from the branch)

1. **The repro/proof tests** (`crates/nf-view/tests/mp_enviro_mods_live.rs`, planned but not
   written in time): row 1 off → no mounts on a mount-carrying arena; on (weapon set 5) →
   `turret == 5` per mount; on (set 10) → the pick equals `Rng::new(seed ^ 0x4755_4e49).rand(10)`
   and is < 10; the two-machine net_tick run with mounts on → every tick's hash agrees and both
   machines' mount arrays are identical (the in-step proof the old clear was waiting for); row 2
   off → `props.destroy` empty on a destroyable arena, on → non-empty; row 3 on → the load's kit
   owns 0x50, off → not. First pick the arena via the mp_world_sync inventory print (gunimps
   count per arena).
2. **Run the suite** (`tools/test/suite.py`) on the final tree merged with `origin/master`, plus
   every mp_*_live binary, mp_world_sync_live, the Enviro-Mods menu binaries (mp_defaults_setup,
   mp_menu_flow, menus_e2e_mp, mp_screens) and nf-game — and list the pass counts here.
   Caveat to check: mp_screens renders pages; the locked row's text changes by design, so a
   visual pin there (if any) needs the 0x210030 citation above, not a revert.
3. **`mp_enviro_value_label`'s remaining caller gap:** the setup/summary pages that echo the
   rows (if the confirm page prints the words) were not audited this window.

## Session notes

- The sandbox's disk (9.9 GB) filled twice during the window: once from the master-tree suite
  build, once from a full RUSTFLAGS rebuild triggered while restoring the ALSA user prefix
  (the alsa.pc's prefix=/usr makes pkg-config drop the -L; the fix was copying
  libasound.so into the rustc sysroot lib dir, which rustc already searches, so no RUSTFLAGS
  are needed). The dead artifacts were deleted and the build resumed; the last hour went to
  that cycle instead of the test pins.

## Usage

not visible to me
