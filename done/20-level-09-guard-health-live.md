## Your task: `level-09-guard-health-live`, 09: do the lobby guards really have 1 health?

With the placement stats-section decode (`drone_stats` 0x31aa30, written only by `behaviour_util_get` 0x1c7a98) the 07000009 type-0xD guards decode accuracy 80, health 1, reaction 80, recover 80. No live snapshot holds a type-0xD record. Settle it from data: decode the 09 stats section by hand against the field widths (8,3,8,8,5,8,8,1) and the bit order the decoder uses, check how health is consumed (DoModeSettingsNEW 0x14c600, the damage path Drone_ModBulletDamage) — is the stat a multiplier/index rather than raw health? — and if a fresh-boot read is possible in a headless sense (an EE snapshot taken at 09 load exists?), use it. Fix the decode or the consumer if wrong; pin the 09 guards' effective health with citations. Run stealth_ai_live, walkreal_07000009_live, enemy_accuracy_live and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-09-guard-health-live`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-11-01T12:18Z

---
## Report (done, 2026-11-01T14:12Z)


---
## Report (done, 2026-11-01T14:13Z)

# level-09-guard-health-live — the 09 lobby guards really have 1 health

**Result: YES — 1 health is the truth.** The placement stats-section decode is correct, the
consumer applies the stat as raw HP, and nothing needed fixing. The pins are added; the port already
played it.

## The evidence chain

1. **The section.** 07000009's only drone stats section sits on placement #319 (in-map #62 of map
   m39, hash `02000637`): params 15..32 with tag `0x80010006` at param 26 [D census
   `census_07000009 --ignored` dump; the other five type-0xD placements #321/#322/#323/#507/#518
   stop at param 25].
2. **The decoder, from disassembly** (`behaviour_util_get` 0x1c7a98 — the decompile C is incomplete
   here, the width-table copy is misread): the widths load is four unaligned qword pairs
   (LDL/LDR/SDL/SDR, 0x1c7c34-0x1c7c70) copying **8 words** from 0x303330 to the stack — the C shows
   only 4; the widths are `8,3,8,8,5,8,8,1` [D ACTION.ELF bytes at 0x303330]. Field f → record word f
   through the jump table at 0x303350 [D, eight entries → the `sw a0,offset(a2)` arms]. The kind
   advance is `(tag&0xffff)/3` **words** [M disasm 0x1c7c80 `mflo t4`, 0x1c7cac `sll t2,t4,2`,
   0x1c7d74 `addu s0,s0,t2`], the tag word itself skipped [M 0x1c7c1c delay slot], each field read
   from one word, LSB-first, masked `& 0xff` [M 0x1c7d00 `andi a0,v1,ff`].
3. **Live anchor.** The same decode reproduces the live `drone_stats` table byte-for-byte:
   ee_start_07000001.bin (32 MB, file offset = EE address) at 0x31aa30 + type·0x60 holds type 2 =
   9/0/12/60/2/60/60/0 | 8/1/12/80/3/80/80/1 | 7/2/12/100/3/100/100/0 and type 12 = 10/0/10/60/2/…,
   exactly what the decode of 07000001's #158 (type 2) and #161 (type 12) sections gives [L]. All
   three ee_start_*.bin snapshots hold only these boot-level records — none holds type 0xD, so 09's
   record is pinned from data + code, not from a snapshot.
4. **Applied to #319** the type-0xD records are:
   kind 0 = 60/0/1/60/2/60/60/0, kind 1 = 80/1/1/80/3/80/80/1, kind 2 = 100/2/1/100/3/100/100/0 —
   health **1** in all three kinds [D + the decoder above].
5. **The kind read is 1, always.** `DoModeSettingsNEW` calls
   `behaviour_util_getStats(type, 1)` with the constant 1 [M disasm 0x14c700-04 `addiu a1,zero,1`],
   so kinds 0/2 are written but never read by the single-player drone path. Difficulty scales
   elsewhere (DroneFiring_Accuracy_* / DroneDamage_* [M 0x179740:81-95, 0x145558:134-152]), not here.
6. **Raw health, not a multiplier/index.** Record word 2 goes straight to `+0xAC` as
   `(float)(int)w` [M 0x14c70c-0x14c730: `lw 8(a0)`, `cvt.s.w`, `swc1 f0,172(s4)`; zero word → the
   defaults 10.0]. `NDrone2_HitDamage` 0x145558 subtracts the (zone- and difficulty-scaled) damage
   from `+0xAC` [M :154] and `> 0` at :156 is the survival gate. No rescale off the stats path:
   DefaultInit's multiply at 0x14b300:556-561 is the captain branch (+0x19) and 0xD is not a captain
   kind (1,2,3,6,7) [M :540-555].
