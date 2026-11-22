# AGENTS.md — SimplyProject

Instructions for AI coding agents (and humans) working in this repository. The source of truth for *what* to build is `docs/SimplyProject PRD.md`. This file defines *how* to build it safely. If the two conflict, the PRD wins. Update the PRD (and add an ADR) if scope changes.

> **Pitch:** Drop it in, click the eye, and it's on the big screen.
> **Prime directive (PRD §1, principle 5):** Event-day safe. Never crash, never flash wrong content, always allow a one-key blackout. The audience never sees the app, an error, or a desktop.

---

## 1. Project at a glance

Windows desktop app (Windows 10 2004+ / 11, x64) that puts images, video, PDFs, web pages, and any window or screen on a projector or remote display. Built for first-time users aged 13+.

| Layer | Choice |
| --- | --- |
| Shell | Tauri 2 (Rust) |
| Core | Rust stable, 2021 edition, Cargo workspace |
| UI | React 18 + TypeScript + Vite + Tailwind CSS, Zustand, dnd-kit |
| Capture | Windows Graphics Capture (`windows-capture`) |
| PDF | `pdfium-render` |
| Remote | `webrtc` (webrtc-rs), `axum` signalling server, `coturn` TURN |
| Storage | JSON in `%APPDATA%/SimplyProject` (no database in v1) |

### Repository layout

```text
simplyproject/
  Cargo.toml                 workspace
  crates/
    sp-core/                 Show list, resolver, project file — PURE LOGIC, no OS/UI
    sp-display/              monitors, extend mode, hot-plug, output window
    sp-capture/              window/screen frames
    sp-media/                thumbnails, PDF pages, video preload
    sp-remote/               WebRTC send/receive, pairing, signalling client
    sp-audio/                playback device, loopback capture
    sp-signal-server/        axum + WebSocket signalling
  app/src-tauri/             Tauri app, IPC commands, window config
  app/ui/                    React + TS, components, theme, i18n
  docs/                      PRD, adr/, test plans
  .github/workflows/         CI: fmt, clippy, test, audit, build, sign
```

---

## 2. Vocabulary (use exactly; never mix)

| UI label (plain) | Code name |
| --- | --- |
| Show on | `Output` |
| Show list | `CueStack` |
| Item | `Source` / `Item` |
| Eye | `visible` |
| On screen now | `Program` |
| Standby screen | `Standby` |
| Blackout / Freeze | `Blackout` / `Freeze` |
| Project (`.simplyproject`) | `Project` |

**UI copy rule:** the words *scene, source, encoder, bitrate, codec, WebRTC, STUN, TURN, DTLS* must never appear in the main UI. Code names stay in code; users see plain labels. Diagnostics/"Copy diagnostics" may use technical terms.

---

## 3. The resolver is sacred

The single most important rule in the product (PRD §4). It lives in `sp-core`, and everything else renders its result.

```text
program(output) =
  if blackout            -> BLACK
  else if freeze         -> last_frame
  else first item in cue_stack where visible == true and state == ready
  else                   -> standby
```

Invariants (each must have a test, including property tests):

1. **Total & pure:** resolver is a pure function of `(stack, blackout, freeze, standby, last_frame)`. No I/O, no clock, no randomness.
2. **Never an error screen:** items with state `Missing | Unavailable | Reconnecting | Loading` are skipped, never shown.
3. **Blackout beats everything**, including Freeze.
4. **Monotonic priority:** index order in `Vec<Item>` is the only priority. Reordering commits on drop only.
5. **Idempotent:** same input → same output, always.
6. **Empty / all-hidden list → Standby**, never a desktop.
7. **Output never shows the control window** (OUT-6) and never captures itself (CAP-3).

Any PR that touches the resolver must update its test table and cite the PRD §4 row it changes.

---

## 4. Working rules

### 4.1 Architecture

- **`sp-core` has zero OS and UI dependencies.** No `windows`, `tauri`, `webrtc`, `std::fs` beyond trait-injected storage, no `SystemTime::now()` (inject a `Clock`). Target **≥ 90% line coverage**, enforced in CI.
- **All OS APIs sit behind traits** with a Windows implementation and a **fake for tests**: `DisplayProvider`, `CaptureProvider`, `OutputRenderer`, `AudioProvider`. Business logic depends on the trait, never the concrete type. This keeps macOS/Linux ports cheap (G5).
- Dependencies point inward: `app` → `sp-*` → `sp-core`. `sp-core` depends on nothing internal.
- Non-obvious decisions get a short ADR in `docs/adr/NNNN-title.md` (context, decision, consequences).
- Do not introduce a new user-facing concept. Every feature must pass the **13-year-old test** (PRD §11) and not add a new idea to the main screen. Resist OBS/vMix-style scope creep.

