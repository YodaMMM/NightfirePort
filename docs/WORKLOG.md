# WORKLOG

## stun-release-no-stun-anims

Branch `s/stun-release-no-stun-anims` created from `origin/master` (6f16146e).

Task: un-`#[ignore]` `crates/nf-view/tests/fix_grenade_types_live.rs` (~line 204, 107a) and make the
stun-grenade release chain finish for guards whose anim set lacks the 0x2d/0x2e stun anims. The stun arm
works (state 0x4d, weapon 1, stun_until = frame + 3600, TIMER_C fires); the release stalls in 0x4d —
guard 0 reaches the Loop, the chosen guard never leaves Impact. Read DroneAnim_CallAnim's no-datum
fallback in the decompile (what the game does when the requested anim is missing from the set: skip,
substitute, or end the clip at once) [M], port it, and fix the whole class (stun dart, stunner states
that call stun/dart anims).

Progress:
- [05:59 UTC] Branch created, WORKLOG started. Setup reused from sandbox (work/, ISO, nf-queue present).
- [06:45 UTC] Reproduced and read the law. DroneAnim_CallAnim 0x13ab58's no-datum fallback (the
  substitute table + the final 0x24 + the same-state CallFullyComplete arm) is ALREADY ported
  complete in call_anim_variant (arm-for-arm vs the decompile); DroneAnim_GetDAnimForCall
  0x13db30's retry-from-state-0 is ported in DroneAnimData::transition (or_else find(0)); a call
  whose transition exists nowhere pends exactly as the game (no timeout — +0x55A/+0x55E/+0x55F
  have no readers). All 24 anim sets can_do 0x2d/0x2e/0x21/0x76 (table probe) — the "sets lacking
  the stun anims" premise doesn't hold at the CanDoAnimState level.
- [06:55 UTC] Found the real [M] deviations in the stun-grenade family: (1) Rand polarity swapped —
  the game plays 0x2E on the draw 0 [M 0x172bb0 case 1 dis 0x172c34], the port played 0x2D;
  (2) the Loop ENTER must RE-ARM the wake timer with the STORED span +0xCC4 = frame + (+0xCE0)
  [M 0x1631c0 case 1] — the port carried the Impact deadline (one fall-clip short); (3) a flash in
  the Loop re-arms and STAYS (case 0x18) — the port re-entered Impact. The dart family
  (0x163430/0x163570/0x163700) and the taser family (0x162b98/0x162eb0) verified correct (polarity,
  stored spans, stay-in-loop arms).
- [07:05 UTC] Port fixes landed: stun_recover (+0xCE0 stand-in, stored at the Impact ENTER before
  the drop), the Loop fresh re-arm, the flash-in-Loop re-arm, the Rand polarity. oddjob_hat's
  fixture writes stun_recover now. EE rounding pinned: RecoverTime(0x35, b9=0) = 3599 (f32
  1800·0.01 = 17.9999996, fptoui truncates) [M 0x147188:44-46].
- [07:10 UTC] The ignored test rewritten and UN-IGNORED: the old selection ("within 20 of Bond")
  predates the delivery law — the flash reaches drones within 20.0 of the BLAST with the sight
  check [M 0x13a438 + 0x176f18]; the level-start guard list[6] stands 7.6 u below the blast floor
  and is unreachable by the game's own rule. New staging: Bond teleported 3 u right of the placed
  guard list[2], facing him (measured the one working offset), throw, pop, assert 0x4d + weapon 1
  + stun_recover 3599 + no explosions; then the release chain must FINISH: held > 3600 (the fresh
  re-arm), <= 4300, exit 0x56 (or 0x66 = Attack's draw after the fists change).
- [07:12 UTC] Green: nf-game lib 655, fix_grenade_types_live 4 (the un-ignored test in),
  anim_events_live 2, oddjob_hat_live 2.
