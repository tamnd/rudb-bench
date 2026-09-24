/* The device card, taken by the harness rather than reported by an engine.
 *
 * notes/Spec/2140/engine-v4/16-measurement.md section 16.3 has five probes, and rudb runs them in
 * rudb-io/src/device.rs. Section 16.4 asks the harness to run the same five on every machine it
 * measures a rival on, with its own code, so a durable number is stated next to a card nobody
 * under test wrote. This is that code. It prints one JSON object whose keys are the column names
 * of rudb_device_card, so the two sit side by side in a report.
 *
 *   cc -O2 -pthread -o device-card scripts/device-card.c
 *   ./device-card DIR [ITERATIONS]
 *
 * All of it runs on scratch files in DIR, which are removed before it exits. The probes:
 *
 *   1. For every sync call the platform has, a 4 KiB pwrite at a sequential offset followed by
 *      that call, timed one iteration at a time, into a file preallocated so the size never moves.
 *   2. The same at 64 KiB.
 *   3. Sequential write bandwidth: 256 MiB in 1 MiB writes, then one sync, timed as a whole.
 *   4. Sync scaling: 1, 2, 4 and 8 threads, each writing and syncing its own file for 200 ms.
 *   5. Plausibility: a 4 KiB sync under 10 us, or any sync on a memory-backed file system, did not
 *      reach the media.
 *
 * Power-loss protection comes from /sys/block/<disk>/queue/write_cache on Linux and is "unknown"
 * under a hypervisor and on macOS, the same rule the engine uses. */
#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/uio.h>
#include <time.h>
#include <unistd.h>
#ifdef __APPLE__
#include <sys/mount.h>
#include <sys/param.h>
#else
#include <sys/statfs.h>
#include <sys/sysmacros.h>
#endif

#define IMPLAUSIBLE_NS 10000ULL
#define SCRATCH (64ULL << 20)
#define BANDWIDTH (256ULL << 20)
#define WINDOW_NS 200000000ULL

enum call { FSYNC, FDATASYNC, FULLFSYNC, BARRIERFSYNC, DSYNC_WRITE };
static const char *NAMES[] = {"fsync", "fdatasync", "F_FULLFSYNC", "F_BARRIERFSYNC", "RWF_DSYNC"};

#if defined(__APPLE__)
static const enum call CALLS[] = {FULLFSYNC, BARRIERFSYNC, FSYNC};
#elif defined(__linux__) && defined(RWF_DSYNC)
static const enum call CALLS[] = {FDATASYNC, FSYNC, DSYNC_WRITE};
#elif defined(__linux__)
static const enum call CALLS[] = {FDATASYNC, FSYNC};
#else
static const enum call CALLS[] = {FSYNC};
#endif
#define NCALLS (sizeof CALLS / sizeof CALLS[0])

static const char *dir;
static char scratch_names[16][4096];
static int scratch_count;

static void die(const char *what) {
    fprintf(stderr, "device-card: %s: %s\n", what, strerror(errno));
    for (int i = 0; i < scratch_count; i++) unlink(scratch_names[i]);
    exit(1);
}

static uint64_t now_ns(void) {
    struct timespec t;
    clock_gettime(CLOCK_MONOTONIC, &t);
    return (uint64_t)t.tv_sec * 1000000000ULL + (uint64_t)t.tv_nsec;
}

static int scratch(const char *tag) {
    if (scratch_count == 16) {
        errno = EMFILE;
        die("too many scratch files");
    }
    char *name = scratch_names[scratch_count++];
    snprintf(name, 4096, "%s/.rudb-bench-device-card-%d-%s", dir, (int)getpid(), tag);
    int fd = open(name, O_RDWR | O_CREAT | O_TRUNC, 0600);
    if (fd < 0) die(name);
    return fd;
}

static void write_all(int fd, const char *block, size_t len, off_t at) {
    while (len > 0) {
        ssize_t n = pwrite(fd, block, len, at);
        if (n < 0) {
            if (errno == EINTR) continue;
            die("pwrite");
        }
        block += n;
        len -= (size_t)n;
        at += n;
    }
}

static void sync_with(int fd, enum call call) {
    int r = 0;
    switch (call) {
    case FSYNC: r = fsync(fd); break;
#ifdef __linux__
    case FDATASYNC: r = fdatasync(fd); break;
#endif
#ifdef __APPLE__
    case FULLFSYNC: r = fcntl(fd, F_FULLFSYNC); break;
    case BARRIERFSYNC: r = fcntl(fd, 85 /* F_BARRIERFSYNC */); break;
#endif
    default: r = fsync(fd); break;
    }
    if (r != 0) die(NAMES[call]);
}

/* One write made durable by `call`. RWF_DSYNC is the write itself. */
static void write_and_sync(int fd, enum call call, const char *block, size_t len, off_t at) {
#if defined(__linux__) && defined(RWF_DSYNC)
    if (call == DSYNC_WRITE) {
        struct iovec v = {(void *)block, len};
        ssize_t n = pwritev2(fd, &v, 1, at, RWF_DSYNC);
        if (n != (ssize_t)len) die("pwritev2");
        return;
    }
#endif
    write_all(fd, block, len, at);
    sync_with(fd, call);
}

