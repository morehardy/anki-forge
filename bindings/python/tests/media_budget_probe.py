"""Fresh-process memory probe used by the installed-wheel public API tests."""
import sys

from anki_forge import Media, MediaError, MediaLimits


if sys.platform == "win32":
    import ctypes
    from ctypes import wintypes

    class ProcessMemoryCounters(ctypes.Structure):
        _fields_ = [("cb", wintypes.DWORD), ("PageFaultCount", wintypes.DWORD)] + [
            (name, ctypes.c_size_t) for name in (
                "PeakWorkingSetSize", "WorkingSetSize", "QuotaPeakPagedPoolUsage",
                "QuotaPagedPoolUsage", "QuotaPeakNonPagedPoolUsage",
                "QuotaNonPagedPoolUsage", "PagefileUsage", "PeakPagefileUsage",
            )
        ]

    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    psapi = ctypes.WinDLL("psapi", use_last_error=True)
    kernel32.GetCurrentProcess.restype = wintypes.HANDLE
    psapi.GetProcessMemoryInfo.argtypes = [
        wintypes.HANDLE, ctypes.POINTER(ProcessMemoryCounters), wintypes.DWORD,
    ]
    psapi.GetProcessMemoryInfo.restype = wintypes.BOOL

    def peak_rss():
        counters = ProcessMemoryCounters()
        counters.cb = ctypes.sizeof(counters)
        if not psapi.GetProcessMemoryInfo(kernel32.GetCurrentProcess(), ctypes.byref(counters), counters.cb):
            raise ctypes.WinError(ctypes.get_last_error())
        return counters.PeakWorkingSetSize
else:
    import resource

    def peak_rss():
        return resource.getrusage(resource.RUSAGE_SELF).ru_maxrss * (1 if sys.platform == "darwin" else 1024)


use_default = sys.argv[1] == "default"
limit = 256 << 20 if use_default else 1024
# Warm native loading and allocate/fault in caller storage before the baseline.
Media.bytes(b"x", "application/octet-stream")
data = b"x" * (limit + 1 if use_default else 64 << 20)
before = peak_rss()
try:
    Media.bytes(data, "application/octet-stream", **({} if use_default else {"limits": MediaLimits(limit)}))
except MediaError as error:
    assert error.kind == "ResourceLimit"
    assert error.code == "MEDIA.RESOURCE_LIMIT_EXCEEDED"
    assert error.details == {
        "error_kind": "ResourceLimit", "causes": [], "source_details": [], "path": None,
        "limit_exceeded": {"resource": "media_bytes", "limit": limit, "observed": len(data)},
    }
    assert error.__cause__ is not None
    assert error.__cause__.__cause__ is None
else:
    raise AssertionError("over-budget media was accepted")
overhead = peak_rss() - before
assert overhead < 32 << 20, f"rejecting over-budget bytes allocated {overhead} resident bytes"
