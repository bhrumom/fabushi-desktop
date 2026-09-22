fn main() {
    // Electron will become the sole process owner during cutover. Until that
    // launcher is wired, this binary remains inert so development cannot
    // accidentally start a second coordinator beside the legacy path.
    eprintln!("mahayana-node-agent-coordinator: launcher not bound");
}
