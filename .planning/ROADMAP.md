# MaScribe Development Roadmap

**Project:** MaScribe（马上听写）  
**Current Version:** v0.4.0  
**Status:** Active Development

---

## Milestone 1: v0.5.0 - User Experience Improvements

**Goal:** Enhance configuration experience and add quality-of-life features

**Target:** 2026 Q2

**Success Criteria:**
- Users can verify API configuration before use
- Reduced configuration errors
- Improved feedback on connection issues

### Phases

#### Phase 1: AI Polishing Connection Test

**Goal:** Add connection test button for online AI polishing API configuration

**Requirements:**
- Test button in settings UI (only visible when "Online API" mode selected)
- Backend command to test API connection
- Display success/failure with detailed error messages
- Show response time on success
- Prevent duplicate clicks during test

**Acceptance Criteria:**
- [ ] Settings page has "Test Connection" button
- [ ] Button triggers API test with current configuration
- [ ] Success shows response time
- [ ] Failure shows specific error (network, auth, model not found, etc.)
- [ ] Test doesn't modify existing configuration

**Related:**
- Linear Issue: GRO-20
- Files: `settings.html`, `src/settings.ts`, `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`

**Estimated Effort:** 2-4 hours

---

## Future Considerations

These are potential future enhancements, not committed to any milestone:

- Windows platform support
- Automated model download
- Auto-update mechanism
- Plugin system for custom post-processing
- Multi-window support
- Customizable UI themes
- Automated testing framework
