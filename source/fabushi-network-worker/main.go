package main

import (
	"bufio"
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"net"
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

func emitRequestError(stdout io.Writer, request workerRequest, code, message string) {
	_ = emit(stdout, workerEvent{
		Type:      "error",
		RequestID: request.RequestID,
		SessionID: request.SessionID,
		Code:      code,
		Message:   message,
	})
}

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

	streams := map[string]net.Conn{}
	defer func() {
		for _, stream := range streams {
			_ = stream.Close()
		}
	}()

	if err := emit(stdout, workerEvent{Type: "ready"}); err != nil {
		return 5
	}

	for {
		request, err := readRequest(scanner)
		if err == io.EOF {
			return 0
		}
		if err != nil {
			_ = emit(stdout, workerEvent{Type: "error", Code: "invalid_request", Message: "network worker request is invalid"})
			continue
		}
		switch request.Type {
		case "status":
			if !validRequestID(request.RequestID) {
				emitRequestError(stdout, request, "invalid_request_id", "requestId is invalid")
				continue
			}
			ctx, cancel := context.WithTimeout(context.Background(), statusTimeout)
			localClient, clientErr := server.LocalClient()
			if clientErr != nil {
				cancel()
				emitRequestError(stdout, request, "status_unavailable", "network status is unavailable")
				continue
			}
			status, statusErr := localClient.StatusWithoutPeers(ctx)
			cancel()
			if statusErr != nil {
				emitRequestError(stdout, request, "status_unavailable", "network status is unavailable")
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
		case "streamDial":
			if !validRequestID(request.RequestID) || !validSessionID(request.SessionID) {
				emitRequestError(stdout, request, "invalid_stream_identity", "stream request identity is invalid")
				continue
			}
			if _, exists := streams[request.SessionID]; exists {
				emitRequestError(stdout, request, "stream_session_exists", "stream session already exists")
				continue
			}
			if len(streams) >= maxStreamSessions {
				emitRequestError(stdout, request, "stream_session_limit", "stream session limit reached")
				continue
			}
			address, ok := streamAddress(request.TargetPeer, request.TargetPort)
			timeoutMS, timeoutOK := streamTimeoutMS(request.TimeoutMS)
			if !ok || !timeoutOK {
				emitRequestError(stdout, request, "invalid_stream_target", "stream target or timeout is invalid")
				continue
			}
			ctx, cancel := context.WithTimeout(context.Background(), time.Duration(timeoutMS)*time.Millisecond)
			conn, dialErr := server.Dial(ctx, "tcp", address)
			cancel()
			if dialErr != nil {
				emitRequestError(stdout, request, "stream_dial_failed", "stream dial failed")
				continue
			}
			streams[request.SessionID] = conn
			_ = emit(stdout, workerEvent{Type: "streamDialed", RequestID: request.RequestID, SessionID: request.SessionID})
		case "streamWrite":
			if !validRequestID(request.RequestID) || !validSessionID(request.SessionID) {
				emitRequestError(stdout, request, "invalid_stream_identity", "stream request identity is invalid")
				continue
			}
			conn := streams[request.SessionID]
			if conn == nil {
				emitRequestError(stdout, request, "stream_session_not_found", "stream session was not found")
				continue
			}
			data, ok := decodeStreamChunk(request.DataB64)
			timeoutMS, timeoutOK := streamTimeoutMS(request.TimeoutMS)
			if !ok || !timeoutOK {
				emitRequestError(stdout, request, "invalid_stream_write", "stream write payload or timeout is invalid")
				continue
			}
			_ = conn.SetWriteDeadline(time.Now().Add(time.Duration(timeoutMS) * time.Millisecond))
			written, writeErr := conn.Write(data)
			if writeErr != nil {
				_ = conn.Close()
				delete(streams, request.SessionID)
				emitRequestError(stdout, request, "stream_write_failed", "stream write failed")
				continue
			}
			_ = emit(stdout, workerEvent{Type: "streamWritten", RequestID: request.RequestID, SessionID: request.SessionID, Bytes: written})
		case "streamRead":
			if !validRequestID(request.RequestID) || !validSessionID(request.SessionID) {
				emitRequestError(stdout, request, "invalid_stream_identity", "stream request identity is invalid")
				continue
			}
			conn := streams[request.SessionID]
			if conn == nil {
				emitRequestError(stdout, request, "stream_session_not_found", "stream session was not found")
				continue
			}
			readSize, sizeOK := boundedReadSize(request.MaxBytes)
			timeoutMS, timeoutOK := streamTimeoutMS(request.TimeoutMS)
			if !sizeOK || !timeoutOK {
				emitRequestError(stdout, request, "invalid_stream_read", "stream read bound or timeout is invalid")
				continue
			}
			buffer := make([]byte, readSize)
			_ = conn.SetReadDeadline(time.Now().Add(time.Duration(timeoutMS) * time.Millisecond))
			count, readErr := conn.Read(buffer)
			if readErr != nil && readErr != io.EOF {
				if timeout, ok := readErr.(net.Error); ok && timeout.Timeout() {
					emitRequestError(stdout, request, "stream_read_timeout", "stream read timed out")
					continue
				}
				_ = conn.Close()
				delete(streams, request.SessionID)
				emitRequestError(stdout, request, "stream_read_failed", "stream read failed")
				continue
			}
			eof := readErr == io.EOF
			if eof {
				_ = conn.Close()
				delete(streams, request.SessionID)
			}
			_ = emit(stdout, workerEvent{
				Type:      "streamData",
				RequestID: request.RequestID,
				SessionID: request.SessionID,
				DataB64:   base64.StdEncoding.EncodeToString(buffer[:count]),
				Bytes:     count,
				EOF:       eof,
			})
		case "streamClose":
			if !validRequestID(request.RequestID) || !validSessionID(request.SessionID) {
				emitRequestError(stdout, request, "invalid_stream_identity", "stream request identity is invalid")
				continue
			}
			conn := streams[request.SessionID]
			if conn == nil {
				emitRequestError(stdout, request, "stream_session_not_found", "stream session was not found")
				continue
			}
			_ = conn.Close()
			delete(streams, request.SessionID)
			_ = emit(stdout, workerEvent{Type: "streamClosed", RequestID: request.RequestID, SessionID: request.SessionID})
		case "shutdown":
			if !validRequestID(request.RequestID) {
				emitRequestError(stdout, request, "invalid_request_id", "requestId is invalid")
				continue
			}
			_ = emit(stdout, workerEvent{Type: "shutdown", RequestID: request.RequestID})
			return 0
		default:
			emitRequestError(stdout, request, "unsupported_request", "network worker request is not supported")
		}
	}
}

func main() {
	os.Exit(run(os.Stdin, os.Stdout, os.Stderr))
}
