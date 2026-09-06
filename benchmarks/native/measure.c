/* Minimal native launch/reap owner. No inherited Python high-water RSS.
 * Timing excludes this collector's startup and starts at direct posix_spawn.
 * Diagnostics are drained continuously, retaining at most 64 KiB per stream. */
#define _DEFAULT_SOURCE
#define _DARWIN_C_SOURCE
#define _POSIX_C_SOURCE 200809L
#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <signal.h>
#include <spawn.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/resource.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

extern char **environ;
static volatile sig_atomic_t child_pid = 0, interrupted = 0;
static void stop(int sig) {
    interrupted = sig;
    if (child_pid > 0) kill(-(pid_t)child_pid, SIGKILL);
}
static uint64_t now_ns(void) {
    struct timespec t;
    clock_gettime(CLOCK_MONOTONIC, &t);
    return (uint64_t)t.tv_sec * 1000000000ULL + t.tv_nsec;
}
struct drain { int fd; FILE *log; size_t total; };
static void *drain_log(void *arg) {
    struct drain *d = arg;
    FILE *out = d->log;
    char buf[8192];
    ssize_t n;
    while ((n = read(d->fd, buf, sizeof buf)) != 0) {
        if (n < 0) { if (errno == EINTR) continue; break; }
        size_t keep = d->total < 65536 ? 65536 - d->total : 0;
        if (keep > (size_t)n) keep = (size_t)n;
        if (out && keep) fwrite(buf, 1, keep, out);
        d->total += (size_t)n;
    }
    if (out) fclose(out);
    close(d->fd);
    return NULL;
}
int main(int argc, char **argv) {
    if (argc < 5) { fprintf(stderr, "usage: measure TIMEOUT OUTLOG ERRLOG EXEC [ARGS...]\n"); return 2; }
    int p_out[2], p_err[2];
    if (pipe(p_out) || pipe(p_err)) return 3;
    /* Avoid inherited unrelated descriptors. Python also launches us close_fds. */
    fcntl(p_out[0], F_SETFD, FD_CLOEXEC); fcntl(p_err[0], F_SETFD, FD_CLOEXEC);
    fcntl(p_out[1], F_SETFD, FD_CLOEXEC); fcntl(p_err[1], F_SETFD, FD_CLOEXEC);
    /* Complete log creation before threads, timing, and exporter launch. */
    FILE *outlog = fopen(argv[2], "wb");
    if (!outlog) { perror("open stdout log"); return 3; }
    FILE *errlog = fopen(argv[3], "wb");
    if (!errlog) { perror("open stderr log"); fclose(outlog); return 3; }
    if (fcntl(fileno(outlog), F_SETFD, FD_CLOEXEC) == -1 ||
        fcntl(fileno(errlog), F_SETFD, FD_CLOEXEC) == -1) {
        perror("close-on-exec log"); fclose(outlog); fclose(errlog); return 3;
    }
    struct drain out = {p_out[0], outlog, 0}, err = {p_err[0], errlog, 0};
    pthread_t tout, terr;
    if (pthread_create(&tout, NULL, drain_log, &out) || pthread_create(&terr, NULL, drain_log, &err)) return 3;
    struct sigaction sa = {0};
    sa.sa_handler = stop; sigemptyset(&sa.sa_mask);
    sigaction(SIGALRM, &sa, NULL); sigaction(SIGINT, &sa, NULL); sigaction(SIGTERM, &sa, NULL);
    posix_spawn_file_actions_t actions;
    posix_spawn_file_actions_init(&actions);
    posix_spawn_file_actions_adddup2(&actions, p_out[1], STDOUT_FILENO);
    posix_spawn_file_actions_adddup2(&actions, p_err[1], STDERR_FILENO);
    posix_spawnattr_t attr;
    posix_spawnattr_init(&attr);
    posix_spawnattr_setflags(&attr, POSIX_SPAWN_SETPGROUP);
    posix_spawnattr_setpgroup(&attr, 0);
    pid_t pid = 0;
    struct rusage usage = {0};
    int status = 0;
    alarm((unsigned)atoi(argv[1]));
    uint64_t start = now_ns();
    int spawn_error = posix_spawn(&pid, argv[4], &actions, &attr, argv + 4, environ);
    child_pid = pid;
    if (interrupted && pid > 0) kill(-pid, SIGKILL);
    close(p_out[1]); close(p_err[1]);
    pid_t reaped = -1;
    if (!spawn_error) {
        do { reaped = wait4(pid, &status, 0, &usage); } while (reaped < 0 && errno == EINTR);
    }
    uint64_t end = now_ns();
    alarm(0);
    int leftover = pid > 0 && kill(-pid, 0) == 0;
    if (leftover) kill(-pid, SIGKILL);
    pthread_join(tout, NULL); pthread_join(terr, NULL);
    posix_spawn_file_actions_destroy(&actions); posix_spawnattr_destroy(&attr);
    struct timespec resolution;
    clock_getres(CLOCK_MONOTONIC, &resolution);
#ifdef __APPLE__
    const char *unit = "bytes";
    unsigned long long rss = (unsigned long long)usage.ru_maxrss;
#else
    const char *unit = "KiB";
    unsigned long long rss = (unsigned long long)usage.ru_maxrss * 1024ULL;
#endif
    printf("{\"elapsed_ns\":%llu,\"start_monotonic_ns\":%llu,\"end_monotonic_ns\":%llu,"
           "\"clock_resolution_ns\":%llu,\"pid\":%d,\"spawn_error\":%d,\"reaped\":%s,"
           "\"exit_code\":%d,\"signal\":%d,\"interrupted_signal\":%d,\"leftover_descendants\":%s,"
           "\"peak_rss_raw\":%ld,\"peak_rss_raw_unit\":\"%s\",\"peak_rss_bytes\":%llu,"
           "\"stdout_bytes\":%zu,\"stderr_bytes\":%zu}\n",
           (unsigned long long)(end-start), (unsigned long long)start, (unsigned long long)end,
           (unsigned long long)resolution.tv_sec * 1000000000ULL + resolution.tv_nsec,
           pid, spawn_error, reaped == pid && pid > 0 ? "true" : "false",
           !spawn_error && WIFEXITED(status) ? WEXITSTATUS(status) : -1,
           !spawn_error && WIFSIGNALED(status) ? WTERMSIG(status) : 0, interrupted,
           leftover ? "true" : "false", usage.ru_maxrss, unit, rss, out.total, err.total);
    return 0;
}
