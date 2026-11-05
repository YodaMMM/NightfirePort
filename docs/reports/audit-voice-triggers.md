# audit-voice-triggers — every caller of the drone voice path, and whether the port plays it

Branch `s/audit-voice-triggers` (cut from origin/master f503c15c7). Proof level 07000005. Tags: [M] code
(function + address, read in `work/decomp/ACTION.ELF/`), [D] disc, [L] EE snapshot / real ELF tables, [S] suggested.

## Method

A parallel research pass enumerated every caller of `NDrone2_SpeechSFX` 0x141b00,
`NDrone2_SpeechSFXd` 0x149d58 and `NDrone2_PlaySFX` 0x141848 across all 3039 decompiled ACTION.ELF files:
**36 SpeechSFX call sites + 1 SpeechSFXd + 10 direct PlaySFX sites in 33 caller files**. Each was read with its
surrounding state-machine arm, and every helper (PainTalk, DeathTalk, …) was traced one level up to the real
trigger. The port's side was audited by reading `crates/nf-game/src/drone.rs` (the `talks` request stream,
the per-state arms) plus `spawners.rs`, `mp/bots.rs`, `breakable.rs`. The ground truth for the line tables is
`Drone_SfxInfo` 0x2812d0 read back from ACTION.ELF by `nf_formats::sfx::voice_lines` (346 × 32 B) [D ELF];
its type-2 rows were re-pinned in the new live test against speech.md §3 [L].

## Fixed this pass (4 triggers, 5 call sites, all in `crates/nf-game/src/drone.rs`)

| trigger | game | what the port was missing | fix |
|---|---|---|---|
| flee from a scary object (grenade/blast) — `NDrone2_DSTATE_RunAwayFromObject` 0x86 ENTER | `NDrone2_DeathTalk` between the `+0x50c = 1.0` write and the 0x400000 set [M 0x168aa8:19-22] | the ENTER had no bark at all | `self.death_talk(d)` after `vision_on` |
| `NDrone2_DSTATE_UnderCoverFire` 0x96 ENTER | `DroneWeap_Fire(1)` then `NDrone2_AttackTalk` [M 0x16a3e0:31-33] | no bark | `attack_talk` after `fire` |
| `NDrone2_DSTATE_UnderCoverSniperFire` 0x97 ENTER | `NDrone2_AttackTalk` right after the 0x10000 set [M 0x16a610:19-20] | no bark | `attack_talk` after `vision_on` |
| `NDrone2_DSTATE_PlayScript` 0x03, start scripts 0x6000168/0x6000169 | ENTER tail: SetScript → `NDrone2_AttackTalk` → `+= 0x400000` [M 0x15ba68:69-74]; msg 3 re-barks EVERY update once `now > +0x11c + 0x1e` [M :107-121]; msg 5 barks after the hold anim [M :150-157] | none of the three; the port's own ANIM_DONE comment even names the AttackTalk it never called | ENTER bark + new field `play_script_talk_gate` (ENTER sets `frame + 30`, the port's stand-in for the game's state-dwell stamp `Drone+0x11c`) gating a per-update re-bark; msg-5 bark in the 0x168/0x169 arm |

All other script arms are silent in the game and stay silent in the port: 0x60005e6/0x60008ae (LAB_0015bb74) and
the default arm (LAB_0015bb50) return after their SetScript, and 0x60007ac (LAB_0015bb9c) skips the shared tail [M 0x15ba68:34-74].

### Pins

- `crates/nf-game/src/drone.rs` (`#[cfg(test)]`, all driving the real state-machine delivery `Drone.send`):
  - `runaway_enter_barks_the_death_line` — ENTER on a type-2 drone requests `(5, forced)` [M 0x168aa8:21 +
    the 0x13eff8 type→line map]; the same-line dedupe holds on the second call [M 0x13eff8:75-85].
  - `undercover_fire_enters_bark_attack_talk` — both 0x96/0x97 ENTERs request 0xAC + Rand(7) (the deterministic
    type-3 voice arm, odds 0 [M 0x13eac0 case 3]).
  - `play_script_talks_on_enter_update_and_done_for_the_start_scripts` — ENTER bark, the gate value (enter+30),
    no re-bark at frame == gate, re-bark past it, the 0x60007ac ENTER silence, and the msg-5 bark.
