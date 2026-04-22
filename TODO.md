# DocFixer Rate Limit & Reliability Fixes

## Current Status

```
Stage 1 ✓ Auto-fixed 248 paragraphs
Stage 2 ✓ 287 flagged / 828 clean (74% token savings!)
Stage 3 ✗ 429 Rate Limits + Connection Errors → Early Termination
```

## Fix Plan (Rate Limit Safe) - STATUS: COMPLETE ✅

### Phase 1: Robust API Client

- [x] Fix endpoint double-path detection
- [x] concurrency: 5 → 1 (serial processing)
- [x] Exponential backoff for 429 (2s,4s,8s,16s,32s)
- [x] Increase max_consecutive_errors: 5 → 20
- [x] Partial success: Export whatever batches succeed

### Phase 2: Testing

- [ ] Test with real document (expect 8/10+ batches success)
- [ ] Verify auto-fixes + partial AI fixes export correctly

### Phase 3: Polish

- [ ] UI: Show "X/10 batches failed, continuing..."
- [ ] Config: User-set concurrency (1-3)

**Expected Results:** 80-90% batch success rate even under rate limits. Always exports auto-fixes + successful AI batches.

**Test command:** `npm run tauri dev` → try SMART MODE again
