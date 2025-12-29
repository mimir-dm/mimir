---
id: create-install-script
level: task
title: "Create Install Script"
short_code: "MIMIR-T-0132"
created_at: 2025-12-06T16:17:57.333169+00:00
updated_at: 2025-12-17T17:05:06.445116+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#feature"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# One-line Install Script for Mimir

## Objective

Enable installation of Mimir via a single curl command, following the pattern used by other Colliery tools:

```bash
curl -sSL https://raw.githubusercontent.com/colliery-io/mimir/main/scripts/install.sh | sh
```

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement  

### Priority
- [x] P2 - Medium (nice to have)

### Business Justification
- **User Value**: Users can install Mimir without needing Rust/Node toolchain or building from source
- **Business Value**: Lower barrier to adoption, easier onboarding for new users
- **Effort Estimate**: S-M (script itself is small, but requires release pipeline)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `scripts/install.sh` exists in repository
- [ ] Script detects OS (macOS, Linux) and architecture (arm64, x86_64)
- [ ] Script downloads appropriate pre-built binary from GitHub Releases
- [ ] Script verifies checksum before installation
- [ ] Script installs to user-accessible location (`~/Applications` for macOS .app)
- [ ] Script provides clear success/failure output
- [ ] Works on macOS Apple Silicon
- [ ] Works on macOS Intel

## Implementation Notes

### Reference Implementation

See the crt install script for pattern:
https://raw.githubusercontent.com/colliery-io/crt/main/scripts/install.sh

### Technical Approach

1. Create `scripts/install.sh` with:
   - Platform detection (uname -s, uname -m)
   - GitHub Release artifact download via curl
   - SHA256 checksum verification
   - Extraction and installation to ~/Applications
   - Gatekeeper/quarantine attribute handling for macOS

2. Tauri produces a `.app` bundle on macOS - script needs to handle this (not a single binary)

3. Consider adding `--version` flag to install specific version

### Dependencies

- Requires GitHub Actions workflow to build and publish release artifacts
- Requires tagged releases with consistent artifact naming convention:
  - `mimir-darwin-arm64.tar.gz`
  - `mimir-darwin-x86_64.tar.gz`
  - `checksums.txt`

### Risk Considerations

- macOS Gatekeeper may block unsigned apps - may need codesigning
- Large download size for Tauri app bundle
- Script must handle network failures gracefully

## Status Updates

*To be added during implementation*