# mission-fail-menu — dying goes straight to a restart

Nothing outside the tests sets `auto_restart` to false, so dying or failing a mission reloads at once with no REPLAY/QUIT menu. Find the game's mission-fail flow in the decompile, port the menu and its choices through the real path, and test it by dying on 07000005 through the App: the page opens, REPLAY restarts from the entry save, QUIT returns to the mission list. Cite [M] addresses.
