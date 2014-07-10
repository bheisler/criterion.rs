import sys
import time

while True:
    line = sys.stdin.readline()
    iters = int(line)

    # Setup

    # Bench
    start = time.monotonic()
    for _ in range(iters):
        time.monotonic()
    end = time.monotonic()

    # Teardown

    # Report
    print(int(1e9 * (end - start)))
