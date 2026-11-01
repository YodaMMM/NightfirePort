# WORKLOG

## level-09-guard-health-live (2026-11-01)

Task: do the 07000009 type-0xD lobby guards really have 1 health? With the placement stats-section
decode (`drone_stats` 0x31aa30, written only by `behaviour_util_get` 0x1c7a98) the 09 type-0xD guards
decode accuracy 80, health 1, reaction 80, recover 80. No live snapshot holds a type-0xD record.
Settle it from data: decode the 09 stats section by hand against the field widths (8,3,8,8,5,8,8,1)
and the bit order the decoder uses, check how health is consumed (DoModeSettingsNEW 0x14c600, the
damage path Drone_ModBulletDamage) — is the stat a multiplier/index rather than raw health? — and use
a fresh-boot EE snapshot at 09 load if one exists. Fix the decode or the consumer if wrong; pin the
09 guards' effective health with citations. Branch start: created s/level-09-guard-health-live from
origin/master.
