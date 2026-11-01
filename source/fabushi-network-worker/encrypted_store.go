package main

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/rand"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"sync"

	"tailscale.com/ipn"
)

const encryptedStateMagic = "FNS1"

type encryptedStateStore struct {
	dir  string
	aead cipher.AEAD
	mu   sync.Mutex
}

func newEncryptedStateStore(dir string, key []byte) (*encryptedStateStore, error) {
	if len(key) != 32 {
		return nil, fmt.Errorf("state encryption key must be 32 bytes")
	}
	if dir == "" {
		return nil, fmt.Errorf("state directory is required")
	}
	block, err := aes.NewCipher(key)
	if err != nil {
		return nil, fmt.Errorf("create state cipher: %w", err)
	}
	aead, err := cipher.NewGCM(block)
	if err != nil {
		return nil, fmt.Errorf("create state AEAD: %w", err)
	}
	if err := os.MkdirAll(dir, 0o700); err != nil {
		return nil, fmt.Errorf("create state directory: %w", err)
	}
	_ = os.Chmod(dir, 0o700)
	return &encryptedStateStore{dir: dir, aead: aead}, nil
}

func (s *encryptedStateStore) ReadState(id ipn.StateKey) ([]byte, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	payload, err := os.ReadFile(s.pathFor(id))
	if errors.Is(err, os.ErrNotExist) {
		return nil, ipn.ErrStateNotExist
	}
	if err != nil {
		return nil, fmt.Errorf("read encrypted state: %w", err)
	}
	headerLen := len(encryptedStateMagic) + s.aead.NonceSize()
	if len(payload) < headerLen || string(payload[:len(encryptedStateMagic)]) != encryptedStateMagic {
		return nil, fmt.Errorf("encrypted state has invalid envelope")
	}
	nonce := payload[len(encryptedStateMagic):headerLen]
	ciphertext := payload[headerLen:]
	plaintext, err := s.aead.Open(nil, nonce, ciphertext, []byte(id))
	if err != nil {
		return nil, fmt.Errorf("decrypt state: %w", err)
	}
	return plaintext, nil
}

func (s *encryptedStateStore) WriteState(id ipn.StateKey, value []byte) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	path := s.pathFor(id)
	if value == nil {
		if err := os.Remove(path); err != nil && !errors.Is(err, os.ErrNotExist) {
			return fmt.Errorf("delete encrypted state: %w", err)
		}
		return nil
	}

	nonce := make([]byte, s.aead.NonceSize())
	if _, err := io.ReadFull(rand.Reader, nonce); err != nil {
		return fmt.Errorf("generate state nonce: %w", err)
	}
	ciphertext := s.aead.Seal(nil, nonce, value, []byte(id))
	envelope := make([]byte, 0, len(encryptedStateMagic)+len(nonce)+len(ciphertext))
	envelope = append(envelope, encryptedStateMagic...)
	envelope = append(envelope, nonce...)
	envelope = append(envelope, ciphertext...)

	tmp, err := os.CreateTemp(s.dir, ".state-*")
	if err != nil {
		return fmt.Errorf("create state temp file: %w", err)
	}
	tmpName := tmp.Name()
	committed := false
	defer func() {
		_ = tmp.Close()
		if !committed {
			_ = os.Remove(tmpName)
		}
	}()
	if err := tmp.Chmod(0o600); err != nil {
		return fmt.Errorf("restrict state temp file: %w", err)
	}
	if _, err := tmp.Write(envelope); err != nil {
		return fmt.Errorf("write encrypted state: %w", err)
	}
	if err := tmp.Sync(); err != nil {
		return fmt.Errorf("sync encrypted state: %w", err)
	}
	if err := tmp.Close(); err != nil {
		return fmt.Errorf("close encrypted state: %w", err)
	}
	if err := replaceFile(tmpName, path); err != nil {
		return fmt.Errorf("commit encrypted state: %w", err)
	}
	_ = os.Chmod(path, 0o600)
	committed = true
	return nil
}

func (s *encryptedStateStore) pathFor(id ipn.StateKey) string {
	sum := sha256.Sum256([]byte(id))
	return filepath.Join(s.dir, hex.EncodeToString(sum[:])+".bin")
}

func replaceFile(source, destination string) error {
	// Both paths live in the same private state directory. os.Rename maps to
	// the platform's same-filesystem rename/replace primitive and prevents a
	// partially-written destination from becoming visible.
	return os.Rename(source, destination)
}
