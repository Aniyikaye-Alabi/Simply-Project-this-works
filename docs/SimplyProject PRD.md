# SimplyProject PRD

Oct 29, 2026 · @Aniyikaye

## 1. Overview

SimplyProject is a Windows desktop app that lets anyone put images, videos, slides, web pages or any window on a projector, TV or remote screen in under two minutes, with no technical knowledge. It replaces the manual "extend display and drag windows across" routine, and the learning curve of tools like OBS and vMix.

**One-line pitch:** Drop it in, click the eye, and it's on the big screen.

**Target user:** Event organisers, church and school media volunteers, conference hosts and speakers. Design for a first-time user aged 13+ who has the venue open right now and no time to read a manual.

**Product principles**

1. **Zero-config first run.** A working projection within 5 clicks of launching the app.
2. **The audience never sees the app.** The projector shows content only. All controls live on the operator's own screen.
3. **One mental model.** An ordered list of things to show. The first one with its eye switched on is what the audience sees.
4. **Plain language.** No words like "scene", "source", "encoder", "bitrate" or "codec" in the main UI.
5. **Event-day safe.** Never crash, never flash the wrong content, always offer a one-key blackout.

**Success metrics (first 90 days after launch)**

| Metric | Target |
| --- | --- |
| Time from install to first projection, new user | under 3 minutes (median) |
| First-time users who finish a projection without opening help | 80% or more |
| Crash-free sessions | 99.5% or more |
| Local capture latency (window to projector) | under 100 ms |
| Remote projection glass-to-glass latency on a 10 Mbps link | under 600 ms |
| Remote session setup success without router changes | 95% or more |

## 2. Problem, goals and non-goals

Projecting at an event today means manual display extension, dragging windows across screens, and juggling professional tools built for broadcast engineers. First-time organisers lose time, and mistakes show up live in front of the audience.

**Goals**

- G1. Project images, video, PDFs, web pages and any screen or window with no setup beyond choosing the display.
- G2. Let the operator control what the audience sees from a single, ordered, drag-to-reorder list with eye toggles.
- G3. Let a presenter anywhere in the world project to a display at the venue over the internet.
- G4. Look calm and friendly in light-green light mode and a matching dark mode, readable in a dim hall.
- G5. Ship Windows first, with an architecture that ports to macOS and Linux without rewrites of the core.

**Non-goals for v1**

- Live streaming to YouTube/Facebook, recording, or multi-camera switching (OBS and vMix territory).
- Video editing, lower-thirds, graphics authoring, lyric or song databases.
- Mobile apps. A browser receiver and phone remote are on the roadmap, not in v1.
- Accounts, payments and cloud storage of user content.

## 3. Personas and user journey

The primary persona is Funke, an organiser with no projection experience who is already at the venue and needs it working now.

| Persona | Who they are | What they need |
| --- | --- | --- |
| Funke, the organiser (primary) | Runs the event, collects guest slides, presents her own | One screen to line up everything and switch fast |
| Guest speaker | Hands over slides on a flash drive or by link | Their content shows without fuss, or they share their own screen |
| Remote presenter | In another country, e.g. Canada | Send their screen to the venue display reliably |
| Venue volunteer | Sets up the receiving projector at the venue, e.g. Nigeria | Connect a display, read out a code, done |

**Funke's journey**

1. Installs SimplyProject and opens it. A welcome screen offers a 4-card walkthrough or Skip.
2. Plugs in the projector. The app detects it and asks "Show on this screen?" She confirms. If Windows is mirroring, the app offers a one-click "Extend my screens".
3. She drags guest slides (PDF/PowerPoint export), a welcome video and a photo into the Show list.
4. She switches the eye on for the welcome video. The projector shows it. The list shows which item is live.
5. For a guest who brings a laptop, she adds a "Window or screen" item and picks their shared window. She turns its eye on and drags it above the video. It now takes over the projector.
6. For a speaker in Canada, she opens Remote, chooses "Receive" on the venue PC, reads the 6-character code to the speaker, who enters it and shares their screen. The venue projector shows it.
7. At the end she presses Esc twice for blackout and closes the app. The project autosaves for next time.

## 4. Core concepts and glossary

The whole product rests on one rule: the audience sees the first item in the Show list whose eye is on. Code and UI use the terms below, so agents and designers stay consistent.

| UI label (plain) | Code name | Meaning |
| --- | --- | --- |
| Show on | `Output` | A destination screen: a local display, or a remote device |
| Show list | `CueStack` | The ordered list of items. Top = highest priority |
| Item | `Source` | One thing that can be shown: image, video, PDF, web page, window, screen, colour/text slide |
| Eye | `visible` flag | On = eligible to be shown. Off = hidden |
| On screen now | `Program` | The single item the resolver picks for the Output |
| Standby screen | `Standby` | What shows when no item has its eye on (black by default, optional logo) |
| Blackout | `Blackout` | Instant override that shows black, ignoring the list |
| Freeze | `Freeze` | Holds the last frame while the operator changes things |
| Project | `Project` | A saved Show list + settings (`.simplyproject` file) |