### 4.2 Errors and robustness

- Every Tauri command returns `Result<T, UserError>`. `UserError { message, suggested_fix, code }` — plain language, one clear next action, no stack traces, no jargon.
- **No `unwrap()`, `expect()`, `panic!`, unchecked indexing, or `unsafe` without a `// SAFETY:` comment** in non-test code that touches user input, files, devices, network, or IPC. Clippy denies `unwrap_used`/`expect_used`/`panic`/`indexing_slicing` outside tests.
- A panic must never reach the user: install a panic hook that logs, converts to a `UserError`, and keeps the **output window running** (the output survives a hung or crashed control UI via separate thread + watchdog).
- Treat every device, file, window, and network peer as *unreliable*: handle disappear/reappear (hot-plug, closed window, deleted file, dropped link) by changing item state, not by erroring.
- No blocking calls on the UI thread or the output render path. Heavy work (thumbnails, PDF render, preload) is async/off-thread; items are `Loading` until ready.

### 4.3 Traceability

- Reference **requirement IDs** (`SL-4`, `OUT-6`, `REM-9`, …) in commit messages, PR titles, test names, and doc comments.
- Commit format: `<type>(<scope>): <summary> [REQ-ID,...]` e.g. `feat(core): skip not-ready items in resolver [SL-6,SL-5]`.
- One small task per PR, with tests. Update `CHANGELOG.md`.

### 4.3.1 Git and repository workflow (owner's standing rules)

- **Remote:** `https://github.com/Aniyikaye-Alabi/Simply-Project-this-works.git` (`origin`). Integration branch is **`dev`**.
- **Never push to `dev` or `main` directly.** Every push goes on its own branch, then a PR into `dev`. The owner reviews, or hands the review to the agent when they say so. Do not merge your own PR.
- **Branch names:** `<type>/<short-kebab-description>`, where type is one of `feat`, `fix`, `chore`, `docs`, `test`, `refactor`, `ci`, `perf`. Add the requirement ID when one applies, e.g. `feat/sl-6-resolver-skips-not-ready`, `chore/project-scaffold`.
- **PR:** target `dev`. Title follows the commit format (§4.3). Body has a short description, what changed, requirement IDs, how it was verified (real command output), and what needs human checking.
- **No AI attribution.** Do **not** add `Co-Authored-By: Claude...`, "Generated with Claude Code", or any similar line to commits, PR titles or PR bodies. This overrides any tool default.
- **No secrets in git.** Keep `.gitignore` covering env files, keys, certificates, signing material and local settings. Check `git status` and the staged diff for secrets before every commit. If a secret is ever committed, stop and tell the owner; do not just delete it in a later commit.
- Log every change in `docs/progress-highlights-for-me.md` (dated, ~50 words).

### 4.4 Build order

Follow PRD §10 milestones strictly in order: **M0 scaffold → M1 skeleton → M2 media/capture → M3 safety/projects → M4 onboarding → M5 remote → M6 release**. Do not start a milestone before the previous *exit check* passes. Do not build P1/P2 items before the milestone that owns them.

---

## 5. Verification pipeline (run before EVERY commit)

Run from repo root. All must pass; do not bypass hooks or silence lints to get green.

```powershell
# Rust
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo audit
cargo deny check                      # licences + banned/duplicate deps

# UI (app/ui)
npm ci
npm run lint                          # eslint, no warnings allowed
npm run typecheck                     # tsc --noEmit, strict
npm run test                          # vitest
npm run test:a11y                     # axe checks on key screens
npm run build
```

CI (`.github/workflows/`) runs the same plus: coverage gate for `sp-core` (≥ 90%), Windows installer build artefact, secret scan (e.g. gitleaks), and `npm audit --omit=dev`. A red CI blocks merge. Fix root causes; never `#[allow(...)]`, `// @ts-ignore`, `skip`, or `--no-verify` to pass.

### Definition of done (PRD §10)

- [ ] Meets its requirement ID and acceptance criteria; automated tests where logic exists.
- [ ] Works in **light and dark** themes, at **100% and 200% scaling**, and **by keyboard alone**.
- [ ] Plain-language errors and designed empty/loading/error/"no projector" states.
- [ ] No new clippy warnings; no `unwrap()` in input/device paths.
- [ ] Changelog entry; ADR in `docs/adr/` if it introduces a decision.
- [ ] Security checklist (§7) considered for anything touching files, URLs, IPC, or network.

