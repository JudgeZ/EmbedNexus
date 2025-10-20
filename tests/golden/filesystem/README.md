# Filesystem Watcher Goldens

The latency burst golden mirrors the fixture regeneration flow. After recording
events, run:

```bash
python scripts/record_fs_events.py transcript \
  --scenario latency-burst \
  --output tests/golden/filesystem/watch-latency-burst.log
```

This transcript aligns with the Input Validation checklist by proving the
enumerator debounces events without leaking filtered paths. Capture commands and
host metadata (OS build, filesystem type) should accompany any update so
reviewers can reproduce the watcher latency profile.