**Resolution rule (the heart of the app)**

```text
program(output) =
  if blackout            -> BLACK
  else if freeze         -> last_frame
  else first item in cue_stack where visible == true and state == ready
  else                   -> standby
```

An item that is on but not ready (missing file, closed window, lost connection) is skipped and flagged with a warning badge, so the audience never sees an error screen.

## 5. Functional requirements

Priority: **P0** = must ship in MVP, **P1** = v1.0 before public launch, **P2** = after launch. Each requirement has an ID so agents can reference it in commits and tests.

### 5.1 Welcome and onboarding

| ID | Requirement | Pri |
| --- | --- | --- |
| ON-1 | First launch shows a welcome screen with two buttons: "Take a quick tour" and "Skip". | P0 |
| ON-2 | The tour is 4 cards with illustrations: (1) pick the screen, (2) add things, (3) use the eye, (4) drag to reorder. Back, Next and Skip are always visible. | P0 |
| ON-3 | After the tour or skip, the user lands in an untitled Quick Project, ready to use. No sign-in, no forms. | P0 |
| ON-4 | The tour can be replayed from Help. A "Don't show again" checkbox is stored in settings. | P1 |
| ON-5 | Empty-state hints: the empty Show list says "Drop files here, or press Add" with an arrow. | P0 |

### 5.2 Choosing where to show (Output)

| ID | Requirement | Pri |
| --- | --- | --- |
| OUT-1 | On launch, detect all displays. If a non-primary display exists, preselect it as "Show on" and ask for one-click confirmation. | P0 |
| OUT-2 | A "Show on" dropdown lists every display with name, resolution and a thumbnail. An "Identify" button flashes a big number on each physical screen. | P0 |
| OUT-3 | If only one display is found, show a friendly card: "Connect your projector or TV" with a button "Extend my screens" that switches Windows to extend mode (`SetDisplayConfig`) and a fallback link to Display settings. | P0 |
| OUT-4 | The output is a borderless fullscreen window on the chosen display, always on top, no taskbar entry, no cursor, hidden from Alt+Tab. The app's control window is never placed on it. | P0 |
| OUT-5 | Hot-plug: if the projector is unplugged, output pauses and the app shows a banner. When it returns, output resumes on the same display automatically. | P0 |
| OUT-6 | If the chosen display is the same as the control window's display, warn: "The audience would see this app. Pick another screen." and block going live. | P0 |
| OUT-7 | Output resolution follows the display's native mode. Content letterboxes with black bars by default. A "Fill screen" toggle per item crops to fill. | P1 |
| OUT-8 | Multiple outputs (e.g. main hall and overflow room), each with its own Show list tab. | P2 |

### 5.3 Show list (items, order, eye)

| ID | Requirement | Pri |
| --- | --- | --- |
| SL-1 | Add items by dragging files onto the panel, the Add button, or paste (images, URLs). Drag from Explorer works for multiple files at once. | P0 |
| SL-2 | Supported file types in MVP: images (PNG, JPG, WEBP, GIF, BMP), video (MP4, MOV, WEBM, MKV via system codecs), PDF, and slides exported as PDF. | P0 |
| SL-3 | Each row shows: drag handle, thumbnail, name, type icon, status badge, the **eye** toggle, and a menu (rename, remove, fill mode, loop, mute). | P0 |
| SL-4 | Drag a row to reorder. Top row = highest priority. Reordering updates the output immediately (with the current transition). | P0 |
| SL-5 | Eye on/off changes the output within 100 ms. The row currently on screen gets a green "ON SCREEN" badge; other eye-on rows show "Next in line" for the first one beneath it. | P0 |
| SL-6 | Multiple eyes can be on. The resolver always shows the highest enabled, ready item (see section 4). | P0 |
| SL-7 | A live preview pane above the list shows exactly what the audience sees, plus the name of the item. | P0 |
| SL-8 | Videos: play/pause, seek, loop, mute, volume on the row. Video starts when it becomes the on-screen item, and pauses (not restarts) when something above it takes over. | P0 |
| SL-9 | PDFs/slides: Next/Previous page buttons on the row, plus keyboard Left/Right/Page Up/Down when the operator window is focused, plus optional global hotkeys. Page state is remembered per item. | P0 |
| SL-10 | Web page item: paste a URL and the app renders it in the output. Optional auto-refresh interval. | P1 |
| SL-11 | Colour or text slide item ("Welcome to our event") with big-font editor and theme colours. Handy as a holding slide. | P1 |
| SL-12 | Undo and redo for list actions (Ctrl+Z, Ctrl+Y). | P1 |
| SL-13 | Select multiple rows to toggle eyes, remove or group. Groups ("Speaker 1") collapse and reorder as one. | P2 |

