# Fabushi Desktop product definition

Status: current repository scope  
Last reviewed: 2026-09-22

Fabushi Desktop is the independent Electron desktop application for Fabushi, with a local Mahayana Agent/runtime stack and desktop integrations.

## Product scope

This repository owns the desktop product boundary, including:

- Electron desktop shell and renderer;
- Mahayana runtime/Host dependencies required by the desktop;
- desktop Agent conversations and runtime projections;
- desktop-local capability surfaces such as computer control and connector/MCP integration where governed by active Specs;
- desktop packaging, update, release, and packaged acceptance.

## Engineering principles

- User-visible behavior is driven by durable Specs and observable acceptance criteria.
- Runtime state has explicit ownership; UI is a projection, not an inferred source of truth.
- Privileged capabilities use explicit security/capability boundaries.
- Architecture is language-agnostic: each layer uses the most appropriate language while preserving ownership boundaries.
- Release claims require exact-source and artifact evidence.

## Out of scope for this document

Mobile products, unrelated backend/service ownership, and future product commitments are not invented here. Their owning repositories/project sources of truth remain authoritative.

See `docs/README.md` for the documentation model and `ROADMAP.md` for how active work is tracked.