- `crates/nf-game/tests/voice_triggers_live.rs` (real level data, real ELF tables):
  - `the_type2_voice_rows_match_the_table` — the real `Drone_SfxInfo` rows the barks resolve to: 5 → sfxId 139 /
    repeat 30; 0x13 → 138; 0x37..0x3f are stream ids 175..198 at repeat 180; 0x49..0x4e at repeat 0 [D ELF].
  - `a_frag_blast_flees_a_guard_with_his_death_bark` — a frag (`0x34`) queued through the real tick drain
    (`HandleExplosives` 0x1435f0 → msg 0x21 → `DroneFunc_ConsiderExplosive` 0x1431c0) flees a real 07000005
    guard (props read from the real tables) into 0x86 and the ENTER requests the death line 5, forced.

### Test runs (release, `--test-threads=1`, NF_ISO=slim)

- `nf-game` full: **1066 passed / 0 failed** over 104 binaries (lib 665 + 5 ignored — includes the 3 new pins).
- `nf-formats`: 92 lib + 11 disc_golden passed; the 3 allowed slim-ISO `disc_golden` failures (bigf_header,
  bigf_tables, directory_tree).
- `nf-audio`: 28 passed (lib 23, bank_live 1, fix_sound_remainder_live 3, music_live 1).
- Every test binary that touches the changed code, individually: `ai_live` 37, `grenade_live` 4,
  `vision_bit_census` 1, `voice_triggers_live` 2, `guard_barks_07000013_live` 4, `guard_first_shot_01_live` 2,
  `stuck_guard_combat_05_live` 2, `escort_follow_07000001_live` 1, `start_corridor_guard_07000011_live` 8,
  `corridor_guards_07000012_live` 1, `fields_actors_live` 1, `guard_stuck_sweep_live` 1 — all green.
- Note: `mp::bots::fell_out_*` (2 tests) fail under the DEBUG profile on the clean branch too (verified by
  `git stash`); under the mandated release run they pass.
- The `nf-view` full-package release run did not fit this session's window (its ~90 test binaries rebuilt for
  ~10 min without reaching the run phase); the 9 `nf-view` walkers that exercise the changed code all ran green
  individually (counts above).

## Verified already correct (no change needed) — the caller inventory vs the port

