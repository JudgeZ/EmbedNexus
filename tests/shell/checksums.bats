#!/usr/bin/env bats

setup() {
  TMPDIR=$(mktemp -d)
}

teardown() {
  rm -rf "$TMPDIR"
}

@test "updates manifest for directories" {
  mkdir -p "$TMPDIR/fixtures/sub"
  echo "alpha" >"$TMPDIR/fixtures/a.txt"
  echo "beta" >"$TMPDIR/fixtures/sub/b.txt"

  run scripts/checksums.sh --update "$TMPDIR/fixtures"
  [ "$status" -eq 0 ]
  [ -f "$TMPDIR/fixtures.sha256" ]

  run cat "$TMPDIR/fixtures.sha256"
  [ "$status" -eq 0 ]
  [[ "$output" == *"a.txt"* ]]
  [[ "$output" == *"sub/b.txt"* ]]
}

@test "verify detects drift" {
  echo "alpha" >"$TMPDIR/sample.txt"
  run scripts/checksums.sh --update "$TMPDIR/sample.txt"
  [ "$status" -eq 0 ]

  echo "changed" >"$TMPDIR/sample.txt"
  run scripts/checksums.sh --verify "$TMPDIR/sample.txt"
  [ "$status" -ne 0 ]
}
