/* Fixture-only first image. Link only libSystem, never CF/Foundation/AppKit.
 * The generated header embeds Info.plist.in verbatim. This is an exact bounded
 * wire format, not a general XML/plist parser. No subprocess or fallback entry.
 */
#include <errno.h>
#include <fcntl.h>
#include <fts.h>
#include <limits.h>
#include <mach-o/dyld.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <sys/stat.h>
#include <unistd.h>
#include "private-info-template.h"

#define LIMIT 2048
#define ID_PREFIX "dev.openergy.updater-fixture."
#define ROOT_PREFIX "oe-updater-fixture-"
#define ENTRY_SUFFIX "/target/Openergy.app/Contents/MacOS/OpenergyPrivateLauncher"

static void require(bool ok, const char *reason) {
    if (ok) return;
    static const char prefix[] = "Private update launcher refused: ";
    (void)write(STDERR_FILENO, prefix, sizeof(prefix) - 1);
    (void)write(STDERR_FILENO, reason, strlen(reason));
    (void)write(STDERR_FILENO, ".\n", 2);
    _exit(78);
}

static void join(char out[LIMIT], const char *base, const char *suffix) {
    int n = snprintf(out, LIMIT, "%s%s", base, suffix);
    require(n > 0 && n < LIMIT, "path exceeds bound");
}

static void canonical(const char *path) {
    char resolved[PATH_MAX];
    require(realpath(path, resolved) != NULL && strcmp(path, resolved) == 0,
            "noncanonical path or symlink");
}

static struct stat attributes(const char *path) {
    struct stat s;
    require(lstat(path, &s) == 0 && s.st_uid == getuid(), "path unavailable or wrong owner");
    return s;
}

static void directory(const char *path) {
    struct stat s = attributes(path);
    require(S_ISDIR(s.st_mode) && (s.st_mode & 07777) == 0700, "private directory mode");
    canonical(path);
}

static void executable(const char *path) {
    struct stat s = attributes(path);
    require(S_ISREG(s.st_mode) && s.st_nlink == 1 && (s.st_mode & 07022) == 0
            && (s.st_mode & 0100) != 0, "unsafe executable");
    canonical(path);
}

/* Read no more than limit bytes plus EOF. Binary/NUL data cannot pass. */
static size_t read_file(const char *path, char *out, size_t limit, bool private) {
    canonical(path);
    int fd = open(path, O_RDONLY | O_NOFOLLOW | O_CLOEXEC | O_NONBLOCK);
    require(fd >= 0, "file unavailable");
    struct stat s;
    require(fstat(fd, &s) == 0 && s.st_uid == getuid() && S_ISREG(s.st_mode)
            && s.st_nlink == 1 && s.st_size >= 0 && (uint64_t)s.st_size <= limit
            && (private ? (s.st_mode & 07777) == 0600 : (s.st_mode & 07022) == 0),
            "unsafe file");
    size_t used = 0;
    for (;;) {
        ssize_t n = read(fd, out + used, limit + 1 - used);
        if (n < 0 && errno == EINTR) continue;
        require(n >= 0, "file read failed");
        if (n == 0) break;
        used += (size_t)n;
        require(used <= limit, "file exceeds bound");
    }
    require(close(fd) == 0 && used == (size_t)s.st_size && memchr(out, 0, used) == NULL,
            "file changed or contains NUL");
    out[used] = 0;
    return used;
}

static bool uuid(const char *s) {
    if (strlen(s) != 36) return false;
    for (size_t i = 0; i < 36; ++i) {
        if (i == 8 || i == 13 || i == 18 || i == 23) {
            if (s[i] != '-') return false;
        } else if (!((s[i] >= '0' && s[i] <= '9') || (s[i] >= 'a' && s[i] <= 'f'))) return false;
    }
    return true;
}

static bool decimal(const char *s, size_t maximum) {
    size_t n = strlen(s);
    if (n == 0 || n > maximum || s[0] == '0') return false;
    for (size_t i = 0; i < n; ++i) if (s[i] < '0' || s[i] > '9') return false;
    return true;
}

static bool root_characters(const char *s) {
    if (strlen(s) >= 512 || s[0] != '/') return false;
    for (; *s; ++s) {
        if (!((*s >= 'a' && *s <= 'z') || (*s >= 'A' && *s <= 'Z')
            || (*s >= '0' && *s <= '9') || strchr("/_-.", *s))) return false;
    }
    return true;
}

struct field { const char *name; char value[512]; bool seen; };

