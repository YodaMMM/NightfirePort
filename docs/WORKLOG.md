# WORKLOG

## weapon-change-refused

Branch cut from origin/master 7370525a4a0310a0e8a3a1250585291202b4d51a. Task: parse the
`level_tag+0x62` byte that gates the refused weapon change (sound 0x1DB,
Player_WeaponChange 0x1a6f80:128 per docs/systems/sound-coverage.md, currently MISSING),
find which levels set it, port the refusal (weapon stays, 0x1DB plays), test on a real
level that sets it, proved red on the old code. No guessing at the byte's meaning: cite
its reader and the disc value.
Progress: setup reused from an earlier session (repo, work/, ISO, nf-queue, cargo present);
branch created fresh from origin/master 7370525a4a0310a0e8a3a1250585291202b4d51a. Started
21:15 UTC; deadline ~23:11 UTC, no new work 22:51.
Outcome: the byte is parsed — it was never a level_tag byte. The reader at 0x1a6f80:127
dereferences `obj+0xDC` — the AnimObject — whose +0x62 is the held weapon id [M obj-tag.md
+0xDC; animobject.md +0x62]; the sweep's `level_tag+0x62` misattributed the struct. The
refusal is the party lock's mode-2 change: `Player_Weapon` 0x1a24c8:265-275 reads the
weapon-cycle actions into `Player_WeaponChange(±1, 2)` (the only mode-2 call sites) while
07000007's ch 0x48 holds; the mode-2 cycle is bounded to [0,2) (disasm 001a71ec-001a71f0),
so from the fists (id 1, `Player_InitWeapon` 0x1a7ef8:184 `anim[0x63] = 1` — the only level
case that selects them) the weapon stays, and the held==1 arm plays `Sound_Play(100.0,
0x1db, 0, 0)` per press (disasm 001a7200-001a7214). Ported: `weapon_lock_input` no longer
zeroes the cycle press; the Sim routes it (scene.rs mode-2 arm, skip the GetBestWeapon
fallback), the click plays, the direct-selection refusal stands. Red proof:
the_lock_refuses_the_cycle_with_the_click failed on the old code at press 0 ("seen 0"),
green after; level_carry_live's swallowed-cycle assert updated to the pass-through contract.
Doc rows: sound-coverage 20 (PORTED) + 49 (the corrected citation). Targeted binaries all
green (interact_07000007 8, level_carry 8, jump_landing_sound 7, fix_silencer_modes 2,
gadget_select 2, walk_07000007 2, fix_hud 6, fix_player4 2, microcam_wheel_zoom 4,
gondola_windows 3, stealth_ai 10, fix_remote_mine_play 3, interact_07000009 16,
interactables_0700000a 13, fix_party_mission 9, party_door_lock 2, party_start_rules 6,
interogate_07 2, mayhew_end_transition 3). Full suite: see the report.
Suite (22:12 UTC): nf-game lib 686 passed / 0 failed / 5 ignored; every nf-game integration
binary ok; nf-formats + nf-audio ok except exactly the 3 allowed disc_golden failures (slim
ISO); all 25 walkreal_07000* level walks + walkreal_dump + npc_doors_live (6) +
fix_guard_misc_live (6) ok; every symbol-matched binary (weapon_cycle / weapon_lock_input /
WEAPON_LOCK grep) ok. The remaining nf-view live binaries (cutscene/render/mp probe suites
that match none of the touched symbols) did not fit the window — owed to the full suite.
