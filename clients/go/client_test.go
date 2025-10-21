package main

import (
	"context"
	"encoding/json"
	"flag"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"testing"
	"time"
)

var updateTranscripts = flag.Bool("update-transcripts", false, "regenerate client transcripts")

func fixturePath(t *testing.T, transport, kind string) string {
	t.Helper()
	_, filename, _, ok := runtime.Caller(0)
	if !ok {
		t.Fatalf("unable to resolve caller path")
	}
	repoRoot := filepath.Dir(filepath.Dir(filepath.Dir(filename)))
	return filepath.Join(repoRoot, "tests", "fixtures", "go", transport, kind+".json")
}

func loadFixture(t *testing.T, transport, kind string) map[string]any {
        t.Helper()
        path := fixturePath(t, transport, kind)
        content, err := os.ReadFile(path)
        if err != nil {
                if os.IsNotExist(err) {
                        t.Skipf("download GitHub Action artifact to populate %s fixtures", transport)
                }
                t.Fatalf("failed to read %s: %v", path, err)
        }
	var payload map[string]any
	if err := json.Unmarshal(content, &payload); err != nil {
		t.Fatalf("failed to unmarshal %s: %v", path, err)
	}
	return payload
}

func TestGoClientTranscripts(t *testing.T) {
	transports := []string{"stdio", "http", "tls"}
	_, filename, _, ok := runtime.Caller(0)
	if !ok {
		t.Fatalf("unable to resolve caller path")
	}
	repoRoot := filepath.Dir(filepath.Dir(filepath.Dir(filename)))

	for _, transport := range transports {
		transport := transport
		t.Run(transport, func(t *testing.T) {
			requestPayload := loadFixture(t, transport, "request")
			responsePayload := loadFixture(t, transport, "response")

			if requestPayload["client"] != "go" {
				t.Fatalf("unexpected client marker: %v", requestPayload["client"])
			}
			if responsePayload["transport"] != transport {
				t.Fatalf("unexpected transport marker: %v", responsePayload["transport"])
			}

			cliArgs := []string{
				"run",
				filepath.Join(repoRoot, "clients", "go"),
				"--transport", transport,
				"--record-transcript",
				filepath.Join(repoRoot, "artifacts", "go", transport+".json"),
			}
			if *updateTranscripts {
				cliArgs = append(cliArgs, "--update-transcripts")
			}

			ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
			defer cancel()
			cmd := exec.CommandContext(ctx, "go", cliArgs...)
			cmd.Dir = repoRoot
			if err := cmd.Run(); err == nil {
				t.Fatalf("expected command failure while implementation is pending")
			}
			if ctx.Err() == context.DeadlineExceeded {
				t.Fatalf("command timed out before failing as expected")
			}

			t.Fatalf("not yet implemented: go client subprocess invocation and transcript diffing (update=%v)", *updateTranscripts)
		})
	}
}
