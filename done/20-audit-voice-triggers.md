# audit-voice-triggers — what makes guards and allies speak

List every caller of the drone voice path (Drone_SfxInfo lines, the speech and bark functions) in ACTION.ELF: the trigger, the condition, the repeat delay, the line. For each say whether the port plays it on that trigger. Include allies, civilians, bosses and the player. Fix every missing or wrong trigger that has a clear port site and pin each with a test on real level data; report the rest as a table. Merge origin/master first.

---
claimed-by: sandbox3 at 2026-11-04T23:27Z

---
## Report (done, 2026-11-05T00:35Z)


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
    guard (props read from the real tables) into 0x86 and the ENTER requests the death l
...(full report: docs/reports/audit-voice-triggers.md on branch s/audit-voice-triggers)