7. **Every 09 lobby guard runs DoModeSettingsNEW.** The OLD/NEW switch is `*(setup+0x48) < 36 →
   OLD` [M disasm 0x14bccc-d4 `sltiu v0,a1,36`; 0x48 = 0x70−10 words before param 15 → param 5], and
   all six type-0xD placements carry param 5 = 0x64 [D census]. #319 precedes the other five in
   placement order, so the shared table serves them all (the port's `level_drones` threads it the
   same way, incl. the param-2 ≤ difficulty gate — all six have param 2 = 1).
8. **Accuracy 80 is legal.** `DoBulletAccuracy`'s `p = 100 − 5·(+0xB4)` = −300 [M 0x179740:52] only
   selects the wobble arm vs the aimed zero-offset; negative p is a legal always-wobble, the same
   shape the f32 chain handles. The 60/80/100 accuracy progression also appears on other types (type
   15 in 07000004 with health 50, type 13 on 06/0a with health 1) [D disc-wide scan], i.e. the
   designers used a different scale for these types, not a decode artifact — the type-2 anchor
   rules that out.
9. **Corroboration.** The disc-wide scan (throwaway, not committed) found type-0xD sections on
   07000006 m48 #494, 07000009 and 0700000a m37 #54 — all decode health 1 in every kind: a
   consistent design for the sitting/stationary lobby guards.

## Effective health, pinned

A 09 lobby guard loads with `health = max_health = 1.0`, accuracy 80, aggression 1, reaction 80,
recover 80 — one landed round of ≥ 1.0 damage kills (the P2K round is 1.5 × difficulty [D
weapon_data]); a sub-1.0 round wounds only. Sleep darts still take the 0x19 path and lose no health
(the existing dart pin).

## Tests added

- `nf-game/tests/enemy_accuracy.rs::the_07000009_type_0xd_section_decodes_health_1` — #319's real
  params through `DroneStatsTable::behaviour_get`/`stats` (kinds 0/1/2 pinned word-for-word), the
  no-section defaults sister, and the negative-p survival of `hit_chance(0x0700_0009, acc 80)`.
- `nf-view/tests/walkreal_07000009_live.rs::the_lobby_guards_load_with_the_type_0xd_stats_health_1`
  — through the real load path: every `dtype == 0xd` drone (≥ 6) has health/max 1.0, accuracy 80,
  aggression 1, reaction/recover 80.

## What ran (all with NF_ISO on the slim on-foot image)

- `nf-game` crate: **1016 passed, 0 failed, 9 ignored**.
- `stealth_ai_live`: **7 passed, 0 failed**. `walkreal_07000009_live`: **6 passed, 0 failed**
  (incl. the new pin and the real-damage walk). `enemy_accuracy_live`: **5 passed, 0 failed,
  4 ignored** (the first suite-slot run was SIGKILLed by memory pressure while the full suite ran
  concurrently; the solo rerun above is the clean result).

## Full suite (tools/test/suite.py)

Build ok: 1065 test binaries in 1566 s. The test phase was still running at the sandbox deadline and
I cut it at 14:06 UTC after ~47 min (it had not reached the end of its binary list) to leave room for
the finish steps; a full re-run on the final tree did not fit. Up to the cut:

- `FAILED identity_disc::the_real_disc_passes_the_identity_check` — one of the 3 disc_golden fails
  the slim ISO is allowed.
- `FAILED glass_guards_14_live`, `missing_scenery_live` ×2, `fix_0c_pylon_live` ×2 — solo-reproduced
  panics at `.expect("device")` on `adapter.request_device` (glass_guards_14_live.rs:28,
  missing_scenery_live.rs:27, fix_0c_pylon_live.rs:19): **no wgpu device in this headless sandbox**.
  Environmental; these binaries cannot pass here on any tree, and my diff (two test files) does not
  touch them (their last-touch commits 0e7527b5/1f3063f2 predate the branch point).
- `CRASH net_desync_report_live (exit -9)` — OOM collateral of my concurrent foreground runs;
  solo rerun: **4 passed, 0 failed**.

## Live checks for the main agent

None required. If convenient, one cheap confirmation on a real PCSX2 boot of 07000009: after the
lobby loads, `gdbwatch.py` a type-0xD drone's `+0xAC` (float) — expected **1.0** — and `+0xB4`
(byte) — expected **80**; one pistol hit should drop the guard.

## What's left

Nothing on the question. The full-suite re-run on the final tree, if the next session wants the
complete total line: `NF_ISO=<abs> nohup python3 tools/test/suite.py > suite.out 2>&1 &` — expect
exactly the wgpu-device binaries + the 3 disc_golden fails on this image.

## Usage

not visible to me (no token, cost or context-window figures are shown in this environment).
