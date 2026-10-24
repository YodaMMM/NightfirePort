# Worklog

## cite-libsd-reverb-tables

- (12:58 UTC, 10-24) Branch created from origin/master (26543436). Task: cite the LIBSD effect parameter table and effect size table on the disc [D] (module, offset) behind crates/nf-audio/src/reverb.rs (STUDIO_3/mode 4 at :416, ROOM spot-checks :459/:492, EFFECT_SIZES[4] == 0x6FE), compare every port value against the disc, fix port+test together where they differ, and answer whether PS2_SetupReverb 0xc944:23's stack word 0x6fe0 agrees with the table. Sandbox reused (repo, work/, decomp of SFX.IRX present, ISO intact, 1061754880 bytes).
