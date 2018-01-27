#!/bin/bash -eu

# ./minrs.git/scrnsht.sh $(ls -p | \grep -v /)
[[ $# -eq 0 ]] && echo "Usage: $0  <file>+" && exit 2

detail_2d="$(dirname "$0")"/minrs
[[ -x "$detail_2d" ]]

mkdir pngs
until [[ $# -eq 0 ]]; do
    f="$1"; shift
    [[ -f "$f" ]] || (printf '\e[1;3m%s\e[0m\n' "$f skipped" && continue)
    printf '\e[1;3m%s\e[0m\n' "$f"
    scrot --quality 100 --count --focused --delay 2 pngs/"$(basename "$f")".png &
    ( cmdpid=$BASHPID;
      (sleep 3 && echo killing && kill $cmdpid) & \
          exec "$detail_2d" "$f"
    ) || true
done
