# WORKLOG

## window-size-fullscreen

Started 2026-11-13T02:55Z: branch from origin/master 23b0ae649. Remaining scope per driver: window size presets (800x600/1280x960/1600x1200/1920x1440) and a fullscreen row on a port-only Display page, both persisted through the existing display.toml (settings::display_path); Alt+Enter + 4:3 pillarbox already on master (631871863). Tests: persistence round trip, aspect fit maths, nf-view --lib, menus_e2e, menus_mouse_parity; headless --shot at two sizes.
- Port-only Display page (PAGE_DISPLAY 0x5000_0006) added: Alt+D opens it, rows are Fullscreen toggle + the 4:3 presets that fit the synced monitor + Back; picks ride display_request → host menu_after drain (apply + the same display.toml). No game page's rows touched; the accidental PAGE_MP_ONLINE_LOBBY value bump was caught and reverted pre-commit.
- Tests on the merged tree (origin/master 2265b403e): nf-view --lib 141, display_page 8 (new), menus_e2e 5, menus_flow 43, menus_mouse_parity 18, page_titles 3 — all green. Headless --page display + --size WxH render aids; two shots in docs/reports/. suite.py did NOT fit (toolchain + downloads ate the window; 2 cores) — listed in the report.
