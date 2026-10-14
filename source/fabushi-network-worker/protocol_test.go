package main

import (
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
