package main

import (
	"bytes"
	"crypto/rand"
	"os"
	"testing"

	"tailscale.com/ipn"
)

func testStateKey(t *testing.T) []byte {
	t.Helper()
	key := make([]byte, 32)
	if _, err := rand.Read(key); err != nil {
		t.Fatal(err)
	}
	return key
}

func TestEncryptedStateStoreRoundTripDoesNotPersistPlaintext(t *testing.T) {
	dir := t.TempDir()
	store, err := newEncryptedStateStore(dir, testStateKey(t))
	if err != nil {
		t.Fatal(err)
	}
	id := ipn.StateKey("_machinekey")
	secret := []byte("machine-private-material-must-never-be-plaintext")
	if err := store.WriteState(id, secret); err != nil {
		t.Fatal(err)
	}
	raw, err := os.ReadFile(store.pathFor(id))
	if err != nil {
		t.Fatal(err)
	}
	if bytes.Contains(raw, secret) {
		t.Fatal("encrypted state file contains plaintext secret")
	}
	if bytes.Contains(raw, []byte(id)) {
		t.Fatal("encrypted state file contains plaintext state key")
	}
	got, err := store.ReadState(id)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(got, secret) {
		t.Fatalf("round trip mismatch: got %q", got)
	}
}

func TestEncryptedStateStoreRejectsTampering(t *testing.T) {
	dir := t.TempDir()
	store, err := newEncryptedStateStore(dir, testStateKey(t))
	if err != nil {
		t.Fatal(err)
	}
	id := ipn.StateKey("_profiles")
	if err := store.WriteState(id, []byte("profile-secret")); err != nil {
		t.Fatal(err)
	}
	path := store.pathFor(id)
	raw, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}
	raw[len(raw)-1] ^= 0xff
	if err := os.WriteFile(path, raw, 0o600); err != nil {
		t.Fatal(err)
	}
	if _, err := store.ReadState(id); err == nil {
		t.Fatal("tampered state unexpectedly decrypted")
	}
}

func TestEncryptedStateStoreNilWriteDeletes(t *testing.T) {
	dir := t.TempDir()
	store, err := newEncryptedStateStore(dir, testStateKey(t))
	if err != nil {
		t.Fatal(err)
	}
	id := ipn.StateKey("_current-profile")
	if err := store.WriteState(id, []byte("value")); err != nil {
		t.Fatal(err)
	}
	if err := store.WriteState(id, nil); err != nil {
		t.Fatal(err)
	}
	if _, err := store.ReadState(id); err != ipn.ErrStateNotExist {
		t.Fatalf("expected ErrStateNotExist, got %v", err)
	}
}

func TestEncryptedStateStoreUsesDifferentNonceForSamePlaintext(t *testing.T) {
	dir := t.TempDir()
	store, err := newEncryptedStateStore(dir, testStateKey(t))
	if err != nil {
		t.Fatal(err)
	}
	id := ipn.StateKey("same-state")
	value := []byte("same plaintext")
	if err := store.WriteState(id, value); err != nil {
		t.Fatal(err)
	}
	first, err := os.ReadFile(store.pathFor(id))
	if err != nil {
		t.Fatal(err)
	}
	if err := store.WriteState(id, value); err != nil {
		t.Fatal(err)
	}
	second, err := os.ReadFile(store.pathFor(id))
	if err != nil {
		t.Fatal(err)
	}
	if bytes.Equal(first, second) {
		t.Fatal("two writes reused the same encrypted envelope")
	}
}
