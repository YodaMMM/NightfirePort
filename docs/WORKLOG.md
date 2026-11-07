# WORKLOG

One entry per task, append only (never rewrite or reorder others' entries).

## mp-human-pain

2026-11-07 04:38 UTC, branch `s/mp-human-pain` (from origin/master 22d5836b7).

Task: route every multiplayer hit (bullet, blast, melee) on a human and a bot through the
ported `Player_HandlePain` 0x1902c8 multiplayer arm (`player_pain_mp` / `bot_pain` in
crates/nf-game/src/mp/botweap.rs): compare scale, locational multipliers, the x3
`location_damage` option (+0x1BC row), armour-first, the 1.0 floor and pain types line by
line against the live master path; add the settings the doc comment says are missing;
damage deterministic across machines, no shared-Rand draws; pin through a real match
(armour first on an armoured player, body-part scaling with the option on, no scaling with
it off); run every mp_*_live + mp_world_sync_live + the net hash binaries + nf-game + full
suite.

- [x] setup: repo/work/ISO/queue worktree reused from the sandbox, branch created from
  origin/master 22d5836b7 and pushed.
- [04:35-05:30 UTC] The live-path census: every MP hit on master routes through `Sim::mp_hurt`
  (scene.rs) — the human arm `combat::mp_damage` + `apply_pain`, the bot arm a bare
  `health -= mp_damage(...)`, the world route `net_world_pain` its own arithmetic. Line-by-line
  against `Player_HandlePain` 0x1902c8 / `BOT_handlePain` 0x125ed8: bullets and the fists' melee
  arrive as pain type 0 with the part [M 0x194458:195], blasts as type 0 location −1 [M
  0x196060], the difficulty factor and `Drone_ModPlayerHitDamage` 0x13a940 are SP-only arms
  [M 0x1902c8:28-50, 0x13a940:5-24]. Gaps found: the ported `player_pain_mp` carried only
  ×Multi + armour + floor (its doc claimed the settings "carry no +0x1BC/+0x1C8" — stale, the
  struct has them); the bot arm ignored the BOT_vars+0x769 armour pool the armour pickup fills
  [M 0x18e650:119-133, drained at 0x125ed8:88-92]; no pain-type fractions anywhere; the MPGame
  0xfffe stamp [M 0x1902c8:91-93] covered by the death credit; `net_world_pain` scaled by the
  SP `damage_factor` the MP arm never applies and missed the below-1.0 floor.
- [05:24 UTC] The routing: `player_pain_mp` takes the whole MP arm (rules + location + type,
  returns the scaled amount for the +0x968 flash [M :126]); `bot_pain` takes `PainRules`; both
  call the shared `combat::mp_damage` block [M 0x1902c8:51-82 = 0x125ed8:48-79]; `mp_hurt`'s
  human arm and `net_world_pain` route through `player_pain_mp`, the bot arm through `bot_pain`
  (the pool written back — hashed). The health loses the FULL scaled damage minus the armour
  share [M :100-108], not the share itself (the frac gates the armour split only). No shared
  stream draws moved (the arms draw nothing; the grunt's `Rand_Rand(4)` [M :111] and the bot
  barks keep their positions). Pinned in mp_pain_options_live (two machines, hashes agree):
  the head round ×4×4 with Location Damage on and the same round ×4 with it off, Professional
  ×3, the armour-first remainder on a human, the armoured bot's pool through the shared hurt
  path. nf-game lib 686/0/5, mp_combat_weapons_live 11/0, mp_pain_options_live 4/0. The full
  suite (every mp_*_live + mp_world_sync_live + the net binaries) running next.