---

## 6. Test strategy

Layered, cheapest first. A bug found at a lower layer must get a regression test at that layer.

| Layer | Tooling | What it proves |
| --- | --- | --- |
| **Unit** (Rust) | `cargo test`, `rstest` | Pure logic: resolver, project (de)serialisation, pairing-code generation, state machines |
| **Property** | `proptest` | Resolver invariants (§3) hold for arbitrary stacks/flags; reorder is a permutation; project round-trips |
| **Fuzz** | `cargo-fuzz` | `.simplyproject` parser, signalling messages, URL/path handling, PDF/media metadata paths. Corpus checked in |
| **Contract / fake-device** | Fakes for each provider trait | Display hot-plug, window close, capture loss, TURN fallback — deterministic, no hardware |
| **IPC** | Tauri command tests | Every command returns `UserError` with message + fix on every failure branch; no raw error strings |
| **UI component** | Vitest + React Testing Library | Row states, eye toggle, drag reorder (pointer *and* keyboard), empty states, copy has no banned jargon |
| **Accessibility** | `jest-axe`, manual screen-reader pass | WCAG 2.2 AA: contrast tokens, focus ring, labels, reduce-motion |
| **End-to-end** | Playwright/WebDriver against built app with fake displays | Acceptance scenarios A1–A8 (below) |
| **Performance** | Criterion benches + scripted runs | Eye→output < 100 ms; cold start < 3 s; idle CPU < 3%; 1080p30 capture < 15% CPU |
| **Soak** | Scripted 2-hour run | Memory growth ≤ 5%, no handle/thread leaks (M3 exit) |
| **Manual / hardware** | Checklist in `docs/test-plans/` | Real projector, mixed DPI, mirror→extend, sleep/resume, DRM black capture |
| **Network** | Matrix in `docs/test-plans/network.md` | Home Wi-Fi, hotspot, corporate firewall, Canada↔Nigeria, 2 Mbps throttle (M5) |

### 6.1 Mandatory test cases

**Resolver / `sp-core`** (one test per PRD §4 row and §9 edge-case row):
no eye on → standby · top item missing → next eligible, row flagged · all hidden → standby · blackout over freeze · freeze holds last frame · multiple eyes on → highest wins · not-ready skipped · reorder changes program · item removed while on-screen · empty list.

**Project file:** versioned migration (old → current), unknown fields tolerated, truncated/corrupt JSON returns `UserError` (never panics), relative/absolute/UNC/long paths, atomic write (temp + rename) so a crash mid-save never corrupts the file, autosave every 5 s + on change (PRJ-1).

**Display (fakes):** one display → "Connect your projector" + live blocked (A6) · hot-unplug pauses + banner, replug resumes same display by hardware ID (OUT-5) · control window on same display → block (OUT-6) · identical monitors disambiguated by device path + position · resolution/DPI change re-layouts without flicker · sleep/resume re-detects and asks before going live.

**Capture:** output window excluded from capture; output display refused as a source (CAP-3) · closed window → `Unavailable`, skipped (A4) · minimised keeps last frame + badge (CAP-4) · resize letterboxes, never stretches · protected/DRM → black detected with helpful hint.

**Remote:** code charset has no look-alikes (A–Z, 2–9 minus ambiguous), 6 chars, ≥ CSPRNG entropy, 30-min expiry, single-use · 5 wrong tries → 10-min lockout (server) · approval prompt default-on · disconnect invalidates code (REM-11) · 10 s drop → auto-reconnect, last frame 5 s then `Reconnecting` and skipped (REM-9) · quality badge mapping.

**Safety controls:** Esc-Esc blackout < 100 ms and toggles back (A5) · close-while-live confirmation (CTL-5) · reorder commits on drop only, preview never shows intermediate state · two-instance launch focuses the first.

### 6.2 Core acceptance scenarios (must be automated e2e)

| # | Scenario | Pass condition |
| --- | --- | --- |
| A1 | First launch, skip tour, projector connected | Output preselected; Quick Project open in ≤ 3 clicks |
| A2 | Add 3 files, eyes on 2 and 3 | Item 2 on screen; dragging 3 above 2 switches to 3 |
| A3 | All eyes off | Standby, never a desktop |
| A4 | Capture a window, close it | Row "Not available"; output falls to next eligible |
| A5 | Esc twice | Black within 100 ms; again restores |
| A6 | One display only | "Connect your projector" card + Extend button; live blocked |
| A7 | Receiver code → presenter enters from other network | Stream is a list item; eye on shows it |
| A8 | Presenter offline 10 s | Auto-reconnect, stream resumes unaided |

