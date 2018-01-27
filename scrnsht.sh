#!/bin/bash -eu

# ./minrs.git/scrnsht.sh $(ls -p | \grep -v /)
[[ $# -eq 0 ]] && echo "Usage: $0  <file>+" && exit 2

detail_2d="$(dirname "$0")"/minrs
[[ -x "$detail_2d" ]]
out=pngs

mkdir $out
until [[ $# -eq 0 ]]; do
    f="$1"; shift
    if [[ -f "$f" ]]; then
        printf '\e[1;3m%s\e[0m\n' "$f skipped"
        continue
    fi
    printf '\e[1;3m%s\e[0m\n' "$f"
    scrot --quality 100 --count --focused --delay 2 $out/"$(basename "$f")".png &
    ( cmdpid=$BASHPID;
      (sleep 3 && echo killing && kill $cmdpid) & \
          exec "$detail_2d" "$f"
    ) || true
done
zip -r $out.zip $out
