# Paw Dogfooding Bug Report Template

Use this template for all functional defects found during Phase 1 dogfooding.

---

## 1) Summary

- **Title**: 
- **Severity**: 
  - `P0` Crash / system down / blocker
  - `P1` Data loss / security issue / irreversible corruption
  - `P2` Functional defect (major behavior incorrect)
  - `P3` UX/usability issue (workflow friction, confusing behavior)
  - `P4` Cosmetic issue (layout/text/visual polish)
- **Area** (auth, messaging, websocket, media, profile, etc.): 

## 2) Environment

- **Date/Time found**: 
- **Tester name**: 
- **Client platform** (iOS/Android/Web/Desktop): 
- **Client build/branch**: 
- **Server commit/branch**: 
- **Backend endpoint** (e.g. `http://localhost:3000`): 

## 3) Preconditions

- User/account state:
- Conversation/message state:
- Network condition (normal/offline/flaky):

## 4) Steps to Reproduce

1. 
2. 
3. 

## 5) Expected Result


## 6) Actual Result


## 7) Evidence

- Screenshot/video:
- Logs (client/server):
- Request/response snippets (if relevant):

## 8) Frequency / Reproducibility

- [ ] Always (100%)
- [ ] Often (>50%)
- [ ] Sometimes (<50%)
- [ ] Rare / one-off

## 9) Impact Assessment

- **User impact**:
- **Business/release impact**:
- **Workaround exists?** (Yes/No + details):

## 10) Triage Fields

- **Owner**:
- **Status**: `New` / `Triaged` / `In Progress` / `Fixed` / `Verified` / `Won't Fix`
- **Target fix phase**: `Phase 1` / `Phase 2`
- **Linked issues/PRs**:

---

### Example Severity Guidance

- **P0**: App/server crash on startup, complete inability to send/receive messages.
- **P1**: Message loss, corrupted conversation history, token leakage.
- **P2**: Message send button fails while app remains usable.
- **P3**: Typing indicator confusing or delayed, profile workflow unclear.
- **P4**: Minor alignment/color/text label inconsistencies.