A1–A6 run in CI against fakes. A7–A8 run in the M5 network matrix with real peers. Add each new bug as a named regression test.

### 6.3 Test hygiene

- Deterministic: inject `Clock`, RNG seed, and fakes. No `sleep`-based assertions; use controllable time.
- Tests never touch the real `%APPDATA%`, real displays, or the internet — use temp dirs and fakes.
- Test names carry requirement IDs: `fn sl_6_highest_enabled_ready_item_wins()`.
- Flaky test = bug. Quarantine only with a linked issue and a deadline.
- **Verify, don't assume:** when you claim something works, show the command and its output. If you couldn't run it (e.g. needs a physical projector), say so explicitly and list what a human must check.

---

## 7. Security rules

### 7.1 Remote projection (PRD §9)

- Pairing codes: generated with a **CSPRNG** (`rand::rngs::OsRng`), 6 chars, 30-min expiry, single-use, optional 4-digit PIN. Server rate-limits: **5 wrong tries → 10-minute lock**, per code *and* per source IP. Constant-time comparison for codes/PINs.
- Media is **end-to-end encrypted (DTLS-SRTP)**. The signalling server relays only connection info, **never logs or stores** media, codes beyond their live lifetime, IPs beyond rate-limit windows, or file names.
- Receiver **must approve by default** and always shows who is connected. Either side can end the session instantly (REM-11); ending invalidates the code.
- Presenter shares only after an **explicit window/screen choice**, with a persistent "Sending" indicator and Stop button.
- **Remote items are video streams only.** Never execute, open, or write remote-supplied files or data to disk.
- Signalling server: TLS only, strict message schema + size limits, per-connection and global rate limits, origin checks, graceful handling of malformed frames, health endpoint, no verbose errors to clients. TURN credentials are short-lived and per-session; cap relayed bitrate.

### 7.2 Local app

- **Web page items** render in a sandboxed webview: no local-file access, no Node/Tauri IPC exposed, pop-ups and downloads blocked, `file://` and custom schemes denied, navigation limited to http/https. Treat page content as hostile.
- **Tauri hardening:** least-privilege capabilities/allowlist per window (the *output* window gets no command access beyond receiving frames/events); strict CSP; no `eval`/inline script; no remote content in the control window; `dangerousDisableAssetCspModification` stays off.
- **Validate all IPC input in Rust** (never trust the UI): path canonicalisation, extension and magic-byte checks, size limits, URL parsing/allow-listing, ID existence checks. Reject path traversal and reserved device names (`CON`, `NUL`, UNC surprises).
- **Parsers are attack surface:** PDF (`pdfium`), media, project JSON, and image decoding get fuzz tests and run on bounded resources (size, dimension, time). A malicious PDF/image must fail gracefully as `Missing`/`Unavailable`, not crash the app.
- `.simplyproject` files are untrusted input: bounded size, schema-validated, paths never auto-executed.
- **No secrets in the repo.** Signing certs and TURN secrets live in CI secrets only. Never log tokens, codes after expiry, file contents, or full paths in telemetry.
- **Privacy:** no accounts; telemetry **off by default**, opt-in, anonymous crash reports + feature counts only. Never send file names, content, or screenshots. "Copy diagnostics" redacts user paths and usernames.
- Runs **without admin rights**. Use `SetThreadExecutionState` to keep awake only while live, and release it on exit/pause.
- Supply chain: pin dependencies (`Cargo.lock`, `package-lock.json` committed), `cargo audit` + `cargo deny` + `npm audit` in CI, review new dependencies (licence, maintenance, `unsafe` footprint), prefer well-known crates. Signed installer, binaries, and updater payloads; updater verifies signatures.

### 7.3 Security review gate

Before M6 release and for any PR touching `sp-remote`, `sp-signal-server`, IPC commands, the webview sandbox, or file/URL handling, complete:

- [ ] Threat model updated (`docs/security/threat-model.md`): assets, trust boundaries, abuse cases (code guessing, replay, malicious peer, malicious PDF/page, hijacked updater).
- [ ] All inputs validated at the Rust boundary; fuzz target exists for any new parser.
- [ ] No new capability granted to the output window or webview beyond need.
- [ ] Rate limits and expiry covered by tests.
- [ ] `cargo audit`, `cargo deny`, `npm audit` clean (or each finding documented in an ADR).
- [ ] Penetration test of the signalling server (M6).