| voice fn / caller @addr | trigger | line (expr) | port plays it? |
|---|---|---|---|
| PainTalk 0x13eeb8 ← HitDamage 0x145558 (surviving bullet), PunchImpact 0x145f28, ExplosiveImpact 0x146480 ×2, msg 6/8 arms of the 16 stun/surrender/ninja states | hurt, alive | type map → 0x12/0x13/0x11/0x14/0x18/0x15/0x1b/0x1c/0x10/0x16, default 0xf/0x17, force 1 [M] | YES — `pain_talk` at the damage paths + stunned/surrender arms [M drone.rs 3646/3666/3748/3860/4004] |
| HitTalkExtra 0x13ed38 (PainTalk type 0x10 only) | extra hit-react set | 5..=7 on 14-16 → 0x133+Rand(5); 0x10 → 0xec+Rand(7) [M] | YES — `hit_talk_extra` (the 3-s throttle + Rand(5) included) |
| DeathTalk 0x13eff8 ← OnInitDeath 0x145bd0 (×2, second dedupes), LocationDeathAnim 0x13c780, RunAwayFromObject ENTER | death | type map; 5/6 on 0c/0d special [M] | YES — `death_talk` at `location_death_anim` + both `handle_death` sites (head-shot `+0x3a` carve-out incl.) |
| AttackTalk 0x13eac0 ← FirstAttack 0x148010:94, AimStandFire 0x166328:51, CombatNoMove 0x1643e8:25, SniperAim 0x15fc18:21, SniperFire 0x15fd88 (msg 3, every frame), AbseilSlide 0x169730:31 / AbseilHang 0x169940:30 (msg 3), UnderCoverFire 0x16a3e0:33, UnderCoverSniperFire 0x16a610:22, PlayScript 0x15ba68:72/:120/:157 | combat entries/updates | 1→0x33+Rand(4) (0xa3+Rand(8) on 11/0b/13); 2→0x37+Rand(9) odds 3; 3→0xac+Rand(7); 5/6, 7, 0x11 level sets [M] | YES after this pass (all sites above verified in drone.rs: 7619/7641/8106/8250/10144/10190 + the four fixed ones) |
| SeenPlayerTalk 0x13f1d0 ← FirstSightState 0x147c48:106, CombatNewSighting 0x173280:23 ENTER | first sight | 05-08 type 2 → 0x49+Rand(6), else 0x21; odds Rand(3) [M] | YES — `seen_player_talk` |
| PatrolTalk 0x13e988 ← DSTATE_Patrol msg 3 | idle patrol chatter | type 3 → 0xb3+Rand(5); global cooldown 0x298fc4; consumes Rand(2) on other types [M] | YES — `patrol_talk` (the cooldown/RNG-consumption fold included) |
| HeardNoiseTalk 0x149920 ← HeardNoiseAlert 0x16b540:23 / Suspect 0x16b0e0:24 ENTER | heard something | 0x26, facial, type 1 only, gated on the global 0x298f84 [M] | YES — `heard_noise_talk` at both ENTERs |
| NewSight/LostSightTalk 0x13f420/0x13f550 ← DroneVision_ProcessDroneSight 0x176a28:208/234 | sight lost/regained, one-shot globals | 5/6 on 0c/0d: 0xd6+Rand(2) / 0xd3+Rand(3) [M] | YES — driven from `Drones::tick` (19938/19948) with the one-shot latches |
| mp_chatter (InjuredTalk 0x13fa20 / RunForCoverTalk 0x13fd08 / UnderCoverTalk 0x13fe80 / UnderCoverAimTalk 0x13fb98) | MP-bot + undercover barks (MP-only / SP-only per the MPSettings gate [M]) | level sets 0xcc..0x138 | YES — `mp_chatter_talk` at the four port sites (3974, 6200, 6230/6376, 6283); the MP-only/SP-only split kept via the MP gate in the callers |
| HandleTalking 0x13f688 ← ControlSTANDARD 0x1490f0:201 | the AFO walkie dialogue | 0x9b/0x9c/0x9d cycle, 0x9e one-shot [M] | YES — `handle_talking` from the tick (19952), gates + throttle + steps pinned by lib tests |
| PowerstationTalk 0x13e770 (HandleTalking on 0c/0d) | mission-console chatter | table walk of `PowerSfx` [M] | YES — `powerstation_talk`, table pinned |
| MayhewTalk 0x149818 ← ReachedDestNode 0x153358:44-65 (route flags 0x10000/0x40000/0x20000/0x80000) | Mayhew's mission barks | 0x44/0x46/0x45/0x47 [M] | YES — `route_reached_special` pushes all four |
| BondTalk 0x1446b0 ← Civilian/CivilianMission(Wait)/PartyGirl/CivilianGuard msg 3 (07000007, facing+dist+unarmed gates) | civilians chatting at Bond | per-type sets, DoBondTalk for 0xe, 7-step jump table for 0x12 [M] | YES — `bond_talk` + `do_bond_talk` + `party_talk_step` (jump table pinned) |
| TalkToMissionObj 0x14a8b8 ← Hostage/HostageKiller msg 0x1a; MonitorTalkToMissionObj 0x144340 is a msg pump, not a voice caller | mission-scripted speech | type 4 → 0x29+Rand(6); 5 → 0x2f+Rand(4) [M] | YES — `talk_to_mission_obj` on both MISSION_TALK arms; `monitor_talk_to_mission_obj` pumps the msg (the double-bark trap avoided) |
| CivilianScaredTalk 0x13f900 ← CivilianScared 0x15d3a0:30 ENTER / HostageGoToGoalPosition 0x165a68:19 | civilian panic | 0x9f+Rand(3) on 09-13/4a then 0x48, facial [M] | YES — `civilian_scared_talk` at both sites |
| CivilianChallenge 0x162558 ENTER | armed civilian challenge | 0xc3+Rand(9) [M] | YES — drone.rs 9085-9093 |
| CivilianDoorGuard 0x160648 (zone 2 / armed zone 3) | door-guard barks | ChooseSFX(0x4f,0x50) / (0x8f,0x92) [M] | YES — `handle_door_guard` 4867-4876 |
| Interogate 0x161a40 msg 3 (weapon drawn → PlaySFX 0x4a enforceDelay 1; closed-in → SpeechSFX 0x4a facial) + msg 0xc L7 step 0 (0x51) | interrogation | 0x4a / 0x51 [M] | YES — `foley_reqs.push(0x4a)` + the facial request + the L7 step (8487/8503) |
| OpenDoor 0x1681f8:117 / KickObject 0x1684c0:51 msg 0x20 | door/kick foley | 0x27 / 0x28 [M] | YES — the `pending_door_msg` drain (state-dependent 0x27/0x28) |
| SeenSurrenderedDrone 0x16ade0:24 ENTER (odds Rand(6)) | spotting a surrendering bot | 0x23 [M] | N/A — entry 0x23 has sfxId 0: the game plays silence [L ee_snapshot]; the port delivers the 0x9f transition in mp/bots.rs |
| HostageDie 0x1725a0:13 ENTER | hostage death scream | 10 [M] | YES — `foley_reqs.push(10)` |
| CivilianPatrol 0x15d610:60 msg 3 | female scream before the alarm run | 0x48 [M] | YES — `foley_reqs.push(0x48)` on the female arm |
| DroneSpawner_Control 0x137e68:200 | wave-cleared taunt (0c/0d spawners) | 0xd6+Rand(2) [M] | YES — `spawners.rs:179` (`sfx_request`) |
| Drone_BrokenObject 0x138588 ← Break_Kill 0x12b970:135 | guards barking at a player-broken script-type-5 object (07000001 only) | ChooseSFX(0x40,0x43) = 0x40+Rand(4) [M 0x1497e8] | **NO — not wired** (see the table below) |

