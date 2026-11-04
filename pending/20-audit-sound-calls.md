# audit-sound-calls — every gameplay sound call against the port

List every Sound_Play3D / Sound_Play call with a constant id in ACTION.ELF's on-foot gameplay functions (weapons, guards, doors, pickups, gadgets, player movement). For each id, find whether the port plays it at the same event. Report a table: function, address, id, port site or MISSING. Fix each MISSING a player would hear, with the address and a test on the sound pool. Skip driving and front-end. Done = the table, fixes tested, the full suite total line.