static void metadata(const char *bytes, struct field fields[6]) {
    const char *format = private_info_template;
    const char *cursor = bytes;
    for (;;) {
        const char *open = strchr(format, '@');
        if (!open) { require(strcmp(format, cursor) == 0, "plist format mismatch"); break; }
        size_t n = (size_t)(open - format);
        require(strlen(cursor) >= n && memcmp(format, cursor, n) == 0, "plist format mismatch");
        cursor += n;
        const char *end = strchr(open + 1, '@');
        require(end != NULL && end[1] == '<', "invalid compiled template");
        struct field *field = NULL;
        for (size_t i = 0; i < 6; ++i) {
            if (strlen(fields[i].name) == (size_t)(end - open - 1)
                && memcmp(fields[i].name, open + 1, (size_t)(end - open - 1)) == 0) field = &fields[i];
        }
        require(field != NULL && !field->seen, "invalid compiled field");
        const char *stop = strchr(cursor, '<');
        require(stop != NULL && stop > cursor && (size_t)(stop - cursor) < sizeof(field->value),
                "invalid plist field");
        memcpy(field->value, cursor, (size_t)(stop - cursor));
        field->value[stop - cursor] = 0;
        field->seen = true;
        cursor = stop;
        format = end + 1;
    }
    for (size_t i = 0; i < 6; ++i) require(fields[i].seen, "missing plist field");
}

static void tree(const char *path, bool private) {
    char start[LIMIT];
    join(start, path, "");
    char *paths[] = { start, NULL };
    FTS *walk = fts_open(paths, FTS_PHYSICAL | FTS_NOCHDIR, NULL);
    require(walk != NULL, "tree unavailable");
    size_t count = 0;
    for (;;) {
        errno = 0;
        FTSENT *entry = fts_read(walk);
        if (!entry) { require(errno == 0, "tree read failed"); break; }
        require(entry->fts_info != FTS_ERR && entry->fts_info != FTS_DNR
                && entry->fts_info != FTS_NS && entry->fts_info != FTS_DC, "invalid tree entry");
        if (entry->fts_info == FTS_DP) continue;
        require(++count <= 8193 && strlen(entry->fts_path) < LIMIT, "tree exceeds bound");
        struct stat s = attributes(entry->fts_path);
        if (private) {
            if (S_ISDIR(s.st_mode)) directory(entry->fts_path);
            else {
                require(S_ISREG(s.st_mode) && s.st_nlink == 1 && (s.st_mode & 07777) == 0600,
                        "unsafe private tree entry");
                canonical(entry->fts_path);
            }
        } else {
            require(S_ISDIR(s.st_mode) || S_ISREG(s.st_mode) || S_ISLNK(s.st_mode), "unsafe bundle type");
            if (!S_ISLNK(s.st_mode)) require((s.st_mode & 07022) == 0, "unsafe bundle mode");
            if (S_ISREG(s.st_mode)) require(s.st_nlink == 1, "bundle hard link");
            char resolved[PATH_MAX];
            require(realpath(entry->fts_path, resolved) != NULL, "broken bundle path");
            size_t n = strlen(path);
            require(strcmp(path, resolved) == 0
                    || (strncmp(path, resolved, n) == 0 && resolved[n] == '/'), "bundle path escapes");
        }
    }
    require(fts_close(walk) == 0, "tree close failed");
}

