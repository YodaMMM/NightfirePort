# audit-text-lines — every Text_AddMsg line against the port

Multiplayer's event lines and its friendly-fire and death lines were all missing until now, so sweep the class. List every Text_AddMsg 0x1b0c30 call in ACTION.ELF with a constant or computed label (23 functions: Mission_Update, Pickup_Handler, Check_Target, Player_WeaponFiring, Door_DoMsg's callers and the rest). For each: the type and id, the label's text, the condition, the display frames, and the port site that shows it. Remember the type rules at 0x1b0c30:81-92: type 4 becomes 5 outside a script camera, 6 becomes 1 in multiplayer, and types 4/5 follow the Subtitles option unless the id is 0xff. Fix every missing or wrong line that has a clear port site and pin each with a test; report the rest as a table. Merge origin/master first.

---
claimed-by: sandbox4 at 2026-11-04T20:55Z
