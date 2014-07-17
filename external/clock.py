import time

while True:
    iters = int(input())

    # Setup

    # Bench
    start = time.monotonic()
    for _ in range(iters):
        time.monotonic()
    end = time.monotonic()

    # Teardown

    # Report
    print(int(1e9 * (end - start)))
