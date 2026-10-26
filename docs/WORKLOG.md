
---
Task ID: level-05-audit
Agent: worker (s/level-05-audit)
Task: the 07000005 behaviour audit — every placed object/character vs the game; write docs/levels/07000005-audit.md; fix small gaps with tests; large gaps → report sections.

Work Log:
- Branch s/level-05-audit created from origin/master (b312d959); sandbox intact from the prior session (work/, ISO present). This entry written before any code change; pushing now as the first checkpoint.
- Next: suite.py in background, census probe for 07000005, audit sweep per the 06/08 method.

Stage Summary:
- Branch s/level-05-audit exists on origin at the clean master point; no code changed yet.

---
Task ID: level-05-audit
Agent: worker (s/level-05-audit)
Task: the 07000005 behaviour audit — every placed object/character vs the game; docs/levels/07000005-audit.md; small-gap fixes.

Work Log:
- Branch s/level-05-audit from origin/master (b312d959); work-slim/ISO reused from the prior session; ALSA user-prefix rebuilt (the sandbox had wiped it).
- Census probe census_07000005.rs --ignored: all 363 placements re-dumped from the disc (32 kinds — matches the level doc's §7 counts exactly).
- The eight live scripts decoded via the dump_sp_script example [D]: the truck ride-in (0x06000A22: s9 event 0xD [115,1] -> ch 0x73), the gatehouse hand-over (0x060008C8: camera 68 keys, event 6 [0] hand-back, event 4), the plunge death NIS (0x06000952: event 0xD [98,1] -> ch 0x62), the ledge NIS (0x06000994: camera 262 keys, two actor anims + weapon-byte 0xB [22], event 6 [2] hand-back), the EndLevel (0x06000A23: event 0x11 [152,152,29,1] — the CONDITIONAL write of ch 0x1D), the two copter flight scripts (event 0xF entity groups).
- Findings folded into docs/levels/07000005-audit.md (363 rows + scripts + mission + corrections + gaps): SP #71 doubly dead (script 0x06000074 absent from the archive [D kind-7 list]; watch 125 blocked by the 0x7D gate [M 0x1b1a78:80-82]); SoundTrigger stop = p4 not p1 [M 0x1b2628+0x3c, 0x1b2790]; TouchOnce p2..p8 inert on types 1/2 (#269's p6=51) with p10 = the disable gate; searchlight p1 = the ON/track channel; pickup p4 = the take sound; the EndLevel 0x11 condition; the sky +0x4a word flagged.
- Fix: the valley hurt test's header corrected to the who law [M 0x17e5d0:51-104] (the who-1 planes are game-inert; the kill is the plunge chain) — no assertion weakened; the file runs 11/11 green.
- Suite: chunked runs (chunk_suite.py, the sandbox kills background jobs) — 263+ binaries recorded, 902+ passes, 44 ignored; failures = the documented pre-existing/environmental set only (fix_0c_pylon_live, probe_pylon_render = the GPU render probes; campaign_flow_live = my 200s chunk limit, needs the 600s one; net_soak/net_resync/mp_shot_effects = exit -9 OOM class). The full pass did not fit the window.

Stage Summary:
- Commit carries the audit + the census probe + the test header fix. The corrections are in the audit doc (the level doc untouched per the shared-file rule). Large gaps: the west-ledge walk-to-grab, D-004 medal gate, the sky +0x4a word.
