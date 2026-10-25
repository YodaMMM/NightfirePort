
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
