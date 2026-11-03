# dmode-init-civilians — the civilian, party-girl and door-guard drone modes

These game functions are never mentioned in crates/*/src: NDrone2_init_DMODE_Civilian, _CivDoorGuard, _HostageKiller, _CivilianScared, _PartyGirlLooker, _PartyGirl, _Sniper (and the other NDrone2_init_DMODE_* left; `grep -ri` the names first). They set up how civilians, party guests, door guards, hostage killers and snipers behave, on levels like 06/07 (the party) and 02.

For each: read the decompile, find what the port does for that mode, and port any difference with the address (start state, alert rules, what scares them, who they look at). Add a real-input test on a level that places the mode. Done = each function ported or cited as replaced, tests green, every walkreal_*_live green, the full suite total line. List each binary you ran.