static int by_value(const void *a, const void *b) {
    uint64_t x = *(const uint64_t *)a, y = *(const uint64_t *)b;
    return x < y ? -1 : x > y;
}

/* The sample at quantile q, rounding up to a sample, as the engine's card does. */
static uint64_t quantile(const uint64_t *sorted, size_t n, double q) {
    if (n == 0) return 0;
    size_t rank = (size_t)(q * (double)n + 0.999999999);
    if (rank < 1) rank = 1;
    if (rank > n) rank = n;
    return sorted[rank - 1];
}

static void timed(int fd, enum call call, size_t size, unsigned iterations, uint64_t *p50,
                  uint64_t *p99) {
    char *block = malloc(size);
    uint64_t *samples = malloc(sizeof(uint64_t) * iterations);
    if (!block || !samples) die("malloc");
    memset(block, 0xa5, size);
    off_t at = 0;
    /* One untimed round first, so the first timed one is not also the first touch. */
    write_and_sync(fd, call, block, size, at);
    for (unsigned i = 0; i < iterations; i++) {
        at = (at + (off_t)size) % (off_t)(SCRATCH - size);
        uint64_t start = now_ns();
        write_and_sync(fd, call, block, size, at);
        samples[i] = now_ns() - start;
    }
    qsort(samples, iterations, sizeof(uint64_t), by_value);
    *p50 = quantile(samples, iterations, 0.5);
    *p99 = quantile(samples, iterations, 0.99);
    free(block);
    free(samples);
}

struct worker {
    int fd;
    enum call call;
    uint64_t until;
    uint64_t syncs;
};

static void *parallel_worker(void *arg) {
    struct worker *w = arg;
    char block[4096];
    memset(block, 0x5a, sizeof block);
    off_t at = 0;
    while (now_ns() < w->until) {
        write_and_sync(w->fd, w->call, block, sizeof block, at);
        at = (at + 4096) % (off_t)(4ULL << 20);
        w->syncs++;
    }
    return NULL;
}

static uint64_t parallel(int threads, enum call call) {
    struct worker workers[8];
    pthread_t ids[8];
    char block[1 << 16];
    memset(block, 0, sizeof block);
    for (int t = 0; t < threads; t++) {
        char tag[32];
        snprintf(tag, sizeof tag, "scale%d-%d", threads, t);
        workers[t].fd = scratch(tag);
        for (off_t at = 0; at < (off_t)(4ULL << 20); at += sizeof block)
            write_all(workers[t].fd, block, sizeof block, at);
        sync_with(workers[t].fd, FSYNC);
        workers[t].call = call;
        workers[t].syncs = 0;
    }
    uint64_t start = now_ns();
    for (int t = 0; t < threads; t++) {
        workers[t].until = start + WINDOW_NS;
        if (pthread_create(&ids[t], NULL, parallel_worker, &workers[t]) != 0) die("pthread_create");
    }
    uint64_t syncs = 0;
    for (int t = 0; t < threads; t++) {
        pthread_join(ids[t], NULL);
        syncs += workers[t].syncs;
        close(workers[t].fd);
    }
    uint64_t elapsed = now_ns() - start;
    for (int t = 0; t < threads; t++) unlink(scratch_names[--scratch_count]);
    return elapsed ? syncs * 1000000000ULL / elapsed : 0;
}

/* The file system type, the device name, and whether it keeps its files in memory. */
static void mount_of(char *device, size_t dlen, char *fs, size_t flen, int *memory) {
    device[0] = fs[0] = 0;
    *memory = 0;
#ifdef __APPLE__
    struct statfs s;
    if (statfs(dir, &s) == 0) {
        snprintf(device, dlen, "%s", s.f_mntfromname);
        snprintf(fs, flen, "%s", s.f_fstypename);
    }
#else
    /* The mount whose point is the longest prefix of the directory, from /proc/self/mounts. */
    char real[4096];
    if (!realpath(dir, real)) return;
    FILE *mounts = fopen("/proc/self/mounts", "r");
    if (!mounts) return;
    char src[4096], point[4096], type[256];
    size_t best = 0;
    while (fscanf(mounts, "%4095s %4095s %255s %*[^\n]", src, point, type) == 3) {
        size_t len = strlen(point);
        int under = strncmp(real, point, len) == 0 &&
                    (real[len] == '/' || real[len] == 0 || (len == 1 && point[0] == '/'));
        if (under && len >= best) {
            best = len;
            snprintf(device, dlen, "%s", src);
            snprintf(fs, flen, "%s", type);
        }
    }
    fclose(mounts);
#endif
    *memory = !strcmp(fs, "tmpfs") || !strcmp(fs, "ramfs") || !strcmp(fs, "devtmpfs");
}

