package main

import (
	"context"
	"errors"
	"io"
	"net"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

const acceptPeerTestGuard = 10 * time.Second

type controlledPeerListener struct {
	acceptFn    func() (net.Conn, error)
	acceptCalls atomic.Int32
	closeCalls  atomic.Int32
	closeOnce   sync.Once
	closed      chan struct{}
}

func newControlledPeerListener(t *testing.T) *controlledPeerListener {
	t.Helper()
	listener := &controlledPeerListener{
		acceptFn: func() (net.Conn, error) { return nil, net.ErrClosed },
		closed:   make(chan struct{}),
	}
	t.Cleanup(func() {
		select {
		case <-listener.closed:
		default:
			_ = listener.Close()
		}
	})
	return listener
}

func (l *controlledPeerListener) Accept() (net.Conn, error) {
	l.acceptCalls.Add(1)
	return l.acceptFn()
}

func (l *controlledPeerListener) Close() error {
	l.closeCalls.Add(1)
	l.closeOnce.Do(func() { close(l.closed) })
	return nil
}

func (l *controlledPeerListener) Addr() net.Addr {
	return &net.TCPAddr{IP: net.IPv4(100, 64, 0, 1), Port: 8443}
}

type peerAddressConn struct {
	net.Conn
	peer net.Addr
}

func (c *peerAddressConn) RemoteAddr() net.Addr { return c.peer }

func newAcceptPeerPipe(t *testing.T, peer string) (net.Conn, net.Conn) {
	t.Helper()
	accepted, remote := net.Pipe()
	t.Cleanup(func() {
		_ = accepted.Close()
		_ = remote.Close()
	})
	for _, conn := range []net.Conn{accepted, remote} {
		if err := conn.SetDeadline(time.Now().Add(acceptPeerTestGuard)); err != nil {
			t.Fatal(err)
		}
	}
	return &peerAddressConn{
		Conn: accepted,
		peer: &net.TCPAddr{IP: net.ParseIP(peer), Port: 51234},
	}, remote
}

type peerAcceptResult struct {
	conn net.Conn
	err  error
}

func startAcceptPeer(ctx context.Context, listener net.Listener) <-chan peerAcceptResult {
	result := make(chan peerAcceptResult, 1)
	go func() {
		conn, err := acceptPeer(ctx, listener, "100.64.0.30")
		result <- peerAcceptResult{conn: conn, err: err}
	}()
	return result
}

func awaitPeerAccept(t *testing.T, result <-chan peerAcceptResult) peerAcceptResult {
	t.Helper()
	timer := time.NewTimer(acceptPeerTestGuard)
	defer timer.Stop()
	select {
	case accepted := <-result:
		return accepted
	case <-timer.C:
		t.Fatal("acceptPeer did not settle")
		return peerAcceptResult{}
	}
}

func awaitPeerSignal(t *testing.T, signal <-chan struct{}) {
	t.Helper()
	timer := time.NewTimer(acceptPeerTestGuard)
	defer timer.Stop()
	select {
	case <-signal:
	case <-timer.C:
		t.Fatal("listener did not reach the controlled step")
	}
}

func assertPeerListenerClosed(t *testing.T, listener *controlledPeerListener, accepts int32) {
	t.Helper()
	if got := listener.acceptCalls.Load(); got != accepts {
		t.Fatalf("Accept calls = %d, want %d", got, accepts)
	}
	if got := listener.closeCalls.Load(); got != 1 {
		t.Fatalf("Close calls = %d, want 1", got)
	}
	select {
	case <-listener.closed:
	default:
		t.Fatal("listener remained open")
	}
}

func assertPeerEOF(t *testing.T, peer net.Conn) {
	t.Helper()
	var data [1]byte
	n, err := peer.Read(data[:])
	if n != 0 || !errors.Is(err, io.EOF) {
		t.Fatalf("peer read = (%d, %v), want connection EOF", n, err)
	}
}

func TestAcceptPeerExpiredBeforeAccept(t *testing.T) {
	listener := newControlledPeerListener(t)
	ctx, cancel := context.WithDeadline(context.Background(), time.Unix(0, 0))
	t.Cleanup(cancel)
	result := awaitPeerAccept(t, startAcceptPeer(ctx, listener))
	if result.conn != nil || !errors.Is(result.err, context.DeadlineExceeded) {
		t.Fatalf("expired attempt = (%v, %v)", result.conn, result.err)
	}
	assertPeerListenerClosed(t, listener, 0)
}

func TestAcceptPeerCancellationUnblocksAccept(t *testing.T) {
	listener := newControlledPeerListener(t)
	acceptStarted := make(chan struct{})
	listener.acceptFn = func() (net.Conn, error) {
		close(acceptStarted)
		<-listener.closed
		return nil, net.ErrClosed
	}
	ctx, cancel := context.WithCancel(context.Background())
	t.Cleanup(cancel)
	resultCh := startAcceptPeer(ctx, listener)
	awaitPeerSignal(t, acceptStarted)
	cancel()
	result := awaitPeerAccept(t, resultCh)
	if result.conn != nil || !errors.Is(result.err, context.Canceled) {
		t.Fatalf("canceled blocked attempt = (%v, %v)", result.conn, result.err)
	}
	assertPeerListenerClosed(t, listener, 1)
}

func TestAcceptPeerClosesLateConnectionAfterCancellation(t *testing.T) {
	listener := newControlledPeerListener(t)
	conn, peer := newAcceptPeerPipe(t, "100.64.0.30")
	acceptStarted := make(chan struct{})
	releaseAccept := make(chan struct{}, 1)
	t.Cleanup(func() { close(releaseAccept) })
	listener.acceptFn = func() (net.Conn, error) {
		close(acceptStarted)
		<-releaseAccept
		return conn, nil
	}
	ctx, cancel := context.WithCancel(context.Background())
	t.Cleanup(cancel)
	resultCh := startAcceptPeer(ctx, listener)
	awaitPeerSignal(t, acceptStarted)
	cancel()
	awaitPeerSignal(t, listener.closed)
	// Accept releases the already obtained connection only after cancellation
	// has closed the listener; listener Close cannot reclaim this connection.
	releaseAccept <- struct{}{}
	result := awaitPeerAccept(t, resultCh)
	if result.conn != nil || !errors.Is(result.err, context.Canceled) {
		t.Fatalf("late connection attempt = (%v, %v)", result.conn, result.err)
	}
	assertPeerEOF(t, peer)
	assertPeerListenerClosed(t, listener, 1)
}

func TestAcceptPeerRejectsWrongPeerAndTransfersMatchingConnection(t *testing.T) {
	listener := newControlledPeerListener(t)
	wrong, wrongPeer := newAcceptPeerPipe(t, "100.64.0.31")
	matching, matchingPeer := newAcceptPeerPipe(t, "100.64.0.30")
	nextAccept := make(chan struct{})
	releaseMatching := make(chan struct{}, 1)
	t.Cleanup(func() { close(releaseMatching) })
	listener.acceptFn = func() (net.Conn, error) {
		switch listener.acceptCalls.Load() {
		case 1:
			return wrong, nil
		case 2:
			close(nextAccept)
			<-releaseMatching
			return matching, nil
		default:
			return nil, errors.New("unexpected additional Accept")
		}
	}
	ctx, cancel := context.WithCancel(context.Background())
	t.Cleanup(cancel)
	resultCh := startAcceptPeer(ctx, listener)
	awaitPeerSignal(t, nextAccept)
	assertPeerEOF(t, wrongPeer)
	releaseMatching <- struct{}{}
	result := awaitPeerAccept(t, resultCh)
	if result.conn != matching || result.err != nil {
		t.Fatalf("matching attempt = (%v, %v)", result.conn, result.err)
	}
	assertPeerListenerClosed(t, listener, 2)
	cancel()
	peerReply := make(chan error, 1)
	go func() {
		var request [4]byte
		_, err := io.ReadFull(matchingPeer, request[:])
		if err == nil && string(request[:]) != "ping" {
			err = errors.New("matching peer received unexpected request")
		}
		if err == nil {
			_, err = matchingPeer.Write([]byte("pong"))
		}
		peerReply <- err
	}()
	if _, err := result.conn.Write([]byte("ping")); err != nil {
		t.Fatalf("caller-owned connection write failed: %v", err)
	}
	var response [4]byte
	if _, err := io.ReadFull(result.conn, response[:]); err != nil {
		t.Fatalf("caller-owned connection read failed: %v", err)
	}
	if string(response[:]) != "pong" {
		t.Fatalf("caller received %q, want pong", response[:])
	}
	select {
	case err := <-peerReply:
		if err != nil {
			t.Fatal(err)
		}
	case <-time.After(acceptPeerTestGuard):
		t.Fatal("matching peer did not finish its I/O")
	}
	if err := result.conn.Close(); err != nil {
		t.Fatal(err)
	}
	assertPeerEOF(t, matchingPeer)
}

func TestAcceptPeerStopsOnAcceptError(t *testing.T) {
	for _, testCase := range []struct {
		name     string
		withConn bool
	}{
		{name: "without_connection"},
		{name: "with_connection", withConn: true},
	} {
		t.Run(testCase.name, func(t *testing.T) {
			listener := newControlledPeerListener(t)
			var conn, peer net.Conn
			if testCase.withConn {
				conn, peer = newAcceptPeerPipe(t, "100.64.0.30")
			}
			acceptErr := errors.New("terminal Accept error")
			listener.acceptFn = func() (net.Conn, error) { return conn, acceptErr }
			ctx, cancel := context.WithCancel(context.Background())
			t.Cleanup(cancel)
			result := awaitPeerAccept(t, startAcceptPeer(ctx, listener))
			if result.conn != nil || !errors.Is(result.err, acceptErr) {
				t.Fatalf("failed attempt = (%v, %v)", result.conn, result.err)
			}
			if ctx.Err() != nil {
				t.Fatal("Accept error waited for cancellation instead of terminating")
			}
			if peer != nil {
				assertPeerEOF(t, peer)
			}
			assertPeerListenerClosed(t, listener, 1)
		})
	}
}
