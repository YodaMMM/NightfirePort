# WORKLOG

## level-0b-objective9-writer (branch s/level-0b-objective9-writer)

- start (master 7d0091a0). Task: 0700000b objective 9 must complete on the warm walkthrough too —
  find every writer of ch 0x54/0x06 (objective 9's OR input) and the ch 0x75 touch after the
  cut-scene re-place [D trigger/SS/script census of 0700000b; M Trigger_Update 0x1b1a78 + script
  event handler]; determine which writer the warm route hits; fix any port gap; make
  `walkreal_0700000b_live` assert objective 9 completes on both walks (sister: open before the
  writer fires). Read level-0b-teleports-to-walks' report open items first.
- outcome (2026-10-31): objective 9 completes on BOTH walks through the game's own writer — the
  warm walk's ch 0x08 poke is DELETED. The writer census [D]: the terminal script 06000035
  stream 8 spawns at its tick 0x1e5 the type-0xEA TouchOnce (hash 0x2000ab5, params [6, 4, ..] =
  out ch 6, mask 4 = use [D nis dump; M Script_CreateEntity 0x1d8968 case 0xEA →
  Trigger_TouchOnce]) — the level's ONLY ch-0x06 writer (trigger census 60 rows has no out 0x6;
  every Event-0xD in 0700000b.bin + both packs writes 0x77/0x62/0x09/0x0a/0x0b/0x57/0x59/0x2a/0x76
  only; 0a carries none; the decompile has no constant switch_channels[6]/[0x54] store); ch 0x54
  has NO writer on 0b/its packs/0a — the OUT #294 → ch 0x08 arm is writer-less, the old poke
  modelled a writer that does not exist. Both walks walk the proven ring legs back from the desk
  and E-fire the spawned plate (sister: ch 0x06/0x4E open before the E) [D + M]. The ch 0x75
  item is NOT a port gap: the roof cut-scene's event-6 re-place key [−14.71, 58.00, −2.06] [D
  pack 0790000b] is 0.18 OUTSIDE lift_stop_trigger #245's box (z −1.88..2.15 [D trig dump]; the
  port's 12-tri glist matches the box bounds; touched_by false at the key, true 0.4 inside [L
  probe on the port tree]) — the doc's "INSIDE #245's volume" was wrong and is corrected; live
  check for the main agent: ch 0x75 (a) at the re-place, (b) after walking north into the box.
  walkreal_0700000b_live 3 passed / 0 failed (21.33 s, merged tree), interactables_0700000b_live
  9 passed / 0 failed (16.65 s); nf-view only (a test file) + docs changed.