### 5.4 Screen and window capture

| ID | Requirement | Pri |
| --- | --- | --- |
| CAP-1 | "Add window or screen" opens a visual picker with live thumbnails of every screen and open window, grouped as Screens and Windows. | P0 |
| CAP-2 | Capture uses the Windows Graphics Capture API at the display's refresh rate up to 30 fps by default, 60 fps option. | P0 |
| CAP-3 | Never capture the output display itself, and always exclude the output window from any screen capture, to prevent the mirror-tunnel effect. If a user picks it, explain why and block. | P0 |
| CAP-4 | A captured window that is minimised keeps its last frame and shows a "Window is minimised" badge on the row. Closed windows turn the row to "Not available" and the resolver skips it. | P0 |
| CAP-5 | Show or hide the mouse cursor in captures (default: show). | P1 |
| CAP-6 | System audio and per-window audio capture (WASAPI loopback) with a mute toggle, so shared videos have sound. | P1 |
| CAP-7 | Capture of a webcam or capture card (HDMI input) as an item. | P2 |

### 5.5 Transitions, safety and controls

| ID | Requirement | Pri |
| --- | --- | --- |
| CTL-1 | Default transition is a 250 ms cross-fade. A setting switches to instant cut. | P0 |
| CTL-2 | **Blackout** big button, plus Esc pressed twice anywhere in the control window. Pressing again restores. A red "BLACKOUT" bar shows while active. | P0 |
| CTL-3 | **Freeze** button holds the last frame on the output, handy while changing windows. | P1 |
| CTL-4 | A clear status bar: output display name, resolution, "LIVE" or "Paused", and warnings. | P0 |
| CTL-5 | Closing the control window while live asks "The projector will go blank. Close anyway?" | P0 |
| CTL-6 | Test pattern button shows a bright grid with the screen name and resolution, to check alignment and cabling. | P1 |
| CTL-7 | Global hotkeys (configurable): blackout, next page, previous page, toggle top item. | P1 |

### 5.6 Projects and files

| ID | Requirement | Pri |
| --- | --- | --- |
| PRJ-1 | The app autosaves the current project every 5 seconds and on every change. After a crash it reopens exactly as left, output paused until the user clicks Resume. | P0 |
| PRJ-2 | Save, Save As, Open and Recent. Format: `.simplyproject` (JSON, versioned). Files are referenced by path, not copied. | P0 |
| PRJ-3 | Missing files show a "Find file" action that relinks the file, and offers to relink siblings in the same folder. | P1 |
| PRJ-4 | "Collect files" copies all referenced media into a project folder so the event can run from another PC. | P2 |
| PRJ-5 | Thumbnails and preloaded video frames are cached so switching is instant. | P0 |

### 5.7 Remote projection

| ID | Requirement | Pri |
| --- | --- | --- |
| REM-1 | A Remote tab with two big choices: **Send my screen** (presenter) and **Receive and show** (venue). | P0 |
| REM-2 | Receiver generates a 6-character pairing code (A-Z, 2-9, no look-alikes) valid for 30 minutes, renewable. Optional 4-digit PIN. | P0 |
| REM-3 | Presenter enters the code, then picks what to send: a window, a screen, or "my Show list output". They see a clear "Sending" indicator and a Stop button at all times. | P0 |
| REM-4 | On the receiver, an incoming remote stream appears in the Show list as a normal item ("Funke from Canada") with its own eye. The operator decides when it goes on screen, by position and eye, like any other item. | P0 |
| REM-5 | Receiver can accept without a prompt (code is the consent) or ask "Allow Funke to project?". Default: ask. | P0 |
| REM-6 | Connection works across NAT and firewalls without port forwarding (STUN, with TURN relay fallback). | P0 |
| REM-7 | Adaptive quality: auto-lowers resolution, frame rate and bitrate on poor links. Mode toggle: "Slides" (sharp text, low fps) and "Video" (smooth motion). Default: Slides. | P0 |
| REM-8 | Audio from the presenter can be sent and played on the receiver's audio device. | P1 |
| REM-9 | Reconnect automatically within 10 seconds of a drop. While down, the receiver shows the last frame for 5 s, then the item is marked "Reconnecting" and the resolver skips it. | P0 |
| REM-10 | Both sides see connection quality (Good / Fair / Poor) and round-trip time in plain words. | P1 |
| REM-11 | Operator can disconnect any remote at any time, and the code becomes invalid. | P0 |
| REM-12 | Browser receiver and presenter (no install) via a link. | P2 |

### 5.8 Settings and help

