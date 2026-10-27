#!/bin/bash
# Resume-capable suite runner: one line per binary in suite-progress.txt
cd "$(git rev-parse --show-toplevel)"
export PATH=$HOME/.cargo/bin:$PATH
export PKG_CONFIG_PATH=/home/z/alsa/dev/usr/lib/x86_64-linux-gnu/pkgconfig
export RUSTFLAGS="-L native=/home/z/alsa/dev/usr/lib/x86_64-linux-gnu"
export LD_LIBRARY_PATH=/home/z/alsa/dev/usr/lib/x86_64-linux-gnu
export NF_ISO=${NF_ISO:?set NF_ISO to the on-foot ISO}
export NF_SILENT=1
while read -r exe; do
  name=$(basename "$exe" | sed 's/-[0-9a-f]\{8,16\}$//')
  grep -q "^[A-Z]* $name \$" suite-progress.txt 2>/dev/null && continue
  timeout 300 "$exe" --test-threads=1 > "/tmp/suite_$name.log" 2>&1
  rc=$?
  if [ $rc -eq 0 ]; then echo "OK $name" >> suite-progress.txt
  elif [ $rc -eq 124 ]; then echo "TIMEOUT $name" >> suite-progress.txt
  else echo "FAIL $name" >> suite-progress.txt; fi
done < exe_list.txt
echo "ALL_DONE" >> suite-progress.txt
