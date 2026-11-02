# level-08-controlroom-route — walk into the control room and break its glass

Branch from origin/master.

`interactables_07000008_live::glass_pane_breaks_on_the_first_bullet` is ignored: the control room panes sit inside the lower station, and its start #8 approach is the documented wedge, so no walked route reaches them.

Do:
1. Find the game's route into the control room interior [D placements/doors, M door/lift code]; fix the wedge if it is a port collision or door-state bug (cite the decompile), otherwise route around it.
2. Walk in on real input and shoot a control room pane; it must break on the first bullet like the game's glass kind.
3. Un-ignore the test; run interactables_07000008_live, walkreal_07000008_live, then the full suite.
