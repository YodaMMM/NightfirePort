
## level-07-guest-faces (2026-10-25, sandbox expiring ~18:36 UTC)
Branch `s/level-07-guest-faces` off origin/master (7c74d96b). Task: 07000007 — party guests
speak BondTalk lines through the facial path [M NDrone2_SpeechSFX 0x141b00,
NDrone2_BondTalk 0x1446b0] but their faces do not move in real play. Find the game's facial
animation for these lines [M — AnimFacialAppend, the +0x53c attach, the facial script's anim
ops, the morph/bone channels the facial anim drives] and what the port does with it (the GPU
facial morph path exists: upload_morph_weights, facial tests facial2/facial3/facial_gpu). Port
what is missing so the talking guest's face animates with the line; a test on the morph
weights / bone channels during a guest line, plus headless renders of a talking guest named
in the report.
Plan: reuse sandbox work/ + ISO + nf-queue; branch pushed at start; fast suite runner started
early; census NDrone2_SpeechSFX / NDrone2_BondTalk / AnimFacialAppend and the +0x53c attach
in the decompile [M], census the port's morph path, fix with citations, tests + headless
renders, report + finish.

- (16:38-17:20) Setup reused; branch pushed; fast suite up (alsa prefix rebuilt). [M] census:
  NDrone2_BondTalk 0x1446b0 → NDrone2_SpeechSFX 0x141b00's FACIAL arm (AnimScriptPeekSfx
  0x1c6ca8 → AnimFacialAppend 0x1c6490 → DCVars+0x53c; disasm: the decompile's void return is
  wrong, $v0 = the AnimScriptAppend script, +0xb5=3, +0x80=0xe0000002, animobj+0x109=2).
  AnimFacialBlend 0x1be648 reads ONLY the three +0x80 marker scripts (AnimListBuild 0x1c0fc0
  bit 0x200 = script+0x80 == marker) into animobj+0xb0 (count skin+0x2e); AnimScriptInit
  0x1c1600:58-61 gives gameplay scripts marker 0 — the body anim's face window is not read in
  gameplay. Port: the talk slot2 was hardwired None in Scene::facial_weights_at AND a body
  anim with no face window returned None (no morphs at all); probe (probe_guest_face): 07000007
  guests' idle/walk body anims carry NO face window, so a talking guest (facial_talk attached,
  op-1 sounding) got zero weights — the frozen face. Fix: zero-base fallback + slot2 fed from
  Drone.facial_talk/_start on the op-1 talk clock (Drones.frame — last_tick was never stamped);
  probe now shows live changing weights during a line. Next: the live test, renders, suite.