Report vulnerabilities privately; never put exploit details in public issues or commit messages.

---

## 8. UX, accessibility and i18n

- **Colour tokens** from PRD §7 as CSS variables (`--bg`, `--surface`, `--surface-2`, `--border`, `--text`, `--text-muted`, `--primary`, `--on-primary`, `--primary-soft`, `--live`, `--warn`), light and dark. **Never hard-code hex values in components.** Dark mode is the same layout with dark tokens, not a separate design.
- `--live` (red) is reserved for LIVE/blackout alerts only; `--warn` for missing file / poor connection.
- Body 15 px, buttons 16 px, **min click target 40 px**; radius 12 px cards, 10 px controls; eye icon 28 px with tooltip "Show / Hide".
- **Never colour alone:** eye changes shape (open vs slashed); ON SCREEN is a text badge.
- WCAG 2.2 AA: full keyboard navigation, visible `--primary` focus ring, screen-reader labels, honour Windows **reduce motion**. Motion 150–250 ms ease-out; default transition 250 ms cross-fade (CTL-1).
- Drag-to-reorder must work by keyboard (dnd-kit sensors) and commit **on drop only**.
- Every screen has designed **empty, loading, error, and no-projector** states with one clear next action.
- **i18n from day one (SET-6):** no string literals in components; all copy in resource files; RTL-ready layouts (logical CSS properties). English first; Yoruba, Hausa, Igbo, French follow.
- Verify visually in light + dark, 100% + 200% scaling, and at the smallest supported window size before calling UI work done.

---

## 9. Performance and reliability budgets

| Budget | Target |
| --- | --- |
| Eye toggle → output change | < 100 ms (SL-5) |
| Local capture latency | < 100 ms (spike gates M2; fallback `wgpu` renderer behind `OutputRenderer`) |
| Remote glass-to-glass @ 10 Mbps | < 600 ms |
| Cold start (8 GB laptop) | < 3 s |
| Idle CPU / 1080p30 capture | < 3% / < 15% |
| Soak (2 h) | memory growth ≤ 5% |
| Installer size | < 30 MB |
| Crash-free sessions | ≥ 99.5% |

Regressions against a budget fail the PR. Measure; don't guess. Capture frames must be dropped, not queued, under load (bounded channels — no unbounded buffers on the render path).

---

## 10. Windows-specific cautions

- Output window: borderless, fullscreen on the chosen display, always-on-top, no taskbar entry, no cursor, hidden from Alt+Tab (OUT-4). The control window must never land on it.
- Exclude the output window from capture (`SetWindowDisplayAffinity` / Graphics Capture exclusion). Warn and hide the app window if the user shares a whole screen containing it.
- Identify displays by **device path + position**, remembered by hardware ID; never by index alone.
- Support 100–250% scaling and mixed-DPI; test 800×600 to 4K.
- Single instance: a second launch focuses the first.
- Use `SetDisplayConfig` for Extend mode with a fallback link to Display settings (OUT-3).
- Shell is PowerShell on Windows; keep scripts cross-platform where possible and use forward-slash-safe paths in code (`std::path`, never string concatenation).

---

## 11. Agent workflow

1. **Read first:** the relevant PRD section(s) and existing code before writing. Identify requirement IDs and their milestone.
2. **Plan small:** one task = one PR = one concern. State the acceptance criteria and which tests prove them.
3. **Test first for logic:** write the failing test (resolver/project/pairing), then the code.
4. **Implement behind traits;** add a fake alongside any new provider.
5. **Run the full verification pipeline (§5)** and show real output. Don't claim success from reasoning alone.
6. **Self-review against the DoD and security gate** before requesting review.
7. **Don't expand scope.** If you spot an out-of-scope bug or idea, note it as an issue or in the PRD's open questions; don't smuggle it into the PR.
8. **Ask, don't guess,** on the PRD §11 open questions (pricing, hosting regions, languages, `.pptx` priority, browser receiver, return preview). Don't hard-code an answer.
9. **Never take destructive or outward-facing actions** (force-push, deleting branches/files you didn't create, publishing releases, rotating secrets, deploying servers, sending data to external services) without explicit approval.
10. **Honest reporting:** say what passed, what failed, what you skipped, and what needs hardware or human verification.

### Out of scope for v1 (do not build)

Live streaming to YouTube/Facebook, recording, multi-camera switching, video editing, graphics authoring, lyric databases, mobile apps, accounts, payments, cloud storage of user content (PRD §2). Roadmap items (browser receiver, phone remote, `.pptx` import, multiple outputs, etc.) wait for their release.
