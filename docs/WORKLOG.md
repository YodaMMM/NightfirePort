# WORKLOG

## sound-gates-sweep

Branch cut from origin/master d6245281a. Task: sweep every PORTED row of
docs/systems/sound-coverage.md — for each, read the branch conditions around the call in the
decompile (work/decomp/ACTION.ELF), name the flag or state that reaches it, and confirm the
port fires on that same event; trace flags to their writers instead of trusting the row's
label. The `Player_HandleJump` 0x1916b0:191-198 head-bump/landing mislabel (fixed by
head-bump-surface-sound) is the pattern to hunt: right id, wrong event, so an id-level test
passes while the game plays nothing there.
Plan: six parallel audits over the table's PORTED rows (player, weapons/bullets, world objects,
drones/music/script), each row read against the decompile branch and the port's gating code;
claimed WRONGs re-verified in disassembly (capstone MIPS64, scripts/mipsdis.py) before any
change; each fix lands with a test that drives the event through the real input path; the doc
rows corrected with the citation. Deliverable: one MATCH/WRONG line per row appended to this
entry, fixes + tests, report in docs/reports/sound-gates-sweep.md.
Progress: setup verified (work/, ISO, nf-queue present from an earlier session; branch created
fresh from master d6245281a). Sweep started 12:59 UTC; deadline ~14:54 UTC, no new work 14:34.
Outcome: all 48 PORTED rows swept against the decompile + disasm (six parallel audits, claimed
WRONGs re-verified by hand). 42 MATCH (with label/citation corrections folded into the doc),
6 WRONG: (1) one-way door +0x64 clunk mirrored — the game's dot is (door−activator)·facing
[M 0x135be0:71-74], fixed + `one_way_door_refuses_from_the_game_side`; (2) the 0x148 special-
death cry fired on state ENTER at the drone — case 5 is ANIM_DONE at the PLAYER (0x163f38),
moved to the ANIM_DONE arm + test rewritten; (3) the hostage 0x10 music cue's `music_reqs`
outbox had no drain — drained in Drones::tick + fail-first test; (4) the substate-change loop
stops (0x12/0x3D/0xF0, [1..3]∪[5..0x10], not 0/4) are NOT ported — left with the condition
pinned; (5) the 0x1DB refused jump is unported (doc said PORTED) — relabelled MISSING,
condition pinned; (6) 0x5EF belongs to Oddjob's hat (base 0x45), not grenades — relabelled
MISSING with the [L] weight-0 audibility open question. Doc labels fixed: Bullet_init:273
(guards pass 0xffff), 0x447 (satchel beep), 0x27C (tripbomb arm), 0x16/0x14, 0x5DD, searchlight,
MusicTrigger (channel-gated repeater, worldobj.rs), 0x286, casings 0x1FD, picklock 0x5DB/0x5DA
now PORTED, [BL+0x898] proven dead code (ELF-wide scan). Tests: nf-game lib 689 passed (2
pre-existing mp::bots failures verified on a clean stash), sound_coverage_live 5/5,
ai_missing_states_live 10/10; the full suite (walkreal_*_live, npc_doors_live,
fix_guard_misc_live) did not fit the window and is listed in the report as owed. Report:
docs/reports/sound-gates-sweep.md.