| ID | Requirement | Pri |
| --- | --- | --- |
| SET-1 | Theme: Light (green), Dark (green), or Match Windows. Applies instantly. | P0 |
| SET-2 | Standby screen: black, a colour, or a chosen logo image. | P1 |
| SET-3 | Default transition, default capture frame rate, hardware acceleration on/off, audio output device. | P1 |
| SET-4 | Start with last project, start minimised, check for updates. | P1 |
| SET-5 | Help: tour replay, keyboard shortcuts sheet, "Fix my display" troubleshooter, and Copy diagnostics (logs and system info) to the clipboard. | P1 |
| SET-6 | Language framework (i18n) from day one. English first, then Yoruba, Hausa, Igbo, French as follow-ups. | P1 |

## 6. Gaps filled and suggested features

The original brief left several real-world gaps. Each is resolved below and is already reflected in section 5 where marked.

| Gap in the brief | Decision |
| --- | --- |
| What happens when no eye is on? | Standby screen (black by default, or logo). Never a desktop or an error. |
| What if a file is missing or a window closes mid-event? | Item is skipped by the resolver and flagged; audience never sees an error (section 4). |
| Guests bring PowerPoint, not PDF | MVP: export to PDF, or open it in PowerPoint and add "Window or screen". v1.1: import .pptx by converting through PowerPoint (if installed) or LibreOffice (optional download). |
| Hall-of-mirrors if someone captures the projector screen | Output window excluded from capture and the output display is blocked as a source (CAP-3). |
| Projector unplugged or display order changes | Hot-plug handling and remembered display by hardware ID (OUT-5). |
| Windows is in mirror (Duplicate) mode | One-click "Extend my screens" (OUT-3). |
| Sound for videos and shared screens | Local audio on the chosen device; WASAPI loopback capture (CAP-6); remote audio (REM-8). |
| Laptop sleeps, screensaver or notification pops up on the projector | Keep display awake while live (`SetThreadExecutionState`) and mute system notifications on the output display. Banner reminding to turn on Do Not Disturb. |
| Wrong content flashes during reordering | Reorder commits on drop only; the preview shows what the audience sees at all times. |
| Venue internet is poor (common in many venues) | Remote quality modes, adaptive bitrate, and a local-only mode that works fully offline (REM-7). |
| Security of remote sessions | Pairing code with expiry, optional PIN, receiver approval, end-to-end encrypted media (section 9). |
| First-time Windows warnings | Signed installer to avoid SmartScreen warnings; WebView2 bootstrapper included. |

**Suggested features beyond the brief**

| Feature | Why it matters | Tier |
| --- | --- | --- |
| Blackout, Freeze, Test pattern | The three panic buttons every operator wants | MVP |
| Live preview of what the audience sees | Removes anxiety about the unseen screen | MVP |
| Playlist mode: auto-advance images or PDFs every N seconds, loop | Holding slides and sponsor loops before the event | v1.0 |
| Countdown and clock item ("Starting in 04:32") | Very common at events | v1.0 |
| Lower-third or caption bar (name and title of the speaker) | Looks professional with zero design skill | v1.1 |
| Phone as remote (scan a QR, tap next/previous, blackout) | Operator can walk around the room | v1.1 |
| Browser receiver (no install; open a link on any TV browser or Smart TV) | Removes install friction at the venue | v1.1 |
| Speaker view: notes and next slide on the operator screen for PDFs | Presenters need it | v1.1 |
| Templates: "Church service", "Conference", "Wedding", "Classroom" | A pre-built list structure for common events | v1.1 |
| Multiple outputs (main hall and overflow) | Larger venues | v1.2 |
| Scheduled cues: a time or trigger switches an eye on | Event running order | v1.2 |
| Song lyrics and scripture module | Big for church users in the target market | v2 |
| Companion web viewer for guests to follow slides on their phones | Audience engagement | v2 |
| macOS and Linux builds | Already planned | v2 |

## 7. UX and visual design

The look is calm, soft green, high-contrast text and large touch-friendly targets, so it stays readable in a bright hall and a dark control booth.

**Main window layout** (single window, no nested menus)

```text
+--------------------------------------------------------------+
| SimplyProject   [Show on: Epson Projector v]   LIVE   Theme  |
+------------------------------+-------------------------------+
|  ON SCREEN NOW (live preview)|  SHOW LIST                    |
|  +------------------------+  |  [+ Add]  [Window/Screen]     |
|  |                        |  |  = [thumb] Welcome.mp4  (eye) |
|  |   what audience sees   |  |  = [thumb] Guest1.pdf   (eye) |
|  |                        |  |  = [thumb] Funke - CA   (eye) |
|  +------------------------+  |  = [thumb] Logo.png     (eye) |
|  [Prev] [Play] [Next]        |                               |
|  [Blackout]  [Freeze]  [Test]|  Drop files here              |
+------------------------------+-------------------------------+
| Status: Projecting to Display 2 - 1920x1080 - 3 items        |
+--------------------------------------------------------------+
```

**Colour tokens** (define as CSS variables; contrast figures are against the listed text colour)

