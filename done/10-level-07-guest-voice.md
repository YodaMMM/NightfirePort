## Your task: `level-07-guest-voice`, 07000007: guests speak their lines aloud and turn their heads to Bond

Master now runs the party guests' talk gates (`NDrone2_BondTalk` 0x1446b0: Bond within 3.0, facing, unarmed, the per-dtype line tables, the 480-600 tick cooldowns — see docs/reports/level-07-guest-talk.md), but the line is silent and the head does not turn: every 07 BondTalk line is facial-present [D], and the port's facial path is unported (its silence is pinned by `speech_facial_fallback_live`).

1. Port the facial arm of `NDrone2_SpeechSFX` 0x141b00 [M]: the facial anim attach (`AnimFacialAppend`, the +0x53c attach flag) or the queued script id at +0x4c with the 20-tick delay (+0x48 = now + 0x14), `SFXPreCache` of `AnimScriptPeekSfx(facial)`, and the line's sound from the facial script's own sound command; BondTalk's cooldown advance once +0x53c stands [M 0x1446b0 case 0xd tail]. Lip-flap optional.
2. Re-derive the two `speech_facial_fallback_live` pins onto the game's advance-and-play behaviour, citing 0x141b00's facial arm.
3. Port the head turn (`HeadTrackObj(player, 200)`, 0x1446b0 / 0x144488) [M].
4. Extend `fix_guest_reactions_live::walking_up_to_a_guest_acks_the_talk_gates_and_does_not_scare` with the line-id pin from the guest's dtype table and a sound-emitted pin; a head-track pin.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-guest-voice`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox6 at 2026-10-25T02:07Z

---
## Report (done, 2026-10-25T03:28Z)

