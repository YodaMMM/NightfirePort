# The refused weapon change

`docs/systems/sound-coverage.md` lists `Player_WeaponChange` 0x1a6f80:128: sound 0x1DB when the change is refused on `level_tag+0x62 == 1`, MISSING because the tag byte is unparsed. Branch from current `origin/master`, push to `s/weapon-change-refused`. Find what that byte is and which levels set it, parse it, and port the refusal: does the weapon stay, and does the sound play? Test on a real level that sets it, proved red on the old code. Do not guess at the byte's meaning; cite its reader and the disc value.

---
claimed-by: sandbox7 at 2026-11-06T21:08Z

---
## Report (done, 2026-11-06T22:13Z)


## Report body (the header above lacked it — path fix)
# weapon-change-refused

Branch `s/weapon-change-refused`, cut from origin/master 7370525a4a0310a0e8a3a1250585291202b4d51a.

## What the byte is

The sweep's row for `Player_WeaponChange` 0x1a6f80:128 pinned the sound condition as
`level_tag+0x62 == 1` and left it MISSING ("the tag byte is unparsed"). The byte is parsed — it was
never a level_tag byte. The reader dereferences `*(obj + 0xDC)` — the **AnimObject** pointer
[obj-tag.md +0xDC], whose **+0x62 is the held weapon id** (the id the held model, auto-aim, reload
and ~30 other readers key off [animobject.md +0x62, reader list M*]):

- [M] `work/decomp/ACTION.ELF/001a6f80_Player_WeaponChange__FP7obj_tagsUc.c:48-49`:
  `iVar7 = *(int *)(param_1 + 0xdc); cVar11 = *(char *)(iVar7 + 0x62);`
- The same function writes that byte's neighbourhood as weapon state: +0x63 (the queued weapon) at
  lines 85/113/161-198, +0x64 (the previous weapon) at line 142 — the animobject.md assignments.

So the sound condition is **held weapon id == 1 — the fists**. No level_tag param feeds it
(level_tag is the 0xB0 per-placement parameter block [level-tag.md]; the player is not created from
one), and nothing needed parsing: the row's struct attribution was the bug.

## Which levels set it, and where the refusal fires

- [M] `Player_InitWeapon` 0x1a7ef8 case 0x0700007 (lines 176-186): after the checkpoint RamLoad and
  the loadout equips, `anim[0x63] = 1` — the party level (Mayhew, 07000007) selects the fists, and
  `Player_WeaponSelect` applies it to +0x62. (Case 0x0700008's conditional fall-through into the
  same case is the only other init route to held==1; it has no lock, so it never reaches mode 2.)
- [M] `Player_Weapon` 0x1a24c8:242-275 is the ONLY caller of `Player_WeaponChange` mode 2
  (grep over `work/decomp/ACTION.ELF`): while `GameState == 0x0700007` and `switch_channels[0x48]`
  is OFF, the weapon-cycle actions (0x10/0xF) run `Player_WeaponChange(±1, 2)` instead of mode 0.
- [M] Mode 2 (`0x1a6f80:124-137`, disasm `001a71ec-001a71f0`: bounds a0=2 / s6=0): the cycle is
  bounded to [0,2), so from the fists it re-picks the fists — **the weapon stays** — and the
  held==1 arm plays the refused click [M disasm `001a7200-001a7214`:
  `f12 = 0x42c80000` (100.0f), `a0 = 475` (0x1DB), `jal Sound_Play`]: **one 0x1DB per press**.

The port already modelled the lock's fist-pinning (`weapon_lock_input`, the party tests) but
zeroed the cycle press and skipped the click ("no switch, sound aside").

## The port change

- `crates/nf-game/src/player.rs` — `weapon_lock_input` no longer zeroes `input.weapon_cycle`; the
  direct-selection refusal (`switch_to != 1` swallowed) stands. Doc comment updated.
