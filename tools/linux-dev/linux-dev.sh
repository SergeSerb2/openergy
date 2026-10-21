#!/bin/bash
set -euo pipefail
root=$(cd "$(dirname "$0")/../.." && pwd)
prefix=oe-linux-dev-wp8
image=$prefix:rust-1.99
case ${1:-} in
 build)
  docker build -t "$image" "$root/tools/linux-dev"
  "$0" shell cargo build --release
  ;;
 run|shell)
  action=$1; shift
  args=(docker run --rm)
  if [[ $action == shell && $# == 0 && -t 0 ]]; then args+=(-it); fi
  args+=(--name "$prefix-$(date +%s)-$$" --network bridge -v "$root:/src:ro" -v "$prefix-target:/target" -v "$prefix-cargo:/cargo" --tmpfs /work:exec,size=512m "$image")
  if [[ $action == shell ]]; then
   if [[ $# == 0 ]]; then "${args[@]}" bash; else "${args[@]}" "$@"; fi
  else "${args[@]}" bash /work/tools/linux-dev/verify.sh "${1:-all}"; fi
  ;;
 down)
  ids=$(docker ps -aq --filter "name=^/$prefix-")
  if [[ -n $ids ]]; then docker rm -f $ids; fi
  ;;
 *) echo 'Usage: linux-dev.sh build|run [inject|clipboard|capture|all|check]|shell [command...]|down' >&2; exit 2 ;;
esac
