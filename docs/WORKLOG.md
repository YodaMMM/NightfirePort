# WORKLOG

## unread-params-doors (s/unread-params-doors)

- 09:01 UTC started: task is to read every `*_Create` for doors, lifts, ladders, moving
  platforms and script-driven props in the disassembly, list every placement param stored,
  find what the port does with each, census non-zero values per level (`Level::parse` +
  `nf_view::dev::LEVELS`), close zero-everywhere params with the census and port real gaps.
- Setup reused (repo, work/, ISO, nf-queue present); branch cut from origin/master 22d5836b7.
