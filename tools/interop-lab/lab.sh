#!/usr/bin/env bash
set -euo pipefail
umask 077
ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
IMAGE=openergy-interop:latest
CONTAINER=${LAB_NAME:-openergy-interop}
OWNER=org.openergy.interop-lab
RANGE=${LAB_PORT_RANGE:-24900-24910}
fail() { printf 'lab: %s\n' "$*" >&2; exit 1; }
usage() {
  cat <<'EOF'
Usage: lab.sh build|up|down|remove|status
       lab.sh server deskflow|barrier [--port 24900] [--screens 'lab,openergy-mac']
                     [--position right|left|top|bottom] [--tls on|off]
                     [--protocol synergy|barrier] [--trust-fingerprint SHA256]
       lab.sh client deskflow|barrier --server host.docker.internal:24901
                     [--name lab] [--tls on|off] [--trust-fingerprint SHA256]
       lab.sh drive <xdotool arguments...>
       lab.sh pointer | clipboard get | clipboard set <text>
       lab.sh events start|stop|dump
       lab.sh logs|stop|fingerprint deskflow|barrier
LAB_PORT_RANGE sets published localhost ports (default 24900-24910).
LAB_NAME sets the container name (default openergy-interop), so parallel runs can use separate labs.
EOF
}
exists() { docker container inspect "$CONTAINER" >/dev/null 2>&1; }
owned() {
  [[ $(docker inspect -f '{{index .Config.Labels "org.openergy.interop-lab"}}' "$CONTAINER") == 1 ]] \
    || fail "container $CONTAINER is not owned by this lab"
  [[ $(docker inspect -f '{{index .Config.Labels "org.openergy.interop-lab.root"}}' "$CONTAINER") == "$ROOT" ]] \
    || fail "container $CONTAINER belongs to a different lab directory (or predates directory ownership labels)"
}
running() { [[ $(docker inspect -f '{{.State.Running}}' "$CONTAINER" 2>/dev/null) == true ]]; }
validate_range() {
  [[ $RANGE =~ ^([0-9]{5})-([0-9]{5})$ ]] || fail 'LAB_PORT_RANGE must be START-END'
  first=$((10#${BASH_REMATCH[1]})); last=$((10#${BASH_REMATCH[2]}))
  ((first >= 24900 && last >= first && last <= 65535 && last-first <= 100)) \
    || fail 'port range must be within 24900-65535 and contain at most 101 ports'
}
cmd=${1:-help}; (($# == 0)) || shift
case "$cmd" in
  help|-h|--help) usage ;;
  build)
    (($# == 0)) || fail 'build takes no arguments'
    docker build --tag "$IMAGE" "$ROOT"
    ;;
  up)
    (($# == 0)) || fail 'up takes no arguments'
    validate_range
    mkdir -p "$ROOT/.state"
    if exists; then
      owned
      old_range=$(docker inspect -f '{{index .Config.Labels "org.openergy.interop-lab.ports"}}' "$CONTAINER")
      old_image=$(docker inspect -f '{{.Image}}' "$CONTAINER")
      new_image=$(docker image inspect -f '{{.Id}}' "$IMAGE")
      if [[ $old_range != "$RANGE" || $old_image != "$new_image" ]]; then
        running && fail 'stop the lab before changing its image or port range'
        docker rm "$CONTAINER" >/dev/null
      fi
    fi
    if ! exists; then
      docker create --name "$CONTAINER" --hostname lab --init=false \
        --label "$OWNER=1" --label "$OWNER.ports=$RANGE" \
        --label "$OWNER.root=$ROOT" \
        --publish "127.0.0.1:$RANGE:$RANGE/tcp" \
        --mount "type=bind,src=$ROOT/.state,dst=/lab" \
        --health-cmd 'xdpyinfo -display :99 >/dev/null 2>&1' \
        --health-interval 5s --health-timeout 2s --health-retries 3 \
        "$IMAGE" >/dev/null
    fi
    running || docker start "$CONTAINER" >/dev/null
    for ((i=0; i<100; i++)); do
      if docker exec "$CONTAINER" xdpyinfo -display :99 >/dev/null 2>&1; then
        printf '%s ready: DISPLAY=:99, localhost:%s\n' "$CONTAINER" "$RANGE"
        exit 0
      fi
      sleep .1
    done
    docker logs "$CONTAINER" >&2
    fail 'Xvfb did not become ready'
    ;;
  down)
    (($# == 0)) || fail 'down takes no arguments'
    if exists; then owned; if running; then docker stop --timeout 5 "$CONTAINER"; fi; fi
    ;;
  remove)
    (($# == 0)) || fail 'remove takes no arguments'
    if exists; then
      owned
      running && fail 'stop the lab before removing it'
      docker rm "$CONTAINER" >/dev/null
    fi
    ;;
  status)
    if ! exists; then printf 'Container absent\n'; exit 0; fi
    owned
    docker inspect -f 'container={{.Name}} running={{.State.Running}} image={{.Config.Image}} ports={{json .HostConfig.PortBindings}}' "$CONTAINER"
    if running; then docker exec "$CONTAINER" lab-peer status; fi
    ;;
  logs)
    [[ $# == 1 && ($1 == deskflow || $1 == barrier) ]] || fail 'logs requires deskflow or barrier'
    if [[ -f $ROOT/.state/$1/peer.log ]]; then cat "$ROOT/.state/$1/peer.log"; else fail "no $1 log yet"; fi
    ;;
  server|client|drive|pointer|clipboard|events|stop|fingerprint)
    exists || fail 'run lab.sh up first'
    owned
    running || fail 'run lab.sh up first'
    docker exec -e LAB_PORT_RANGE="$(docker inspect -f '{{index .Config.Labels "org.openergy.interop-lab.ports"}}' "$CONTAINER")" \
      "$CONTAINER" lab-peer "$cmd" "$@"
    ;;
  *) usage >&2; fail "unknown command: $cmd" ;;
esac
