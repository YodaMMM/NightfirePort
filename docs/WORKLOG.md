# WORKLOG

## unread-params-doors (s/unread-params-doors)

- 09:01 UTC started: task is to read every `*_Create` for doors, lifts, ladders, moving
  platforms and script-driven props in the disassembly, list every placement param stored,
  find what the port does with each, census non-zero values per level (`Level::parse` +
  `nf_view::dev::LEVELS`), close zero-everywhere params with the census and port real gaps.
- Setup reused (repo, work/, ISO, nf-queue present); branch cut from origin/master 22d5836b7.
- Census done (placement_param_census_live): Destroy p6 non-zero ONLY on 07000014 (17 rows,
  max 10) — the task's gap confirmed; SS p7 one row (07000046#54, value 1, write target
  cel+0x90 has no reader in the decomp — unknown-meaning field, default −500); Ladder p1 zero
  on every level (closed); Ladder p0 one row but the creator never reads it. Door/CamSubject/
  DynObject/Break params all read by the port.
- Ported the Destroy p6 countdown: `Destroy.countdown` (p6·−30 at create [L load FRAME_RATE],
  `Destroy_Smash` negate [M 0x12bb60:31], `Destroy_Update` count arm [M 0x12c4f8:16-28] in
  `channel_kills`), the bullet-path double-flip [M 0x12bd48:41-49], the draw swap gate moved
  to the smash latch (`latched`). Tests: 3 unit + 2 real-data (07000014 #923/#294) — red on
  the old code (the smash fired the channel in-call).