static const char *plp_of(const char *device) {
#ifdef __linux__
    FILE *cpu = fopen("/proc/cpuinfo", "r");
    if (cpu) {
        char line[8192];
        int virtualised = 0;
        while (fgets(line, sizeof line, cpu))
            if (!strncmp(line, "flags", 5) && strstr(line, " hypervisor")) virtualised = 1;
        fclose(cpu);
        if (virtualised) return "unknown";
    }
    if (strncmp(device, "/dev/", 5) != 0) return "unknown";
    char path[4200], node[4096];
    snprintf(path, sizeof path, "/sys/class/block/%s", device + 5);
    if (!realpath(path, node)) return "unknown";
    /* A partition's directory sits inside its disk's, and only the disk has a queue. */
    for (int up = 0; up < 2; up++) {
        char file[4200], cache[64] = {0};
        snprintf(file, sizeof file, "%s/queue/write_cache", node);
        FILE *f = fopen(file, "r");
        if (f) {
            if (!fgets(cache, sizeof cache, f)) cache[0] = 0;
            fclose(f);
            if (!strncmp(cache, "write through", 13)) return "yes";
            if (!strncmp(cache, "write back", 10)) return "no";
            return "unknown";
        }
        char *slash = strrchr(node, '/');
        if (!slash) break;
        *slash = 0;
    }
#endif
    (void)device;
    return "unknown";
}

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: device-card DIR [ITERATIONS]\n");
        return 2;
    }
    dir = argv[1];
    unsigned iterations = argc > 2 ? (unsigned)strtoul(argv[2], NULL, 10) : 200;
    if (iterations == 0) iterations = 1;
    struct stat st;
    if (stat(dir, &st) != 0 || !S_ISDIR(st.st_mode)) {
        errno = ENOTDIR;
        die(dir);
    }
    char device[4096], fs[256];
    int memory;
    mount_of(device, sizeof device, fs, sizeof fs, &memory);

    int fd = scratch("sync");
    {
        char *block = calloc(1, 1 << 20);
        if (!block) die("calloc");
        for (off_t at = 0; at < (off_t)SCRATCH; at += 1 << 20) write_all(fd, block, 1 << 20, at);
        free(block);
        sync_with(fd, FSYNC);
    }
    uint64_t p50_4k[NCALLS], p99_4k[NCALLS], p50_64k[NCALLS], p99_64k[NCALLS];
    for (size_t c = 0; c < NCALLS; c++) {
        timed(fd, CALLS[c], 4 << 10, iterations, &p50_4k[c], &p99_4k[c]);
        timed(fd, CALLS[c], 64 << 10, iterations, &p50_64k[c], &p99_64k[c]);
    }
    close(fd);
    unlink(scratch_names[--scratch_count]);

    fd = scratch("bandwidth");
    char *block = malloc(1 << 20);
    if (!block) die("malloc");
    memset(block, 0x3c, 1 << 20);
    uint64_t start = now_ns();
    for (off_t at = 0; at < (off_t)BANDWIDTH; at += 1 << 20) write_all(fd, block, 1 << 20, at);
    sync_with(fd, CALLS[0] == DSYNC_WRITE ? FSYNC : CALLS[0]);
    uint64_t elapsed = now_ns() - start;
    free(block);
    close(fd);
    unlink(scratch_names[--scratch_count]);
    double write_mib_s = elapsed ? (double)BANDWIDTH / (1 << 20) / ((double)elapsed / 1e9) : 0;

    static const int THREADS[] = {1, 2, 4, 8};
    uint64_t syncs[4];
    for (int i = 0; i < 4; i++) syncs[i] = parallel(THREADS[i], CALLS[0]);
    double scaling = syncs[0] ? (double)syncs[3] / (double)(syncs[0] * 8) : 0;

    printf("{\"path\":\"%s\",\"device\":\"%s\",\"filesystem\":\"%s\",\"memory_backed\":%s,"
           "\"plp\":\"%s\",\"iterations\":%u,\"write_mib_s\":%.1f,"
           "\"syncs_1\":%llu,\"syncs_2\":%llu,\"syncs_4\":%llu,\"syncs_8\":%llu,"
           "\"scaling\":%.3f,\"calls\":[",
           dir, device, fs, memory ? "true" : "false", plp_of(device), iterations, write_mib_s,
           (unsigned long long)syncs[0], (unsigned long long)syncs[1],
           (unsigned long long)syncs[2], (unsigned long long)syncs[3], scaling);
    for (size_t c = 0; c < NCALLS; c++) {
        printf("%s{\"sync_call\":\"%s\",\"chosen\":%s,\"p50_4k_us\":%.3f,\"p99_4k_us\":%.3f,"
               "\"p50_64k_us\":%.3f,\"p99_64k_us\":%.3f,\"plausible\":%s}",
               c ? "," : "", NAMES[CALLS[c]], c == 0 ? "true" : "false", p50_4k[c] / 1e3,
               p99_4k[c] / 1e3, p50_64k[c] / 1e3, p99_64k[c] / 1e3,
               !memory && p50_4k[c] >= IMPLAUSIBLE_NS ? "true" : "false");
    }
    printf("]}\n");
    return 0;
}