## Left as a table — missing or unverifiable, with the reason

| item | game | port status | why not fixed here |
|---|---|---|---|
| `Drone_BrokenObject` 0x138588 guard bark (0x40+Rand(4), 07000001, player-broken script-type-5 object, victims type 0x0c alive) | the Break_Kill tail | `breakable.rs` documents the call but never barks | the bark needs a breakable→Drones cross-module call (scan the drone list for type-0x0c victims near the kill); a clean site exists (`Breaks` kill path → `Drones`) but it is new plumbing, not a one-arm fix — next task can wire it and pin on 07000001 glass |
| MP bot voices: `BOT_handlePain` 0x125ed8 (survived → 0x154/0x157 + Rand(3); died → 0x152/0x14d, gated on `Sound_Playing_Ref(+0x73c)`) and `BOT_soundEffect` 0x126618 (trigger 1/0x1e/0x1f from DealWithObjHit 0x13fff0:336, BotDeathByExplosion 0x1755a8:15, BotDeathAnim 0x175508:13) | MP bot pain/death barks | `bot_sound_effect` exists with exact line pools + tests (bots.rs 4751-4755) but has **no production caller**; `body.sfx_out` is only cleared, never pushed | the wiring point is the MP bot damage/death flow in `mp/bots.rs` (where the death states are driven); needs that flow read end-to-end first — listed with the exact pools so the next task can wire + pin |
| Interogate msg 0xc, level 0x7000005 scripted sequence: step 0 → anim 0x21 + SpeechSFX(0x49,0,1,1) + 337-tick timer; step 1 → 0x4d + 60; step 4 → 0x4e + 45; step 5 → Attack [M 0x161a40:211-244] | the scripted interrogation | the port's non-0x7 branch runs the generic other-level arm (anim 10/0xb/10, no speech) | the game itself can never reach Interogate 0x3c on 07000005: FirstSightState returns 0x3c only on 0x7000007/0x7000041 [M 0x147c48:92-100] and no other SetState producer targets 0x3c on that level — dead code on disc; the port matches the reachable behaviour. Pin the sequence only if a live trace shows the state entered |
| `PlayScript`'s 0x400000 deaf-flag: the port's msg::SOUND arm treats 0x6000169 + 0x60007ac as deaf [M drone.rs comment], while the decomp puts the shared `+= 0x400000` on the 0x6000168/0x6000169 tail (0x60007ac's arm jumps past it) [M 0x15ba68:69-74] | which scripted guards hear gunfire | possibly inverted for 0x168 vs 0x7ac | not a voice trigger (no line); needs a disasm pass over 0x15bb50-0x15bba0 before touching the sound arm — flagged for the main agent |
| BondTalk case 0x12's jump table (0x1449dc) — the party-girl sequencer | up to 6 hidden sub-branches | ported as `party_talk_step` with the 6 steps pinned against the lui constants | the decompiler could not statically resolve the table; the port's step table matches every resolvable constant — a live trace of 07000007 party chatter would confirm the order |

## Live checks for the main agent

1. **RunAwayFromObject death bark** [M 0x168aa8:21]: in PCSX2, lob a frag near a 07000011/07000005 guard with
   the flee props; watch the guard's `Drone+0x4D0` (the voice handle) and `Drone_SfxInfo[5].lastFrame`
   (0x2812d0 + 5*0x20 + 8) — write when the flee ENTER fires. Expected: lastFrame jumps to `now` on the flee
   frame, and the line is 139 (death pool), not a pain line.
2. **PlayScript ENTER/UPDATE/ANIM_DONE barks** [M 0x15ba68:72/:120/:157]: break-point `NDrone2_AttackTalk`
   0x13eac0 on 07000001 while the start guard runs script 0x6000169; expected: three hits — one at the ENTER,
   then repeats once `GameState+0x34 > Drone+0x11c + 0x1e`, then one at msg 5 — and `Drone+0x11c` written at
   the ENTER (confirms the state-dwell reading).
3. **Drone_SfxInfo row 0x60007ac deafness** [M 0x15ba68:69-74 vs drone.rs]: during the same walk, watch
   `Drone+0x4F8` bit 0x400000 at the PlayScript ENTER for a 0x60007ac drone (if any exists on disc) — expected
   per the decomp reading: NOT set (the port may have it backwards).
4. **MP bot pain/death barks**: MP with a bot; kill it and check `BOT_handlePain`'s line pool fires (watch
   `Sound_Play3D`'s id argument for 0x14d/0x152-range ids) — confirms the missing production wiring is a real
   user-visible gap, not a silent-in-practice path.
5. **`Rand(0)` odds gates** [M 0x13fa20:50 / 0x13fd08:50 / 0x13fe80:45 / 0x13fb98:45]: single-step `Rand_Rand`
   0x1e3738 at the InjuredTalk/RunForCoverTalk/UnderCover* call sites with `n = 0` — confirm the return is
   always 0 (the gates vacuous) so the port's omission of them stays correct.
6. **DroneSpawner_Control's NULL dcvars** [M 0x137e68:200]: the call passes dcvars = 0 while PlaySFX reads
   `dcvars[3]` and writes `drone+0x4d4` [M 0x141848:122/130] — disassemble 0x1382f0-0x138330 to see whether the
   decomp dropped a null check; the port plays the line through the spawner's own outbox (no dcvars), which is
   safe either way.

## Usage

Not visible to me. The sandbox exposes no token, credit or context-window figures for this session (no
environment variable, file or tool output in this session reported any such numbers).
