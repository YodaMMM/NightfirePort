# Unreached or dead ACTION.ELF gameplay functions

Recorded per the unported-function tasks: a function the shipped game never reaches on any
on-foot level, the front end or MP — or that is reached only into a silent return — with the
evidence. Evidence tags: [M] decompile/disassembly with the file and line, [D] disc, [L] live
PCSX2. "No callers" below means: no direct call in the decompile, no `jal`/`j` encoding of the
address anywhere in ACTION.ELF, and no absolute data word of the address (checked with an
ELF scan over all load segments).

## `DroneFunc_UnderCoverAimTalk` 0x13fb98 — dead in both modes

Called from `NDrone2_DSTATE_UnderCoverAim` 0x16a2d0 [M caller grep], so the game does reach it —
but the body returns before anything observable:

- The first statement is `if (MPSettings+0x184 == 0) return;` [M 0013fb98:7-9] — the SP game
  returns at once (the decompiler's `MPSettings._388_4_` is `MPSettings+0x184`; the same gate
  field `Drone_FellOutMap` 0x13a878 branches on, SP == 0).
- In MP the line pick runs, but `NDrone2_SpeechSFX` refuses every request in MP (speech.md
  §2.1, [M 0x149818 disasm note]), so the play never happens.
- The remaining writes are the would-be `CoverTalkCooldown` (`DAT_00298FC8 = now + 5·FRAME_RATE`)
  behind a final `Rand(0) == 0` gate that is always true [M 0x13fb98:38-45] — but that store is
  only reached after `SpeechSFX` returns non-zero, which never happens.

So there is no input on which the function has an observable effect. speech.md §2.3 documents the
same verdict for its siblings (`InjuredTalk`/`RunForCoverTalk`/`UnderCoverTalk`: "dead in both
modes", verified in the disassembly). Not ported; no observable behaviour to pin.