int main(int argc, char **argv) {
    (void)argv;
    require(argc == 1, "no arguments required");
    require(getuid() != 0 && getuid() == geteuid() && getgid() == getegid(), "unexpected user");
    errno = 0;
    int priority = getpriority(PRIO_PROCESS, 0);
    require(errno == 0 && priority <= 15, "nice 15 would raise priority");
    require(setpriority(PRIO_PROCESS, 0, 15) == 0 && getpriority(PRIO_PROCESS, 0) == 15, "absolute nice 15 failed");
    umask(077);

    char process[PATH_MAX], resolved[PATH_MAX];
    uint32_t size = sizeof(process);
    require(_NSGetExecutablePath(process, &size) == 0 && realpath(process, resolved) != NULL,
            "executable path unavailable");
    size_t n = strlen(resolved), suffix = strlen(ENTRY_SUFFIX);
    require(n > suffix && n - suffix < 512 && strcmp(resolved + n - suffix, ENTRY_SUFFIX) == 0,
            "unexpected entry location");
    char root[LIMIT];
    memcpy(root, resolved, n - suffix);
    root[n - suffix] = 0;
    require(root_characters(root), "invalid root characters");
    directory(root);
    char *base = strrchr(root, '/');
    require(base != NULL && strncmp(base + 1, ROOT_PREFIX, strlen(ROOT_PREFIX)) == 0,
            "invalid root basename");
    const char *id = base + 1 + strlen(ROOT_PREFIX);
    require(uuid(id), "invalid root UUID");
    char parent[LIMIT], temporary[PATH_MAX], user_temp[PATH_MAX];
    memcpy(parent, root, (size_t)(base - root));
    parent[base - root] = 0;
    size_t length = confstr(_CS_DARWIN_USER_TEMP_DIR, temporary, sizeof(temporary));
    require(length > 0 && length <= sizeof(temporary) && realpath(temporary, user_temp) != NULL,
            "user temp root unavailable");
    require(strcmp(parent, "/private/tmp") == 0 || strcmp(parent, user_temp) == 0, "temporary root required");

    char path[LIMIT], bundle[LIMIT], image[LIMIT], daemon[LIMIT];
    join(bundle, root, "/target/Openergy.app");
    join(image, bundle, "/Contents/MacOS/Openergy");
    join(daemon, bundle, "/Contents/Helpers/openergy");
    join(path, root, "/target"); directory(path);
    executable(resolved); executable(image); executable(daemon);
    tree(bundle, false);
    char plist[8193];
    join(path, bundle, "/Contents/Info.plist");
    (void)read_file(path, plist, 8192, false);
    struct field fields[6] = {{ .name = "FIXTURE_ID" }, { .name = "ROOT" },
        { .name = "NONCE" }, { .name = "PORT" }, { .name = "BUILD" }, { .name = "PUBLIC_ED_KEY" }};
    metadata(plist, fields);
    char identifier[LIMIT];
    join(identifier, ID_PREFIX, id);
    require(strcmp(fields[0].value, identifier) == 0 && strcmp(fields[1].value, root) == 0,
            "plist root or identifier mismatch");
    require(uuid(fields[2].value) && strcmp(fields[2].value, id) != 0, "independent nonce required");
    require(decimal(fields[3].value, 5) && decimal(fields[4].value, 9), "invalid port or build");
    // Carry the already selected fixture public key for the external official
    // CLI. This checks only canonical base64 syntax, never trust or signatures.
    const char *key = fields[5].value;
    require(strlen(key) == 44 && key[43] == '=' && strchr("AEIMQUYcgkosw048", key[42]) != NULL,
            "invalid fixture public key");
    for (size_t i = 0; i < 43; ++i)
        require((key[i] >= 'A' && key[i] <= 'Z') || (key[i] >= 'a' && key[i] <= 'z')
                || (key[i] >= '0' && key[i] <= '9') || key[i] == '+' || key[i] == '/',
                "invalid fixture public key");
    long port = strtol(fields[3].value, NULL, 10);
    require(port >= 25000 && port <= 65535, "private port required");

    const char *dirs[] = { "/state", "/state/tls", "/home", "/tmp", "/receipts" };
    for (size_t i = 0; i < sizeof(dirs) / sizeof(dirs[0]); ++i) {
        join(path, root, dirs[i]); directory(path);
    }
    const char *trees[] = { "/state", "/home", "/tmp", "/receipts" };
    for (size_t i = 0; i < sizeof(trees) / sizeof(trees[0]); ++i) {
        join(path, root, trees[i]); tree(path, true);
    }
    join(path, root, "/state/control.sock");
    struct stat socket;
    require(strlen(path) < 104 && lstat(path, &socket) == -1 && errno == ENOENT, "socket must be absent");
    char bytes[4097], config[4097];
    int written = snprintf(config, sizeof(config),
        "version = 1\nrole = \"off\"\nname = \"updater-fixture-%s\"\n"
        "[server]\nlisten = \"127.0.0.1:%ld\"\nport = %ld\n"
        "[client]\nremote = \"\"\nautoconnect = false\n[tls]\nenabled = false\n"
        "[clipboard]\nenabled = false\n", fields[2].value, port, port);
    require(written > 0 && (size_t)written < sizeof(config), "config exceeds bound");
    join(path, root, "/state/config.toml");
    (void)read_file(path, bytes, 4096, true);
    require(strcmp(config, bytes) == 0, "private config mismatch");
    join(path, root, "/fixture.json");
    (void)read_file(path, bytes, 4096, true); /* App image also decodes identity/lifetime/builds. */
    join(path, root, "/launch.lock");
    (void)read_file(path, bytes, 0, true);

    /* Fresh allowlist: no inherited OPENERGY, DYLD, HOME, XDG, or CF overrides.
     * These buffers outlive execve; no framework or child process is used here.
     */
    char values[10][LIMIT];
    const char *names[] = { "HOME", "CFFIXED_USER_HOME", "TMPDIR", "XDG_CONFIG_HOME", "XDG_CACHE_HOME",
        "OPENERGY_CONFIG", "OPENERGY_CONFIG_DIR", "OPENERGY_TLS_DIR", "OPENERGY_SOCKET", "OPENERGY_DAEMON" };
    const char *ends[] = { "/home", "/home", "/tmp/", "/state", "/home", "/state/config.toml",
        "/state", "/state/tls", "/state/control.sock", "/target/Openergy.app/Contents/Helpers/openergy" };
    char *environment[16];
    for (size_t i = 0; i < 10; ++i) {
        written = snprintf(values[i], LIMIT, "%s=%s%s", names[i], root, ends[i]);
        require(written > 0 && written < LIMIT, "environment exceeds bound");
        environment[i] = values[i];
    }
    environment[10] = "OPENERGY_DAEMON_ARGS=--platform virtual";
    environment[11] = "OPENERGY_DISABLE_DISCOVERY=1";
    environment[12] = "OPENERGY_ALLOW_PORT_24800=0";
    environment[13] = "PATH=/usr/bin:/bin";
    environment[14] = "LANG=C";
    environment[15] = NULL;
    require(chdir(root) == 0, "private directory unavailable");
    char *arguments[] = { image, "--private-update-fixture", "--e2e", NULL };
    execve(image, arguments, environment);
    require(false, "private app exec failed");
    return 78;
}
