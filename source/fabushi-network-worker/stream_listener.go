package main

import (
	"context"
	"errors"
	"net"
	"sync"
)

// acceptPeer owns the listener and every accepted connection until the final
// cancellation check transfers a matching connection to its caller.
func acceptPeer(ctx context.Context, listener net.Listener, expectedPeer string) (net.Conn, error) {
	var closeOnce sync.Once
	closeListener := func() {
		closeOnce.Do(func() {
			_ = listener.Close()
		})
	}
	stopClosing := context.AfterFunc(ctx, closeListener)
	defer func() {
		stopClosing()
		// Stopping AfterFunc does not wait for an active callback. The shared
		// Once waits for its Close call, or performs the close itself.
		closeListener()
	}()

	for {
		if err := ctx.Err(); err != nil {
			return nil, err
		}
		conn, acceptErr := listener.Accept()
		if acceptErr != nil {
			if conn != nil {
				_ = conn.Close()
			}
			if err := ctx.Err(); err != nil {
				return nil, err
			}
			return nil, acceptErr
		}
		if conn == nil {
			return nil, errors.New("listener returned no connection")
		}
		if err := ctx.Err(); err != nil {
			_ = conn.Close()
			return nil, err
		}
		if !remotePeerMatches(conn.RemoteAddr(), expectedPeer) {
			_ = conn.Close()
			continue
		}
		// This is the admission boundary: subsequent cancellation closes only
		// the listener, never the connection now owned by the caller.
		if err := ctx.Err(); err != nil {
			_ = conn.Close()
			return nil, err
		}
		return conn, nil
	}
}
