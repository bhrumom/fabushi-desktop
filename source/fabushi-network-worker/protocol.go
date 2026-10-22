package main

import (
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"net"
	"net/netip"
	"net/url"
	"path/filepath"
	"strings"
)

const protocolVersion = 1

const (
	maxStreamChunkBytes = 32 * 1024
	maxStreamSessions   = 32
	minStreamTimeoutMS  = 100
	maxStreamTimeoutMS  = 30_000
)

type workerRequest struct {
	Type            string `json:"type"`
	ProtocolVersion int    `json:"protocolVersion,omitempty"`
	Hostname        string `json:"hostname,omitempty"`
	ControlURL      string `json:"controlUrl,omitempty"`
	AuthKey         string `json:"authKey,omitempty"`
	StateDir        string `json:"stateDir,omitempty"`
	StateKeyB64     string `json:"stateKeyB64,omitempty"`
	RequestID       string `json:"requestId,omitempty"`
	SessionID       string `json:"sessionId,omitempty"`
	TargetPeer      string `json:"targetPeer,omitempty"`
	TargetPort      int    `json:"targetPort,omitempty"`
	DataB64         string `json:"dataB64,omitempty"`
	MaxBytes        int    `json:"maxBytes,omitempty"`
	TimeoutMS       int    `json:"timeoutMs,omitempty"`
}

type workerEvent struct {
	Type         string   `json:"type"`
	RequestID    string   `json:"requestId,omitempty"`
	SessionID    string   `json:"sessionId,omitempty"`
	BackendState string   `json:"backendState,omitempty"`
	TailscaleIPs []string `json:"tailscaleIps,omitempty"`
	DataB64      string   `json:"dataB64,omitempty"`
	Bytes        int      `json:"bytes,omitempty"`
	EOF          bool     `json:"eof,omitempty"`
	Code         string   `json:"code,omitempty"`
	Message      string   `json:"message,omitempty"`
}

type privateTransportRoute struct {
	NodeID     string `json:"nodeId"`
	CellID     string `json:"cellId"`
	Generation int64  `json:"generation"`
	ExpiresAt  int64  `json:"expiresAt"`
}

type privateTransportRequest struct {
	RequestID    string                `json:"requestId"`
	DeviceID     string                `json:"deviceId"`
	NetworkRoute privateTransportRoute `json:"networkRoute"`
	TargetPort   int                   `json:"targetPort"`
	TimeoutMS    int                   `json:"timeoutMs"`
	Payload      json.RawMessage       `json:"payload"`
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

func validOpaqueID(value string) bool {
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

func validRequestID(value string) bool { return validOpaqueID(value) }
func validSessionID(value string) bool { return validOpaqueID(value) }

func tailnetPeer(value string) (string, bool) {
	addr, err := netip.ParseAddr(strings.TrimSpace(value))
	if err != nil {
		return "", false
	}
	addr = addr.Unmap()
	if addr.Is4() {
		prefix := netip.MustParsePrefix("100.64.0.0/10")
		if !prefix.Contains(addr) {
			return "", false
		}
	} else {
		prefix := netip.MustParsePrefix("fd7a:115c:a1e0::/48")
		if !prefix.Contains(addr) {
			return "", false
		}
	}
	return addr.String(), true
}

func streamAddress(peer string, port int) (string, bool) {
	normalized, ok := tailnetPeer(peer)
	if !ok || port < 1 || port > 65535 {
		return "", false
	}
	return net.JoinHostPort(normalized, fmt.Sprintf("%d", port)), true
}

func listenAddress(port int) (string, bool) {
	if port < 1 || port > 65535 {
		return "", false
	}
	return fmt.Sprintf(":%d", port), true
}

func remotePeerMatches(address net.Addr, expected string) bool {
	if address == nil {
		return false
	}
	host, _, err := net.SplitHostPort(address.String())
	if err != nil {
		return false
	}
	actual, ok := tailnetPeer(host)
	if !ok {
		return false
	}
	normalizedExpected, ok := tailnetPeer(expected)
	return ok && actual == normalizedExpected
}

func streamTimeoutMS(value int) (int, bool) {
	if value == 0 {
		return 5_000, true
	}
	if value < minStreamTimeoutMS || value > maxStreamTimeoutMS {
		return 0, false
	}
	return value, true
}

func decodeStreamChunk(value string) ([]byte, bool) {
	data, err := base64.StdEncoding.DecodeString(value)
	if err != nil || len(data) > maxStreamChunkBytes {
		return nil, false
	}
	return data, true
}

func boundedReadSize(value int) (int, bool) {
	if value <= 0 {
		return maxStreamChunkBytes, true
	}
	if value > maxStreamChunkBytes {
		return 0, false
	}
	return value, true
}

func redactError(err error) workerEvent {
	if err == nil {
		return workerEvent{Type: "error", Code: "unknown", Message: "network worker failed"}
	}
	return workerEvent{Type: "error", Code: "network_worker_error", Message: "network worker operation failed"}
}
