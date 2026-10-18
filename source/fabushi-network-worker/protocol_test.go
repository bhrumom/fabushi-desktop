package main

import (
	"bufio"
	"encoding/base64"
	"errors"
	"path/filepath"
	"strings"
	"testing"
)

func validInitializeRequest(t *testing.T) workerRequest {
	t.Helper()
	return workerRequest{
		Type:            "initialize",
		ProtocolVersion: protocolVersion,
		Hostname:        "device-node-01234567",
		ControlURL:      "https://coord.example.test",
		AuthKey:         "enrollment-secret",
		StateDir:        filepath.Join(t.TempDir(), "state"),
		StateKeyB64:     base64.StdEncoding.EncodeToString(make([]byte, 32)),
	}
}


func TestReadRequestContinuesAfterRecoverableMalformedFrame(t *testing.T) {
	scanner := bufio.NewScanner(strings.NewReader("{invalid-json}\\n{\\"type\\":\\"status\\"}\\n"))
	scanner.Buffer(make([]byte, 16), maxControlFrameBytes)
	if _, err := readRequest(scanner); err == nil {
		t.Fatal("malformed JSON frame unexpectedly accepted")
	}
	request, err := readRequest(scanner)
	if err != nil {
		t.Fatalf("valid frame after malformed frame was not read: %v", err)
	}
	if request.Type != "status" {
		t.Fatalf("unexpected request type: %q", request.Type)
	}
}

func TestReadRequestMarksOversizedFrameAsTerminal(t *testing.T) {
	input := strings.Repeat("x", maxControlFrameBytes+1) + "\\n"
	scanner := bufio.NewScanner(strings.NewReader(input))
	scanner.Buffer(make([]byte, 4096), maxControlFrameBytes)
	_, err := readRequest(scanner)
	if err == nil {
		t.Fatal("oversized frame unexpectedly accepted")
	}
	var terminal terminalReadError
	if !errors.As(err, &terminal) {
		t.Fatalf("oversized frame must be terminal, got %T: %v", err, err)
	}
	if terminal.Error() != "terminal_input_error" {
		t.Fatalf("terminal error classification leaked input details: %q", terminal.Error())
	}
}

func TestValidateInitializeAcceptsPrivateTsnetConfiguration(t *testing.T) {
	request := validInitializeRequest(t)
	config, err := validateInitialize(request)
	if err != nil {
		t.Fatal(err)
	}
	if config.Hostname != request.Hostname || config.ControlURL != request.ControlURL+"/" {
		t.Fatalf("unexpected normalized config: %#v", config)
	}
	if len(config.StateKey) != 32 {
		t.Fatalf("unexpected state key length %d", len(config.StateKey))
	}
}

func TestValidateInitializeRejectsUnsafeControlURLAndKey(t *testing.T) {
	for name, mutate := range map[string]func(*workerRequest){
		"http":           func(r *workerRequest) { r.ControlURL = "http://coord.example.test" },
		"userinfo":       func(r *workerRequest) { r.ControlURL = "https://token@coord.example.test" },
		"relative-state": func(r *workerRequest) { r.StateDir = "relative/state" },
		"short-key":      func(r *workerRequest) { r.StateKeyB64 = base64.StdEncoding.EncodeToString(make([]byte, 16)) },
		"bad-hostname":   func(r *workerRequest) { r.Hostname = "node with spaces" },
	} {
		t.Run(name, func(t *testing.T) {
			request := validInitializeRequest(t)
			mutate(&request)
			if _, err := validateInitialize(request); err == nil {
				t.Fatal("unsafe initialize request was accepted")
			}
		})
	}
}

func TestProtocolErrorsNeverEchoSecretMaterial(t *testing.T) {
	secret := "super-secret-auth-key"
	event := redactError(errors.New("upstream failed with " + secret))
	if strings.Contains(event.Message, secret) || strings.Contains(event.Code, secret) {
		t.Fatal("wire error leaked secret material")
	}
}

func TestRequestIDIsBoundedAndOpaque(t *testing.T) {
	if !validRequestID("request-0123456789") {
		t.Fatal("valid request id rejected")
	}
	for _, value := range []string{"short", "contains space 123456", strings.Repeat("x", 161)} {
		if validRequestID(value) {
			t.Fatalf("invalid request id accepted: %q", value)
		}
	}
}

func TestStreamTargetIsTailnetOnlyAndBounded(t *testing.T) {
	for _, peer := range []string{"100.64.0.10", "100.127.255.254", "fd7a:115c:a1e0::1234"} {
		if _, ok := streamAddress(peer, 443); !ok {
			t.Fatalf("valid tailnet target rejected: %s", peer)
		}
	}
	for _, peer := range []string{"127.0.0.1", "10.0.0.1", "192.168.1.1", "8.8.8.8", "2001:4860:4860::8888", "example.com"} {
		if _, ok := streamAddress(peer, 443); ok {
			t.Fatalf("non-tailnet target accepted: %s", peer)
		}
	}
	if _, ok := streamAddress("100.64.0.10", 0); ok {
		t.Fatal("zero port accepted")
	}
	if _, ok := streamTimeoutMS(99); ok {
		t.Fatal("sub-minimum timeout accepted")
	}
	if value, ok := streamTimeoutMS(0); !ok || value != 5000 {
		t.Fatalf("default timeout mismatch: %d %v", value, ok)
	}
}

func TestStreamChunkAndReadBoundsAreFailClosed(t *testing.T) {
	okChunk := base64.StdEncoding.EncodeToString(make([]byte, maxStreamChunkBytes))
	if data, ok := decodeStreamChunk(okChunk); !ok || len(data) != maxStreamChunkBytes {
		t.Fatal("maximum stream chunk was rejected")
	}
	tooLarge := base64.StdEncoding.EncodeToString(make([]byte, maxStreamChunkBytes+1))
	if _, ok := decodeStreamChunk(tooLarge); ok {
		t.Fatal("oversized stream chunk was accepted")
	}
	if _, ok := decodeStreamChunk("***"); ok {
		t.Fatal("invalid base64 stream chunk was accepted")
	}
	if _, ok := boundedReadSize(maxStreamChunkBytes + 1); ok {
		t.Fatal("oversized stream read was accepted")
	}
}

type protocolTestAddr string

func (a protocolTestAddr) Network() string { return "tcp" }
func (a protocolTestAddr) String() string  { return string(a) }

func TestPeerBoundListenValidation(t *testing.T) {
	if address, ok := listenAddress(8443); !ok || address != ":8443" {
		t.Fatalf("listen address mismatch: %q %v", address, ok)
	}
	if _, ok := listenAddress(0); ok {
		t.Fatal("ephemeral/unbound port unexpectedly accepted")
	}
	if !remotePeerMatches(protocolTestAddr("100.64.0.30:51234"), "100.64.0.30") {
		t.Fatal("expected Tailnet peer did not match")
	}
	if remotePeerMatches(protocolTestAddr("100.64.0.31:51234"), "100.64.0.30") {
		t.Fatal("different Tailnet peer unexpectedly matched")
	}
	if remotePeerMatches(protocolTestAddr("10.0.0.5:51234"), "100.64.0.30") {
		t.Fatal("non-Tailnet remote peer unexpectedly matched")
	}
}
