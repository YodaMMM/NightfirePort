# WORKLOG

## window-size-fullscreen

Started 2026-11-13T02:55Z: branch from origin/master 23b0ae649. Remaining scope per driver: window size presets (800x600/1280x960/1600x1200/1920x1440) and a fullscreen row on a port-only Display page, both persisted through the existing display.toml (settings::display_path); Alt+Enter + 4:3 pillarbox already on master (631871863). Tests: persistence round trip, aspect fit maths, nf-view --lib, menus_e2e, menus_mouse_parity; headless --shot at two sizes.
