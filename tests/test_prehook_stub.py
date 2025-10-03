import time

def test_stub_budget():
    t0 = time.time()
    # Simulate under-budget path; no external calls
    time.sleep(0.01)
    assert (time.time() - t0) * 1000 < 50

