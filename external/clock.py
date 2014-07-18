import ctypes
import sys
import time

CLOCK_MONOTONIC_RAW = 4

class timespec(ctypes.Structure):
    _fields_ = [
        ('tv_sec', ctypes.c_long),
        ('tv_nsec', ctypes.c_long)
    ]

librt = ctypes.CDLL('librt.so.1', use_errno=True)
clock_gettime = librt.clock_gettime
clock_gettime.argtypes = [ctypes.c_int, ctypes.POINTER(timespec)]

while True:
    iters = int(input())

    # Setup
    start, end, dummy = timespec(), timespec(), timespec()

    # Bench
    clock_gettime(CLOCK_MONOTONIC_RAW, ctypes.byref(start))
    for _ in range(iters):
        clock_gettime(CLOCK_MONOTONIC_RAW, ctypes.byref(dummy))
    clock_gettime(CLOCK_MONOTONIC_RAW, ctypes.byref(end))

    # Teardown

    # Report
    secs = end.tv_sec - start.tv_sec
    nsecs = end.tv_nsec - start.tv_nsec
    print(secs * 1000000000 + nsecs)
    sys.stdout.flush()
