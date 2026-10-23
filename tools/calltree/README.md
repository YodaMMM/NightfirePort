# calltree.py — what the game's code reaches that the port never mentions

`python -I tools/calltree/calltree.py <decomp dir> <ACTION.ELF> <repo dir> <entry> [...] [--list]`
`python -I tools/calltree/calltree.py <decomp dir> <ACTION.ELF> <repo dir> --all-mentioned-dstates --list`

Walks the decompile's call graph from each entry function (direct calls, plus `Drone_SM_SetState`
state changes mapped through `NDrone2_StateFuncs` 0x29b300) and lists every reachable function whose
name or address appears nowhere in `crates/*/src`. "Mentioned" is not "ported complete" — but a
never-mentioned reachable function is certainly missing (or must be proven unused / replaced, with a
citation in the code).

**The rule:** a feature is not done while its entry reports never-mentioned callees. Each remaining
callee is ported, or cited in the code as replaced by a named port function, or proven never called
on the shipped path.
