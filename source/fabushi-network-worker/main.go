package main

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"time"

	"tailscale.com/envknob"
	"tailscale.com/tsnet"
)

const (
	maxControlFrameBytes = 64 * 1024
	statusTimeout        = 5 * time.Second
)

func emit(writer io.Writer, event workerEvent) error {
	encoded, err := json.Marshal(event)
	if err != nil {
		return err
	}
	_, err = fmt.Fprintf(writer, "%s\n", encoded)
	return err
}

func readRequest(scanner *bufio.Scanner) (workerRequest, error) {
	if !scanner.Scan() {
		if err := scanner.Err(); err != nil {
			return workerRequest{}, err
		}
		return workerRequest{}, io.EOF
	}
	var request workerRequest
	if err := json.Unmarshal(scanner.Bytes(), &request); err != nil {
		return workerRequest{}, errorsForWire("invalid_json")
	}
	return request, nil
}

type wireError string

func (e wireError) Error() string { return string(e) }

func errorsForWire(code string) error { return wireError(code) }

func run(stdin io.Reader, stdout, stderr io.Writer) int {
	envknob.SetNoLogsNoSupport()

	scanner := bufio.NewScanner(stdin)
	scanner.Buffer(make([]byte, 4096), maxControlFrameBytes)

	first, err := readRequest(scanner)
	if err != nil {
		_ = emit(stdout, workerEvent{Type: "error", Code: "initialize_required", Message: "network worker initialization failed"})
		return 2
	}
	config, err := validateInitialize(first)
	if err != nil {
		_ = emit(stdout, workerEvent{Type: "error", Code: "invalid_initialize", Message: "network worker initialization failed"})
		return 2
	}

	store, err := newEncryptedStateStore(config.StateDir, config.StateKey)
	for i := range config.StateKey {
		config.StateKey[i] = 0
	}
	if err != nil {
		_ = emit(stdout, redactError(err))
		return 3
	}

	server := &tsnet.Server{
		Dir:          config.StateDir,
		Store:        store,
		Hostname:     config.Hostname,
		AuthKey:      config.AuthKey,
		ControlURL:   config.ControlURL,
		RunWebClient: false,
		UserLogf: func(string, ...any) {
			_, _ = fmt.Fprintln(stderr, "[fabushi-network] tsnet user status changed")
		},
	}
	config.AuthKey = ""

	if err := server.Start(); err != nil {
		_ = emit(stdout, redactError(err))
		return 4
	}
	defer server.Close()

	if err := emit(stdout, workerEvent{Type: "ready"}); err != nil {
		return 5
	}

	for {
		request, err := readRequest(scanner)
		if err == io.EOF {
			// stdin is the inherited parent-lifetime pipe. EOF means the Host
			// generation disappeared; never leave a network node behind.
			return 0
		}
		if err != nil {
			_ = emit(stdout, workerEvent{Type: "error", Code: "invalid_request", Message: "network worker request is invalid"})
			continue
		}
		switch request.Type {
		case "status":
			if !validRequestID(request.RequestID) {
				_ = emit(stdout, workerEvent{Type: "error", RequestID: request.RequestID, Code: "invalid_request_id", Message: "requestId is invalid"})
				continue
			}
			ctx, cancel := context.WithTimeout(context.Background(), statusTimeout)
			localClient, clientErr := server.LocalClient()
			if clientErr != nil {
				cancel()
				_ = emit(stdout, workerEvent{Type: "error", RequestID: request.RequestID, Code: "status_unavailable", Message: "network status is unavailable"})
				continue
			}
			status, statusErr := localClient.StatusWithoutPeers(ctx)
			cancel()
			if statusErr != nil {
				_ = emit(stdout, workerEvent{Type: "error", RequestID: request.RequestID, Code: "status_unavailable", Message: "network status is unavailable"})
				continue
			}
			ips := make([]string, 0, len(status.TailscaleIPs))
			for _, ip := range status.TailscaleIPs {
				ips = append(ips, ip.String())
			}
			_ = emit(stdout, workerEvent{
				Type:         "status",
				RequestID:    request.RequestID,
				BackendState: status.BackendState,
				TailscaleIPs: ips,
			})
		case "shutdown":
			if !validRequestID(request.RequestID) {
				_ = emit(stdout, workerEvent{Type: "error", RequestID: request.RequestID, Code: "invalid_request_id", Message: "requestId is invalid"})
				continue
			}
			_ = emit(stdout, workerEvent{Type: "shutdown", RequestID: request.RequestID})
			return 0
		default:
			_ = emit(stdout, workerEvent{Type: "error", RequestID: request.RequestID, Code: "unsupported_request", Message: "network worker request is not supported"})
		}
	}
}

func main() {
	os.Exit(run(os.Stdin, os.Stdout, os.Stderr))
}
