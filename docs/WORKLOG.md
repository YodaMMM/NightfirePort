
## los-mask-repins-02 — branch cut

Branch cut from origin/master c120d7b49 (contains 823ff190b, the game's guard sight: per-object
masks, the cel walk, the sight rotation, the guard's own cel, the own-cel boundary push,
FellOutMap). Task: the 07000002 walks were ignored because the game-correct sight moved their
fights (walk_the_castle_all_objectives_real_health_00agent and friends). Read each ignore message,
re-route each walk with real input so it passes, and remove its ignore. Never lower damage or
health, no top-ups; helpers gated to 07000002 and the difficulty. Read
docs/reports/los-mask-repairs.md on origin/s/los-mask-repairs for the sight rules. Run
walkreal_07000002_live (and its sister binaries) and interactables_07000002_live.
Status: branch cut + WORKLOG pushed; setup (work/, ISO) starting.
