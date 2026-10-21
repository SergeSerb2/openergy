#!/usr/bin/env bash
set -euo pipefail
umask 077
export DISPLAY=:99
fail() { printf 'lab: %s\n' "$*" >&2; exit 1; }
alive() { [[ -s $1 ]] && kill -0 "$(cat "$1")" 2>/dev/null; }
stop_pid() {
  local file=$1 pid i
  if alive "$file"; then
    pid=$(cat "$file")
    kill "$pid" 2>/dev/null || true
    for ((i=0; i<30; i++)); do
      if ! kill -0 "$pid" 2>/dev/null || [[ $(ps -o stat= -p "$pid") == Z* ]]; then break; fi
      sleep .1
    done
    kill -KILL "$pid" 2>/dev/null || true
  fi
  rm -f "$file"
}
impl_ok() { [[ $1 == deskflow || $1 == barrier ]] || fail 'implementation must be deskflow or barrier (Input Leap is not packaged)'; }
port_ok() { [[ $1 =~ ^[0-9]{5}$ ]] && ((10#$1 >= 24900 && 10#$1 <= 65535)) || fail 'ports must be within 24900-65535'; }
name_ok() { [[ $1 =~ ^[a-zA-Z0-9][a-zA-Z0-9_.-]{0,254}$ ]] || fail "invalid screen name: $1"; }
cert_path() {
  if [[ $1 == deskflow ]]; then printf '/lab/deskflow/tls/deskflow.pem'; else printf '/lab/barrier/SSL/Barrier.pem'; fi
}
cert_make() {
  local pem
  pem=$(cert_path "$1")
  mkdir -p "$(dirname "$pem")"
  if [[ ! -s $pem ]]; then
    openssl req -x509 -newkey rsa:2048 -nodes -days 3650 -subj "/CN=openergy-interop-$1" \
      -keyout "$pem.key" -out "$pem.crt" >/dev/null 2>&1
    cat "$pem.key" "$pem.crt" > "$pem"
    rm -f "$pem.key" "$pem.crt"
  fi
}
fingerprint() {
  local pem hex colon
  pem=$(cert_path "$1")
  [[ -s $pem ]] || fail "no certificate for $1; start it with --tls on first"
  colon=$(openssl x509 -in "$pem" -noout -sha256 -fingerprint)
  colon=${colon#*=}
  hex=$(printf '%s' "$colon" | tr -d ':' | tr '[:upper:]' '[:lower:]')
  printf 'v2:sha256:%s\n%s\n' "$hex" "$colon"
}
cmd=${1:-status}; (($# == 0)) || shift
case "$cmd" in
  drive) (($# > 0)) || fail 'drive needs xdotool arguments'; exec xdotool "$@" ;;
  pointer) (($# == 0)) || fail 'pointer takes no arguments'; exec xdotool getmouselocation --shell ;;
  clipboard)
    case "${1:-}" in
      get) [[ $# == 1 ]] || fail 'clipboard get takes no text'; exec xclip -selection clipboard -out ;;
      set)
        [[ $# == 2 ]] || fail 'clipboard set requires one text argument (quote it)'
        stop_pid /lab/clipboard.pid
        printf '%s' "$2" | xclip -selection clipboard -in -quiet > /lab/clipboard.log 2>&1 &
        echo $! > /lab/clipboard.pid
        ;;
      *) fail 'use clipboard get or clipboard set <text>' ;;
    esac
    ;;
  events)
    case "${1:-}" in
      start)
        [[ $# == 1 ]] || fail 'events start takes no arguments'
        if ! alive /lab/events.pid; then
          stdbuf -oL -eL xinput test-xi2 --root > /lab/events.log 2>&1 &
          echo $! > /lab/events.pid
          sleep .2
          alive /lab/events.pid || fail 'event recorder failed'
        fi
        ;;
      stop) [[ $# == 1 ]] || fail 'events stop takes no arguments'; stop_pid /lab/events.pid ;;
      dump) [[ $# == 1 ]] || fail 'events dump takes no arguments'; cat /lab/events.log ;;
      *) fail 'use events start|stop|dump' ;;
    esac
    ;;
  status)
    printf 'DISPLAY=%s\n' "$DISPLAY"
    xdpyinfo | sed -n '/dimensions:/p'
    ps -eo pid,args | awk '/[X]vfb|[d]eskflow-core|[b]arriers|[b]arrierc|[x]input test-xi2/'
    ss -ltn
    ;;
  stop|fingerprint)
    [[ $# == 1 ]] || fail "$cmd requires an implementation"
    impl_ok "$1"
    if [[ $cmd == stop ]]; then stop_pid "/lab/$1/peer.pid"; else fingerprint "$1"; fi
    ;;
  server|client)
    (($# > 0)) || fail "$cmd requires an implementation"
    impl=$1; shift; impl_ok "$impl"
    port=24900; screens=lab,openergy-mac; position=right; tls=off; trust=; server=; name=lab; protocol=synergy
    while (($#)); do
      (($# >= 2)) || fail "missing value for $1"
      case "$1" in
        --port) port=$2 ;;
        --screens) screens=$2 ;;
        --position) position=$2 ;;
        --client-name) screens="lab,$2" ;;
        --tls) tls=$2 ;;
        --server) server=$2 ;;
        --name) name=$2 ;;
        --protocol) protocol=$2 ;;
        --trust-fingerprint) trust=$2 ;;
        *) fail "unknown option: $1" ;;
      esac
      shift 2
    done
    [[ $tls == on || $tls == off ]] || fail '--tls must be on or off'
    [[ $protocol == synergy || $protocol == barrier ]] || fail '--protocol must be synergy or barrier'
    case "$position" in right) opposite=left ;; left) opposite=right ;; top) opposite=bottom; position=up ;; bottom) opposite=up; position=down ;; *) fail '--position must be right, left, top, or bottom' ;; esac
    # Config syntax uses up/down rather than top/bottom.
    [[ $opposite != bottom ]] || opposite=down
    if [[ $cmd == client ]]; then
      [[ $server == host.docker.internal:* || $server == host.orb.internal:* ]] \
        || fail '--server must be host.docker.internal:PORT or host.orb.internal:PORT'
      port=${server##*:}; host=${server%:*}; name_ok "$name"
    else
      [[ -z $server ]] || fail '--server belongs to client mode'
      name=lab
    fi
    port_ok "$port"
    if [[ $cmd == server ]]; then
      range=${LAB_PORT_RANGE:-24900-24910}; low=${range%-*}; high=${range#*-}
      ((10#$port >= 10#$low && 10#$port <= 10#$high)) || fail 'server port is outside the published LAB_PORT_RANGE'
    fi
    if [[ -n $trust ]]; then
      [[ $tls == on ]] || fail '--trust-fingerprint requires --tls on'
      trust=${trust#v2:sha256:}; trust=${trust//:/}; trust=${trust,,}
      [[ $trust =~ ^[0-9a-f]{64}$ ]] || fail 'trust fingerprint must contain exactly 64 SHA-256 hex digits'
    fi
    dir=/lab/$impl; mkdir -p "$dir"
    signature=$(printf '%s\0' "$cmd" "$port" "$screens" "$position" "$tls" "$trust" "$server" "$name" "$protocol" | sha256sum | cut -d' ' -f1)
    if alive "$dir/peer.pid" && [[ -f $dir/signature && $(cat "$dir/signature") == "$signature" ]]; then
      printf '%s %s already running (pid %s)\n' "$impl" "$cmd" "$(cat "$dir/peer.pid")"
      if [[ $tls == on ]]; then fingerprint "$impl"; fi
      exit 0
    fi
    # One X display is shared. Stop old peers before giving another peer its input grab.
    for peer in deskflow barrier; do stop_pid "/lab/$peer/peer.pid"; done
    if [[ $cmd == server ]]; then
      if [[ $screens == *'section: screens'* ]]; then
        printf '%s\n' "$screens" > "$dir/server.conf"
      else
        screens=${screens//,/ }; read -r -a screen_names <<< "$screens"
        if ((${#screen_names[@]} == 1)); then remote=${screen_names[0]}; else
          ((${#screen_names[@]} == 2)) && [[ ${screen_names[0]} == lab ]] || fail '--screens requires lab,CLIENT or CLIENT or a complete config'
          remote=${screen_names[1]}
        fi
        name_ok "$remote"; [[ $remote != lab ]] || fail 'client screen must differ from lab'
        cat > "$dir/server.conf" <<EOF
section: screens
    lab:
    $remote:
end
section: links
    lab:
        $position = $remote
    $remote:
        $opposite = lab
end
EOF
      fi
    fi
    if [[ $tls == on ]]; then cert_make "$impl"; fi
    if [[ $impl == deskflow ]]; then
      mkdir -p "$dir/tls"
      if [[ $cmd == server ]]; then
        # 1.27 moved computer declarations into the settings INI. Keep only
        # links/options in the external server file; translate legacy screens.
        awk '/^[[:space:]]*section: screens[[:space:]]*$/ {s=1; next}
             s && /^[[:space:]]*end[[:space:]]*$/ {s=0; next}
             s && /^[[:space:]]*[a-zA-Z0-9_.-]+:[[:space:]]*$/ {gsub(/[[:space:]:]/, ""); print}' \
          "$dir/server.conf" > "$dir/computers"
        [[ -s $dir/computers ]] || fail 'Deskflow server config needs a screens section'
        awk '/^[[:space:]]*section: screens[[:space:]]*$/ {s=1; next}
             s && /^[[:space:]]*end[[:space:]]*$/ {s=0; next} !s {print}' \
          "$dir/server.conf" > "$dir/links.conf"
      fi
      # Replace the pin for this launch. Never silently retain an old server's trust.
      db=trusted-servers; [[ $cmd != server ]] || db=trusted-clients
      : > "$dir/tls/$db"
      [[ -z $trust ]] || printf 'v2:sha256:%s\n' "$trust" > "$dir/tls/$db"
      check=false; [[ $cmd != client && -z $trust ]] || check=true
      cat > "$dir/settings.ini" <<EOF
[core]
lastVersion=1.27.0
computerName=$name
display=:99
interface=0.0.0.0
port=$port
processMode=1
[client]
remoteHost=${host:-}
[server]
externalConfig=true
externalConfigFile=$dir/links.conf
protocol=$protocol
[security]
tlsEnabled=$([[ $tls == on ]] && echo true || echo false)
checkPeerFingerprints=$check
certificate=$dir/tls/deskflow.pem
[log]
level=VERBOSE
toFile=true
file=$dir/peer.log
EOF
      if [[ $cmd == server ]]; then
        while IFS= read -r computer; do
          name_ok "$computer"
          printf '\n[computer_%s]\nname=%s\n' "$computer" "$computer" >> "$dir/settings.ini"
        done < "$dir/computers"
      fi
      args=(deskflow-core "$cmd" --settings "$dir/settings.ini" --new-instance)
    else
      mkdir -p "$dir/SSL/Fingerprints"
      db=TrustedServers.txt; [[ $cmd != server ]] || db=TrustedClients.txt
      : > "$dir/SSL/Fingerprints/$db"
      [[ -z $trust ]] || printf 'v2:sha256:%s\n' "$trust" > "$dir/SSL/Fingerprints/$db"
      args=(--no-daemon --debug DEBUG2 --name "$name" --display :99 --profile-dir "$dir")
      if [[ $tls == off ]]; then args+=(--disable-crypto); fi
      if [[ $cmd == server ]]; then
        if [[ -z $trust ]]; then args+=(--disable-client-cert-checking); fi
        args=(barriers "${args[@]}" --address "0.0.0.0:$port" --config "$dir/server.conf");
      else args=(barrierc "${args[@]}" "$server"); fi
    fi
    if [[ -f $dir/peer.log ]]; then cp "$dir/peer.log" "$dir/peer.previous.log"; fi
    output=$dir/peer.log
    if [[ $impl == deskflow ]]; then output=$dir/launcher.log; : > "$dir/peer.log"; fi
    "${args[@]}" > "$output" 2>&1 < /dev/null &
    echo $! > "$dir/peer.pid"
    printf '%s\n' "$signature" > "$dir/signature"
    # Observe listener readiness without making a protocol connection.
    for ((i=0; i<100; i++)); do
      if ! alive "$dir/peer.pid"; then cat "$dir/peer.log" >&2; fail "$impl exited during startup"; fi
      if [[ $cmd == client ]] || [[ -n $(ss -H -ltn "sport = :$port") ]]; then
        sleep .2
        alive "$dir/peer.pid" || { cat "$dir/peer.log" >&2; fail "$impl exited during startup"; }
        printf '%s %s started (pid %s, port %s, TLS %s)\n' "$impl" "$cmd" "$(cat "$dir/peer.pid")" "$port" "$tls"
        if [[ $tls == on ]]; then fingerprint "$impl"; fi
        exit 0
      fi
      sleep .1
    done
    cat "$dir/peer.log" >&2
    stop_pid "$dir/peer.pid"
    fail 'peer did not start listening'
    ;;
  *) fail "unknown command: $cmd" ;;
esac