| Token | Light mode | Dark mode | Use |
| --- | --- | --- | --- |
| `--bg` | #F3F9F4 | #0F1712 | App background |
| `--surface` | #FFFFFF | #16211A | Cards and panels |
| `--surface-2` | #E7F3EA | #1D2B22 | Rows, hover, inputs |
| `--border` | #D3E5D8 | #2A3C31 | Dividers |
| `--text` | #1B2B21 | #E3EFE7 | Body text (above 12:1) |
| `--text-muted` | #4F6657 | #9DB5A5 | Secondary text (above 4.5:1) |
| `--primary` | #2A8554 | #6CC28C | Buttons, eye on, focus |
| `--on-primary` | #FFFFFF | #0B1A11 | Text on primary (4.6:1 and above) |
| `--primary-soft` | #CFEAD7 | #23402E | Selected row, chips |
| `--live` | #D93F3F | #FF6B6B | LIVE, blackout, recording-style alerts only |
| `--warn` | #B26A00 | #F2B84B | Missing file, poor connection |

**Design rules**

- Font: Inter or Segoe UI Variable, body 15 px, buttons 16 px, minimum click target 40 px.
- Corner radius 12 px on cards, 10 px on controls. Subtle shadows in light mode, flat in dark.
- Never rely on colour alone: eye icon changes shape (open and slashed), ON SCREEN uses a text badge.
- The eye is the most prominent control on each row, 28 px, with a tooltip "Show / Hide".
- Motion: 150 to 250 ms ease-out. Respect Windows "reduce motion".
- Full keyboard navigation, visible focus ring using `--primary`, WCAG 2.2 AA contrast.
- Empty, loading, error and "no projector" states are designed for every screen, with one clear next action.
- Dark mode is the same layout with the dark tokens, never a separate design.

## 8. Technical architecture

Recommended stack: a Rust core with a Tauri 2 shell and a React + TypeScript interface. Rust owns everything performance- or OS-critical (displays, capture, media, networking), and the web UI owns the friendly screens, which makes theming, drag-and-drop and onboarding quick to build and easy for agents to iterate on.

| Layer | Choice | Why |
| --- | --- | --- |
| App shell | Tauri 2 (Rust) | Small installer (about 10 MB), native windows, multi-window, uses WebView2 already on Windows 11 |
| Core language | Rust (stable, 2021 edition) | Safety, performance, and the OS-specific parts sit behind traits for later macOS and Linux ports |
| UI | React 18 + TypeScript + Vite + Tailwind CSS | Large ecosystem, easy theming with CSS variables |
| UI state and drag | Zustand, dnd-kit | Simple store; accessible drag-to-reorder with keyboard support |
| Windows APIs | `windows` crate | Monitors, `SetDisplayConfig`, power requests, global hotkeys |
| Capture | Windows Graphics Capture via `windows-capture` crate | Modern, GPU-based, per-window and per-screen, can exclude windows |
| Media (video) | WebView2 HTML5 video for MP4/WebM; fallback to FFmpeg sidecar for odd formats | Uses system hardware decoding, no heavy dependencies at first |
| PDF | `pdfium-render` | Fast, accurate page rendering to bitmaps |
| Remote transport | `webrtc` (webrtc-rs) | NAT traversal, encryption by default, adaptive bitrate |
| Signalling server | Rust `axum` + WebSocket, with `coturn` for TURN | Tiny stateless service. Only exchanges connection info, never media except through TURN relay |
| Storage | JSON files in `%APPDATA%/SimplyProject` (settings, projects, cache index) | No database needed for v1 |
| Installer and updates | Tauri bundler (NSIS or MSI), Tauri updater, code signing | Auto-update and no SmartScreen warnings |
| Minimum OS | Windows 10 version 2004 and Windows 11 | Needed for Graphics Capture without yellow borders |

**Alternatives considered**

- Pure Rust UI (egui or iced): fewer moving parts, but harder to reach the friendly, polished look and theme in v1.
- Electron: easiest screen capture, but 150 MB+ install and weaker control of OS-level display behaviour.
- .NET (WPF/WinUI): excellent Windows fit, but ports poorly to macOS and Linux later.

**How the pieces connect**

```text
Control window (React)  <--Tauri IPC-->  Rust core
                                          |-- sp-core    : Show list, resolver, project file (pure logic)
                                          |-- sp-display : monitors, extend, hot-plug, output window
                                          |-- sp-capture : window/screen frames (Graphics Capture)
                                          |-- sp-media   : thumbnails, PDF pages, video preload
                                          |-- sp-remote  : WebRTC send/receive, pairing, signalling client
                                          '-- sp-audio   : playback device, loopback capture
Output window (fullscreen, on projector)  <-- frames/events -- resolver result
Signalling server (axum) + TURN relay  <--->  sp-remote on both PCs
```

