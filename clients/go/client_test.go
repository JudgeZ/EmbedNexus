package goclient_test

import (
	"encoding/json"
	"flag"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"testing"
)

var updateTranscripts = flag.Bool("update-transcripts", false, "regenerate golden transcripts for Go client tests")

func fixtureRoot(tb testing.TB) string {
	tb.Helper()
	_, filename, _, ok := runtime.Caller(0)
	if !ok {
		tb.Fatalf("unable to resolve caller information")
	}

	return filepath.Join(filepath.Dir(filename), "..", "..", "tests", "fixtures", "go")
}

func TestGoClientTranscripts(t *testing.T) {
	transports := []string{"stdio", "http", "tls"}
	fixtures := fixtureRoot(t)

	for _, transport := range transports {
		transport := transport
		t.Run(transport, func(t *testing.T) {
			requestPath := filepath.Join(fixtures, transport, "request.json")
			responsePath := filepath.Join(fixtures, transport, "response.json")

			requestPayload := readJSON(t, requestPath)
			responsePayload := readJSON(t, responsePath)

			if got := requestPayload["params"].(map[string]interface{})["transport"]; got != transport {
				t.Fatalf("unexpected transport in request fixture: %v", got)
			}

			if responsePayload["id"] != requestPayload["id"] {
				t.Fatalf("response id %v did not match request id %v", responsePayload["id"], requestPayload["id"])
			}

			if *updateTranscripts {
				t.Skip("transcript regeneration not yet implemented for Go client tests")
			}

			cmd := exec.Command("go", "run", "./clients/go", "--transport", transport)
			cmd.Dir = repositoryRoot(t)
			cmd.Env = os.Environ()
			output, err := cmd.CombinedOutput()
			if err == nil {
				t.Fatalf("not yet implemented: expected Go client CLI to be missing, got success: %s", string(output))
			}

			t.Fatalf("not yet implemented: Go client CLI execution placeholder (%v)\n%s", err, string(output))
		})
	}
}

func readJSON(tb testing.TB, path string) map[string]interface{} {
	tb.Helper()
	data, err := os.ReadFile(path)
	if err != nil {
		tb.Fatalf("failed to read fixture %s: %v", path, err)
	}

	var payload map[string]interface{}
	if err := json.Unmarshal(data, &payload); err != nil {
		tb.Fatalf("failed to parse fixture %s: %v", path, err)
	}

	return payload
}

func repositoryRoot(tb testing.TB) string {
	tb.Helper()
	_, filename, _, ok := runtime.Caller(0)
	if !ok {
		tb.Fatalf("unable to resolve caller information")
	}

	return filepath.Join(filepath.Dir(filename), "..", "..")
}
