package main

import (
	"encoding/base64"
	"errors"
	"fmt"
	"net/url"
	"path/filepath"
	"strings"
)

const protocolVersion = 1

type workerRequest struct {
	Type            string `json:"type"`
	ProtocolVersion int    `json:"protocolVersion,omitempty"`
	Hostname        string `json:"hostname,omitempty"`
	ControlURL      string `json:"controlUrl,omitempty"`
	AuthKey         string `json:"authKey,omitempty"`
	StateDir        string `json:"stateDir,omitempty"`
	StateKeyB64     string `json:"stateKeyB64,omitempty"`
	RequestID       string `json:"requestId,omitempty"`
}

type workerEvent struct {
	Type         string   `json:"type"`
	RequestID    string   `json:"requestId,omitempty"`
	BackendState string   `json:"backendState,omitempty"`
	TailscaleIPs []string `json:"tailscaleIps,omitempty"`
	Code         string   `json:"code,omitempty"`
	Message      string   `json:"message,omitempty"`
}

type initializeConfig struct {
	Hostname   string
	ControlURL string
	AuthKey    string
	StateDir   string
	StateKey   []byte
}

func validateInitialize(request workerRequest) (initializeConfig, error) {
	if request.Type != "initialize" {
		return initializeConfig{}, errors.New("first request must be initialize")
	}
	if request.ProtocolVersion != protocolVersion {
		return initializeConfig{}, fmt.Errorf("unsupported protocolVersion %d", request.ProtocolVersion)
	}
	hostname := strings.TrimSpace(request.Hostname)
	if len(hostname) < 8 || len(hostname) > 128 {
		return initializeConfig{}, errors.New("hostname must be 8..128 characters")
	}
	for _, r := range hostname {
		if !(r == '-' || r == '_' || r == '.' || r == ':' ||
			(r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9')) {
			return initializeConfig{}, errors.New("hostname contains unsupported characters")
		}
	}
	control, err := url.Parse(strings.TrimSpace(request.ControlURL))
	if err != nil || control.Scheme != "https" || control.Host == "" || control.User != nil || control.Fragment != "" {
		return initializeConfig{}, errors.New("controlUrl must be an absolute HTTPS URL without userinfo or fragment")
	}
	if control.Path == "" {
		control.Path = "/"
	}
	if request.StateDir == "" || !filepath.IsAbs(request.StateDir) {
		return initializeConfig{}, errors.New("stateDir must be an absolute path")
	}
	key, err := base64.StdEncoding.DecodeString(request.StateKeyB64)
	if err != nil || len(key) != 32 {
		return initializeConfig{}, errors.New("stateKeyB64 must decode to exactly 32 bytes")
	}
	if len(request.AuthKey) > 512 {
		return initializeConfig{}, errors.New("authKey is too large")
	}
	return initializeConfig{
		Hostname:   hostname,
		ControlURL: control.String(),
		AuthKey:    request.AuthKey,
		StateDir:   request.StateDir,
		StateKey:   key,
	}, nil
}

func validRequestID(value string) bool {
	if len(value) < 8 || len(value) > 160 {
		return false
	}
	for _, r := range value {
		if !(r == '-' || r == '_' || r == '.' || r == ':' ||
			(r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9')) {
			return false
		}
	}
	return true
}

func redactError(err error) workerEvent {
	if err == nil {
		return workerEvent{Type: "error", Code: "unknown", Message: "network worker failed"}
	}
	// Never serialize the original error here: upstream errors can include URLs
	// or enrollment material. Detailed diagnostics stay in bounded local stderr.
	return workerEvent{Type: "error", Code: "network_worker_error", Message: "network worker operation failed"}
}