**Rendering the output.** The output window is a second, borderless Tauri window on the chosen display. Images, video, PDF pages and web pages render natively in it. Captured windows and remote streams reach it as a hardware-decoded video stream over a loopback WebRTC connection, so local and remote capture share one code path. Milestone 2 starts with a time-boxed spike to confirm latency under 100 ms. The fallback is a native `wgpu` renderer for the output window, which the `OutputRenderer` trait already isolates.

**Remote projection flow.** The receiver registers with the signalling server and gets a pairing code. The presenter enters it, both sides exchange WebRTC offers and answers, media flows peer to peer (TURN relay if blocked). Media is encrypted with DTLS-SRTP, so even the relay cannot read it. The presenter's Rust core captures, encodes H.264 with hardware encoders (Media Foundation), and sends. Default 1080p at 5 to 15 fps in Slides mode, up to 30 fps in Video mode, with automatic downscaling.

**Repository layout**

```text
simplyproject/
  Cargo.toml                (workspace)
  crates/ sp-core  sp-display  sp-capture  sp-media  sp-remote  sp-audio
  crates/ sp-signal-server
  app/src-tauri/            (Tauri app, IPC commands, windows config)
  app/ui/                   (React + TS, components, theme, i18n)
  docs/                     (PRD, ADRs, test plans)
  .github/workflows/        (CI: fmt, clippy, test, build, sign)
```

**Key data model** (Rust, serialised to the project file with `serde`)

```rust
struct Project { version: u32, id: Uuid, name: String, output: OutputConfig,
                 items: Vec<Item>, settings: ProjectSettings }
struct Item { id: Uuid, name: String, visible: bool, fill: FillMode,
              kind: ItemKind }                    // order in Vec = priority
enum ItemKind { Image{path}, Video{path, loop_, muted, volume},
                Pdf{path, page}, Web{url, refresh_secs},
                Capture{target: CaptureTarget, fps, cursor, audio},
                Slide{text, bg, fg}, Remote{peer_id, label} }
enum OutputTarget { Local{display_id: String}, Remote{peer_id: String} }
enum ItemState { Ready, Loading, Missing, Unavailable, Reconnecting }
```

**IPC contract** (Tauri commands and events; every command returns `Result<T, UserError>` where `UserError` has a plain-language message and a suggested fix)

| Command or event | Purpose |
| --- | --- |
| `list_displays()` / event `displays_changed` | Monitors with id, name, size, primary flag |
| `extend_displays()` | One-click Windows extend mode |
| `set_output(display_id)` / `identify_displays()` | Choose and flash numbers |
| `add_items(paths_or_urls)` / `remove_item(id)` | Build the list |
| `reorder_items(ordered_ids)` | Commit new priority order |
| `set_visible(id, bool)` | The eye |
| `item_action(id, action)` | play, pause, seek, next\_page, prev\_page |
| `list_capture_targets()` | Windows and screens with thumbnails |
| `set_blackout(bool)` / `set_freeze(bool)` | Safety controls |
| event `program_changed{item_id}` | Resolver result, drives the ON SCREEN badge |
| `remote_create_code()` / `remote_connect(code)` / `remote_disconnect()` | Pairing |
| event `remote_state{peer, quality, rtt}` | Connection health |
| `save_project()` / `open_project(path)` / `get_settings()` / `set_settings()` | Persistence |

## 9. Non-functional requirements, security and edge cases

**Non-functional requirements**

| Area | Requirement |
| --- | --- |
| Performance | Cold start under 3 s on a mid-range laptop (8 GB RAM). Eye toggle to output under 100 ms. Idle CPU under 3% and one 1080p30 capture under 15% CPU on a GPU-capable machine. |
| Reliability | Output keeps running if the control UI hangs (separate process threads and watchdog). Autosave and crash recovery (PRJ-1). Zero unhandled panics reach the user; all errors become plain-language messages. |
| Offline | Everything except remote projection works with no internet. |
| Accessibility | WCAG 2.2 AA, full keyboard use, screen-reader labels, reduce-motion support. |
| Privacy | No accounts. Telemetry is off until the user opts in; when on, only anonymous crash reports and feature counts. Never send file names, content or screenshots. |
| Compatibility | Windows 10 (2004+) and 11, x64 first, ARM64 later. 100%-250% display scaling. Projectors from 800x600 to 4K, mixed-DPI setups. |
| Localisation | All strings in resource files; right-to-left ready. |
| Footprint | Installer under 30 MB. No admin rights needed to run. |

**Security (remote projection)**

- Pairing codes are random, 6 characters, expire in 30 minutes, are single-use, and are rate-limited on the server (5 wrong tries then locked for 10 minutes).
- Media is end-to-end encrypted (DTLS-SRTP). The signalling server never sees content and stores nothing beyond the live code.
- The receiver must approve by default, and always shows who is connected. Any side can end the session instantly.
- The presenter's screen is only shared after an explicit choice of window or screen, with a persistent "Sending" indicator.
- Never execute or open remote-supplied files. Remote items are video streams only.
- Web page items run in a sandboxed webview with no access to local files, and block pop-ups and downloads.
- Installer, binaries and updates are signed. Dependencies scanned in CI (`cargo audit`).

