# mine-bob-41

Branch cut from origin/master 6042ee53 (2026-11-01 22:49 UTC). WORKLOG started fresh (master tracks no docs/WORKLOG.md; recreated per prior-task convention).

## Task
Mine.posed_y (0xFE mine bob, Mine_Update 0x17c930) is computed but no draw path reads it. Census every level's 0xFE placements for a non-zero bob parameter [D]; if any bob, drive the mine's posed slot (scene.rs mine_placements) with posed_y; pin with tests (sister: zero-bob mine stays put). Run every *07000015*/*07000041* binary, ambient_sweep_live, load_scene_equality, nf-game.

## Progress
- Read Mine_Update 0x17c930 (decomp + capstone disasm 0x17c930-0x17ca08) and Mine_Create 0x17ce28 (+ disasm): the bob is `obj+0x34 = inst+4 + sin(GameState+0x52·inst+0x14)·inst+0x18` [M disasm 0x17c9c8-0x17c9f4]; inst+0x14 = param3·0.0002, inst+0x18 = param4·0.1 [M 0x17ce28:45-47]. The port's `bob_rate` used param2 — WRONG param (param2 accumulates the obj+0x54 spin angle [M 0x17c978-0x17c988]); fixed to param3.
- Draw chain [M]: View_DrawObjectGlist 0x1e6780:44 draws obj+0xD8 at obj+0x90; Control_BuildWorldSph 0x133070:17 rebuilds obj+0x90 per tick (the mine sets obj+0xFA |= 0x20 [M 0x17c9ec], the handler walk calls BuildWorldSph [M 0x1336f8:209]); Control_GetObjMatrix 0x134210:10-12 builds it from obj+0x50 rot + obj+0x30 pos; View_RotTransMatrix 0x1e5168:46-48 puts pos in the drawn translation. Param pairs scatter into scratch+0x2C+4k [M 0x1d1b68:65-76] — the port's `param(k)` over p.params is correct.
- scene.rs mine slot now takes the live posed_y into the drawn matrix y (was the static placement matrix).
- mine_bob_live.rs: [D] census over every level (expected: 9 on 07000015 bob off, 1 on 07000041 with p3=0x32 p4=0xa), 07000041 slot-bob pin, 07000015 stay-put sister.
- mine_bob_live: 3/3 pass — the [D] census pins (11 records: 07000015 m54 x9 bob off, 07000041 m99 #582 params 50/10/50/50/10/0x6000052, 1 null record on 07000041 non-main m9), the 07000041 slot-bob pin (slot y == pos.y + sin(tick*0.01)*1.0 for 260 ticks, peak > +0.99), the 07000015 stay-put sister (120 ticks).
- Full suite relaunched on the final tree (origin/master unchanged, 0 commits ahead).
