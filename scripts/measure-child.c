/* Linux-only resource reader. A small native parent avoids charging the Python
 * orchestrator's pre-exec resident pages to small children. It records wait4's
 * per-child counters, not the cumulative RUSAGE_CHILDREN high water mark. */
#define _GNU_SOURCE
#include <errno.h>
#include <spawn.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/resource.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>
extern char **environ;

int main(int argc, char **argv) {
    if (argc < 3) return 125;
    struct timespec start, end;
    struct rusage usage;
    pid_t pid;
    int status;
    clock_gettime(CLOCK_MONOTONIC, &start);
    int error = posix_spawnp(&pid, argv[2], NULL, NULL, argv + 2, environ);
    if (error) { errno = error; perror("posix_spawnp"); return 125; }
    while (wait4(pid, &status, 0, &usage) < 0) {
        if (errno != EINTR) { perror("wait4"); return 125; }
    }
    clock_gettime(CLOCK_MONOTONIC, &end);
    FILE *out = fopen(argv[1], "w");
    if (!out) { perror("resource report"); return 125; }
    int code = WIFEXITED(status) ? WEXITSTATUS(status) : -WTERMSIG(status);
    fprintf(out, "{\"exit_code\":%d,\"wall_s\":%.9f,\"user_s\":%.6f,\"system_s\":%.6f,"
        "\"peak_rss_bytes\":%lld,\"read_bytes\":%lld,\"write_bytes\":%lld,"
        "\"major_faults\":%ld,\"minor_faults\":%ld,"
        "\"voluntary_switches\":%ld,\"involuntary_switches\":%ld}\n",
        code, (end.tv_sec-start.tv_sec)+(end.tv_nsec-start.tv_nsec)/1e9,
        usage.ru_utime.tv_sec+usage.ru_utime.tv_usec/1e6,
        usage.ru_stime.tv_sec+usage.ru_stime.tv_usec/1e6,
        (long long)usage.ru_maxrss*1024, (long long)usage.ru_inblock*512,
        (long long)usage.ru_oublock*512, usage.ru_majflt, usage.ru_minflt,
        usage.ru_nvcsw, usage.ru_nivcsw);
    if (fclose(out)) return 125;
    return code < 0 ? 128-code : code;
}
