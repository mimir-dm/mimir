---
id: print-and-pdf-generation-test
level: task
title: "Print and PDF Generation Test Coverage"
short_code: "MIMIR-T-0338"
created_at: 2026-01-14T01:50:48.847631+00:00
updated_at: 2026-01-14T01:50:48.847631+00:00
parent: MIMIR-I-0039
blocked_by: [MIMIR-I-0037]
archived: true

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0039
---

# Print and PDF Generation Test Coverage

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0039]]

## Objective

Add test coverage for the print and PDF generation functionality in `mimir-dm-print`, testing document rendering with Typst.

## Scope

**Target: `crates/mimir-dm-print/`**

The print crate handles:
- Document rendering using Typst
- Font handling and embedding
- Image processing and base64 encoding
- PDF generation for campaigns/modules

**Test Focus:**
1. Template rendering with sample data
2. Font loading and embedding
3. Image handling
4. PDF output validation (structure, not visual)

## Acceptance Criteria

## Acceptance Criteria

- [ ] Create test infrastructure for print/render operations
- [ ] Add tests for Typst template compilation
- [ ] Test with sample campaign/module data
- [ ] Verify PDF output is valid (parseable, correct structure)
- [ ] Test error handling (missing fonts, invalid templates)
- [ ] Test image embedding and base64 encoding
- [ ] Tests don't require visual inspection (structural validation)

## Implementation Notes

### Technical Approach

**Test Strategy:**
```rust
#[test]
fn test_render_campaign_pdf() {
    let campaign = create_test_campaign();
    let renderer = PdfRenderer::new();
    
    let pdf_bytes = renderer.render_campaign(&campaign)?;
    
    // Validate PDF structure
    assert!(!pdf_bytes.is_empty());
    assert!(pdf_bytes.starts_with(b"%PDF"));
    
    // Optionally parse with pdf crate to validate structure
    let doc = pdf::Document::from_bytes(&pdf_bytes)?;
    assert!(doc.pages().count() > 0);
}
```

**Test Scenarios:**
1. Empty campaign renders valid (minimal) PDF
2. Campaign with modules renders multi-page PDF
3. Missing template returns appropriate error
4. Font fallback works correctly
5. Large images are handled properly

### Dependencies
- May need `pdf` crate for validation
- Sample test data (campaign, module fixtures)
- Test fonts or system font access

### Risk Considerations
- Typst version compatibility
- Font availability on different systems
- PDF parsing libraries for validation
- Large PDF generation may be slow in tests

### Testing with angreal

Run print/PDF tests:
```bash
# Run core tests (includes mimir-dm-print)
angreal test unit --core

# Check coverage for rendering code
angreal test coverage --core --open
```

Coverage reports output to `target/coverage/tarpaulin-report.html`

## Status Updates **[REQUIRED]**

### 2026-01-14 - Blocked by Schema Hardening

This task is blocked by [[MIMIR-I-0037]] (Schema-First Catalog Model Hardening).

**Reason:** Print/PDF rendering includes monster stat blocks, spell cards, and class feature tables. Test fixtures for these will depend on catalog model shapes. Building fixtures against `serde_json::Value` fields risks significant rework post-schema hardening.

**Unblock condition:** Complete MIMIR-I-0037 or at minimum complete the monster/spell/class type migrations.