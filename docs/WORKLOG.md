# WORKLOG

## combat-sight-loss-exit

- (08:33 UTC, 10-24) Branch re-created (previous start commit 806c0575) and merged origin/master (26543436). Task: port the combat router's sight-loss exit to NoOpponent [M 0x148338] with the game's own HasOpponent hold (+0x238 stamp, 0x14ad58), and the NoOpponent 0x60 case-3 +0x170 re-point [M 0x1650a0], without stalling the walks. Suite started in background (tools/test/suite.py). Setup was already done (repo, work/, ISO, alsa-prefix; cargo + PKG_CONFIG_PATH needed re-export in fresh shells).