- `crates/nf-view/src/scene.rs` — the weapon-cycle branch routes a cycle press to the game's mode-2
  change when the lock holds (level 0x0700_0007, ch 0x48 off): held==1 → `audio.play(0x1db)`, no
  switch; held ∉ [1,0x48) (unreachable under the lock) → request the fists, no click — the game's
  [0,2) walk pull-back [0x1a6f80:134-137,162-198]. The GetBestWeapon fallback tail skips a
  mode-2-handled press (the game's mode 2 always resolves to the fists; it never falls to
  `Player_WeaponNone`'s best-weapon scan — without the skip the press pulled the P2K, caught by the
  test).

## Verification

- **Red on the old code**: `the_lock_refuses_the_cycle_with_the_click`
  (crates/nf-view/tests/interact_07000007_live.rs) loads 07000007, waits out the intro, asserts the
  fists + the lock, then presses the weapon cycle through the view loop's real merge
  (`weapon_lock_input` → `step_bond`) — next and prev, four presses. On the pre-fix code it failed
  at press 0: "the refused change plays 0x1db, seen 0". Committed red (fb541a1ad), then fixed.
- **Green after**: the same test passes — each press leaves one more 0x1DB node (the headless Sim
  never advances nodes; `sound_tick` is the render loop's pump), the weapon stays 1, and the direct
  P2K request is still refused.
- Doc rows corrected in `docs/systems/sound-coverage.md`: line 20 (player class now fully PORTED)
  and line 49 (the 0x1DB row: the corrected reader + the party-lock route, PORTED).

### Per-binary test counts (all `--test-threads=1`, slim ISO; only the 3 `disc_golden` failures allowed by the rules)

| Binary | Result |
|---|---|
| nf-game lib | 686 passed, 0 failed, 5 ignored |
| nf-game integration binaries (all) | ok, 0 failed |
| nf-formats + nf-audio | ok except exactly the 3 allowed `disc_golden` |
| interact_07000007_live | 8 passed (incl. the new test) |
| level_carry_live | 8 passed (the swallowed-cycle assert updated to the pass-through contract) |
| jump_landing_sound_live | 7 passed |
| fix_silencer_modes_live / gadget_select_live / walk_07000007_live | 2 / 2 / 2 passed |
| fix_hud_live / fix_player4_live / microcam_wheel_zoom_live | 6 / 2 / 4 passed |
| gondola_windows_live / stealth_ai_live / fix_remote_mine_play | 3 / 10 / 3 passed |
| interact_07000009_live / interactables_0700000a_live | 16 / 13 passed |
| fix_party_mission_live / party_door_lock_07000007_live / party_start_rules_live | 9 / 2 / 6 passed |
| interogate_07_live / mayhew_end_transition_live | 2 / 3 passed |
| walkreal_07000001..07000041 (all 25 level walks + dump) | ok, 0 failed (e.g. 01: 8, 03: 6, 05: 14, 08: 6, 0a: 13, 14: 9, 16: 14, 1b: 17, 41: 15) |
| npc_doors_live / fix_guard_misc_live | 6 / 6 passed |

The binaries above are every test whose files mention `weapon_cycle` / `weapon_lock_input` /
`WEAPON_LOCK` (the symbols touched) plus the whole walkreal level-walk set, npc_doors and
fix_guard_misc. The remaining nf-view live binaries (cutscene/render/mp suites that match none of
the touched symbols) did not fit the window and are owed to the full suite.

## What's left

- The untouched-nf-view-surface suites listed above (cutscene/render/mp probe binaries) still owed
  one full-suite pass on a longer window.
- The sister MISSING row next door (`Player_Update` 0x193600:121, `level_tag+0xA1` music event) is a
  different mechanism and was not touched.

## Live checks for the main agent

- On 07000007 (Mayhew, before the hall plate): watch `anim+0x62` (the player AnimObject's held
  weapon byte) — expect 1 from the loadout on; press the weapon-cycle button: expect **no change**
  (+0x62 stays 1) and the 0x1DB click each press. After TouchOnce #412 (ch 0x48 on): the same press
  should cycle normally (mode 0), no 0x1DB.
- Optional: `watch *(char*)(*(int*)(player_obj+0xDC)+0x62)` during the lock — the write should only
  come from `Player_WeaponSelect` applying +0x63 = 1 at the loadout.

## Usage

not visible to me
