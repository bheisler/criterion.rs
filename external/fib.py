import sys
import time

def fib(n):
    return fib(n - 1) + fib(n - 2) if n > 1 else n + 1

n = int(sys.argv[1])

while True:
    iters = int(input())

    # Setup

    # Bench
    start = time.monotonic()
    for _ in range(iters):
        fib(n)
    end = time.monotonic()

    # Teardown

    # Report
    print(int(1e9 * (end - start)))