**Edge cases the build must handle**

| Situation | Expected behaviour |
| --- | --- |
| No item has its eye on | Show the standby screen |
| Top item is on but its file is deleted | Skip it, flag the row, show next eligible item |
| Two displays have the same resolution and model | Identify by Windows device path plus position; confirm by Identify button |
| Display resolution or scale changes while live | Re-layout the output window without flicker |
| Captured window changes size | Letterbox, never stretch; re-fit smoothly |
| Capture of DRM or protected video returns black | Show a clear hint: "This app blocks screen sharing. Try a file instead." |
| Laptop lid closed or sleep | Prevent sleep while live; on resume, re-detect displays and ask before going live |
| Remote link drops and returns | Auto-reconnect (REM-9) and resume at the same list position |
| The presenter shares the whole screen including the SimplyProject window | Warn and hide the app window from the capture |
| Very large file (4K, 2 GB video) | Show "Preparing..." with progress. The item is not ready until preloaded |
| User drags a folder | Add its supported files in natural name order |
| Two instances launched | Second one focuses the first |

## 10. Agentic build plan

Build in seven vertical milestones, each ending in something runnable and demonstrable. Do them in order; every milestone must pass its exit check before the next begins.

**Working rules for coding agents**

- Keep `sp-core` (Show list, resolver, project file) free of OS and UI code, with 90%+ unit-test coverage. It is the source of truth for section 4.
- Hide every OS-specific API behind a trait (`DisplayProvider`, `CaptureProvider`, `OutputRenderer`, `AudioProvider`) with a Windows implementation and a fake for tests. This is what makes macOS and Linux later cheap.
- Reference requirement IDs (for example `SL-4`) in commit messages, tests and PR titles.
- No user-facing technical jargon. Every error returns a `UserError` with a plain message and a fix suggestion.
- Record non-obvious decisions as short ADRs in `docs/adr/`. Update this PRD if scope changes.
- Each task is small enough to land in one PR with tests. Run `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, and the UI lint and tests before every commit.

**Milestones**

### M0. Scaffold and CI

- [ ] Cargo workspace with the crates in section 8, Tauri 2 app, React + Vite + Tailwind UI, theme tokens from section 7 as CSS variables.
- [ ] GitHub Actions: fmt, clippy, test, UI build, Windows installer build artefact.
- [ ] Light and dark theme switch working on an empty shell.

*Exit check:* CI is green and an installer produces a window that opens and toggles themes.

### M1. Walking skeleton: images on a second screen

- [ ] `sp-core`: Project, Item, resolver with tests for every row in section 4 and the edge-case table.
- [ ] `sp-display`: list displays, identify, output window on chosen display, hot-plug events (OUT-1, 2, 4, 5, 6).
- [ ] UI: Show list with drag-and-drop add, reorder, eye, thumbnails and live preview, for images only (SL-1, 3 to 7).
- [ ] "Extend my screens" helper (OUT-3).

*Exit check:* drop 3 images, toggle eyes, drag order: projector output matches the resolver every time, verified by an automated test using a fake display and a manual test on a real projector.

### M2. Media and capture

- [ ] **Spike (time-box 3 days):** loopback WebRTC from capture to the output window. Pass if latency is under 100 ms at 1080p30; otherwise implement the `wgpu` renderer fallback.
- [ ] Video items with playback controls and preload (SL-8). PDF items with page control (SL-9).
- [ ] Window and screen picker with live thumbnails, capture items, output-window exclusion (CAP-1 to 5).
- [ ] Audio output device and loopback capture (CAP-6).

*Exit check:* a Zoom or Google Meet window, a PowerPoint window and a 1080p video can each be placed in the list and switched by eye with no visible tearing.

### M3. Safety, projects and reliability

- [ ] Blackout, Freeze, Test pattern, transitions, status bar, close confirmation (CTL-1 to 6).
- [ ] Autosave, crash recovery, Save, Open, Recent, relink missing files (PRJ-1 to 3, 5).
- [ ] Keep-awake while live, standby screen, web page and text-slide items (SL-10, 11, SET-2).

*Exit check:* kill the process mid-show and restart; project returns as left. Unplug and replug a display; output resumes. 2-hour soak test with no memory growth above 5%.

### M4. Onboarding, settings and polish

- [ ] Welcome screen and 4-card tour with skip, replay, empty states (ON-1 to 5).
- [ ] Settings screen, theme match with Windows, i18n framework with English strings (SET-1 to 6).
- [ ] Undo/redo, global hotkeys, keyboard accessibility pass, usability test with 5 first-time users aged 13 to 60.

*Exit check:* 4 of 5 first-time testers complete "project a photo to the projector" in under 3 minutes with no help.

### M5. Remote projection

- [ ] `sp-signal-server` (axum + WebSocket + coturn config), deployed with TLS, rate limiting and health checks.
- [ ] `sp-remote`: pairing, WebRTC send and receive, adaptive quality, reconnect, audio (REM-1 to 11).
- [ ] Remote tab UI, receiver approval prompt, "Sending" indicator, connection quality badge.
- [ ] Network test matrix: home Wi-Fi, mobile hotspot, corporate firewall, Canada to Nigeria, and a 2 Mbps throttled link.

*Exit check:* a presenter on another continent projects a window to the venue display with under 600 ms delay at 10 Mbps, and recovers from a 10 s outage on its own.

### M6. Hardening and release

- [ ] Signed installer, auto-updater, WebView2 bootstrapper, uninstaller cleanup.
- [ ] Crash reporting (opt-in), diagnostics export, "Fix my display" troubleshooter.
- [ ] Security review of pairing and signalling, `cargo audit`, penetration test of the signalling server.
- [ ] Closed beta with 10 real events, feedback loop, bug fix sprint.

*Exit check:* all P0 and P1 requirements pass acceptance tests and the success metrics in section 1 are measurable.

**Definition of done for any task**

- Meets its requirement ID and acceptance criteria, with automated tests where logic exists.
- Works in light and dark themes, at 100% and 200% scaling, and by keyboard alone.
- Has plain-language error messages and empty states.
- No new clippy warnings, no `unwrap()` in non-test code paths that handle user input or devices.
- Documented in the changelog, and in `docs/` if it introduces a decision.

**Core acceptance scenarios (end-to-end tests)**

| # | Scenario | Pass condition |
| --- | --- | --- |
| A1 | First launch, skip tour, projector connected | Output display preselected and a Quick Project is open in 3 clicks |
| A2 | Add 3 files, turn on eyes for items 2 and 3 | Item 2 is on screen. Dragging item 3 above item 2 switches the output to item 3 |
| A3 | Turn all eyes off | Standby screen shown, never a desktop |
| A4 | Add a window capture, close that window | Row shows Not available. Output falls to the next eligible item |
| A5 | Press Esc twice | Output goes black within 100 ms; Esc twice again restores |
| A6 | Only one display connected | "Connect your projector" card and Extend my screens button appear; going live is blocked |
| A7 | Receiver creates a code, presenter enters it from another network | Stream appears as a list item; eye on shows it on the venue display |
| A8 | Disable network on the presenter for 10 s | Auto-reconnect and the stream resumes without manual action |

## 11. Risks, open questions and roadmap

**Risks**

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Loopback WebRTC into the output window adds too much latency | Captured content feels laggy | Time-boxed spike in M2; `wgpu` renderer fallback behind `OutputRenderer` |
| Slow or unstable venue internet breaks remote projection | Presenter looks bad live | Adaptive Slides mode, auto-reconnect, and the fallback of sending a PDF to the venue operator |
| TURN relay bandwidth costs grow | Hosting spend | Prefer direct peer-to-peer; cap relayed bitrate; self-hostable relay later |
| DRM-protected content captures black | User confusion | Detect and explain; recommend file playback |
| Windows SmartScreen warns on an unsigned installer | Lost trust at first run | Code-signing certificate before beta |
| Scope creep toward OBS-style features | Violates the simplicity promise | Every new feature must pass the "13-year-old test" and not add a new concept to the main screen |
| Porting to macOS (ScreenCaptureKit) and Linux (PipeWire) differs | Schedule | Provider traits from day one; keep OS code out of `sp-core` |

**Open questions for you**

1. Pricing and licensing: free, freemium, or paid per event? This affects licensing and update design.
2. Is a browser-based receiver (no install on the venue TV) important enough to move from v1.1 into v1.0?
3. Which languages first after English: Yoruba, Hausa, Igbo, French?
4. Do guests mostly bring PowerPoint? If yes, native .pptx import should move earlier.
5. Who hosts the signalling and relay servers (your own cloud, or a managed provider), and in which regions? A server in Africa and one in North America would cut latency for your Canada to Nigeria case.
6. Should the remote presenter be able to see the venue's output (a return preview)? It is low cost and builds trust.

**Roadmap**

| Release | Scope |
| --- | --- |
| MVP (M0 to M3) | Local projection: images, video, PDF, window and screen capture, eye and priority list, safety controls, autosave |
| v1.0 (M4 to M6) | Onboarding, themes, settings, remote projection, signed installer, closed beta |
| v1.1 | PowerPoint import, browser receiver, phone remote, speaker view, templates, captions and lower-thirds |
| v1.2 | Multiple outputs, scheduled cues, playlists and countdown polish |
| v2 | macOS and Linux, lyrics and scripture module, audience companion viewer |
