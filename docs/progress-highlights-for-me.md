# Progress Highlights

A short, dated log of everything done on SimplyProject, newest at the bottom. Each entry is about 50 words. Add a new entry whenever something is added, updated or changed.

## 2026-10-29

**1. Project instructions written.** Read the PRD and created `AGENTS.md` at the project root. It sets the rules for building safely: the resolver as the core rule, architecture limits, a pre-commit checklist, a layered test plan, security rules for remote projection, performance targets, and a step-by-step agent workflow.

**2. PRD moved to docs.** You moved the PRD into the `docs` folder. I updated the path in `AGENTS.md` to `docs/SimplyProject PRD.md`, removing the "currently in the root" note. I also checked the machine: Node 24 and git were installed, but Rust was not, so the Rust scaffold had to wait.

**3. Repository set up.** Ran `git init` and added `.gitignore` (build output, secrets, certificates), `CHANGELOG.md`, and an empty `docs/adr/` folder for future decision records. No commit has been made yet. Nothing has been pushed anywhere.

**4. UI scaffold built.** Created `app/ui` with React 18, TypeScript, Vite and Tailwind. Added a strict content security policy, strict TypeScript settings, and ESLint that rejects warnings. Installed Zustand for state. The page is a simple shell with a header and the "Drop files here, or press Add" empty-state hint.

**5. Light and dark theme added.** Put the PRD colour tokens into `index.css` as CSS variables for light and dark. Built a theme switch (Light, Dark, Match Windows) that applies instantly, remembers the choice, and still works if browser storage is blocked. All text lives in `en.json`, ready for translation.

**6. UI tests added and passing.** Wrote five tests: theme resolution, switching and saving, blocked storage, a check that no on-screen text uses banned jargon, and an automated accessibility check. Lint, typecheck, tests, build and `npm audit` all passed. One deviation: the accessibility test checks `violations` directly, not the jest-axe matcher.

**7. CI workflow created.** Added `.github/workflows/ci.yml`. UI jobs run lint, types, tests, accessibility, build and audit, plus a secret scan. Rust jobs (fmt, clippy, test, `cargo deny`) and the installer build switch on automatically once `Cargo.toml` and the Tauri config exist. It has not run on GitHub yet.

**8. Rust setup guidance given.** Gave you step-by-step instructions to install the C++ build tools and Rust on Windows. They covered the toolchain, clippy, rustfmt, `cargo-audit` and `cargo-deny`, plus how to verify each one. You installed Rust on your side and reported errors setting up the toolchain.

## 2026-11-06

**9. Install failure diagnosed.** Found that `rustup` and the C++ build tools were installed correctly, but no toolchain was downloaded. The install failed with "not enough space on the disk". The C: drive had only 0.13 GB free out of about 476 GB, so Rust could not unpack.

**10. Disk space freed.** Checked two large Temp folders, `buuz5wps` (4.1 GB) and `0pt4n1tm` (3.0 GB). They held old Visual Studio and .NET installer caches and had not been touched for days. With your approval I deleted both folders. Free space on C: rose to 7.55 GB.

**11. Rust toolchain installed.** Reran the install now that space was free. Rust 1.99.1 was installed with cargo 1.99.1, clippy 0.1.99 and rustfmt 1.10.0, all on the 64-bit MSVC target. I confirmed each version by running it. About 6 GB remained free afterwards.

**12. Linker error traced.** The security tools failed to build with `cannot open file 'msvcrt.lib'`. Rust picks the newest Visual Studio, VS 2026 Community, whose x64 libraries are missing, probably from an install interrupted by the full disk. Your VS 2022 Build Tools and Windows SDK are intact.

**13. Security tools installed.** Built `cargo-audit` 0.22.2 and `cargo-deny` 0.20.2 using the VS 2022 Build Tools environment. The build took about 13 minutes. Both tools ran with `--version` afterwards. These will scan dependencies for known vulnerabilities and licence problems before each commit and in CI.

**14. Build helper script added.** Created `scripts/dev-shell.cmd`, which loads the working VS 2022 environment before running a command, for example `scripts\dev-shell.cmd cargo build`. It is a workaround. The permanent fix is repairing VS 2026 Community in Visual Studio Installer, after which the script can be deleted.

## 2026-11-07

**15. Progress log started.** Created this file, `docs/progress-highlights-for-me.md`, at your request. It records past work from memory plus all future changes, with a date on each entry and about 50 words per entry. From now on I add an entry for each addition, update or change.

**16. Cargo workspace scaffolded.** Created the root `Cargo.toml` with seven crates: sp-core, sp-display, sp-capture, sp-media, sp-remote, sp-audio and sp-signal-server. Each has a short purpose note. Added `rust-toolchain.toml`, `rustfmt.toml` and `clippy.toml`. Release builds keep panic unwinding so the output window can survive a crash.

**17. Strict safety lints enforced.** Workspace lints now deny `unwrap`, `expect`, `panic`, unchecked indexing, `todo` and `dbg!` in normal code. Tests are exempt. This enforces the AGENTS.md rule against crashes on user input, files, devices and network paths. Missing documentation raises a warning.

**18. First real code: UserError.** Added `UserError` to sp-core, holding a stable code, a plain-language message and a suggested fix. It is the error type every future command will return. One test checks that its text reads "message, then fix". sp-core still depends on nothing else.

**19. Dependency policy added.** Created `deny.toml`: only allow-listed licences (MIT, Apache-2.0, BSD and similar), crates.io as the only source, no yanked crates, and no wildcard versions for registry crates. Our own workspace paths are exempt. Our crates stay unlicensed until the pricing question is decided.

**20. Rust checks all passing.** Ran fmt, clippy with warnings denied, tests (1 passed), `cargo audit` (no vulnerabilities across 7 crates) and `cargo deny` (advisories, bans, licences, sources all ok). rustfmt reformatted one line, and a deny setting was adjusted for path crates. Free space rose to about 10.9 GB.

## 2026-11-12

**21. Tauri app shell added.** Created `app/src-tauri` with the Tauri 2 setup: build script, config, entry point and library. Added it to the Cargo workspace and installed the Tauri tools in the UI project. The window is named SimplyProject, with a minimum size of 800 by 560. A startup failure exits cleanly instead of crashing.

**22. App locked down.** The shell has a strict content security policy and no global scripting access. Its permissions file gives the control window only the basic core permissions. The output window will get its own, smaller set in Step 2. The installer targets Windows only and downloads the web-view helper if it is missing.

**23. App icons made.** Windows needs an icon to build, so I drew a simple green circle with an eye as a placeholder and generated every icon size from it. I deleted the Android and iOS icons the tool also made, since this app is Windows-only. Replace the placeholder with real artwork before release.

**24. Checks passed with Tauri.** Fmt, clippy with warnings denied, and tests all passed. `cargo audit` found no vulnerabilities across 427 crates, only two warnings in Linux-only libraries. `cargo deny` passed. One lint fix was needed: two files lacked a short top comment. The build script folder bug was also fixed.

**25. Installer built and app launched.** Built a debug installer, `SimplyProject_0.0.0_x64-setup.exe` (2.4 MB). I ran the app: a window titled SimplyProject opened, used about 28 MB, and closed cleanly. I did not click inside the window, so the theme switch there is untested. CI's installer step was corrected to match.

## 2026-11-22

**26. Git rules recorded.** Added the GitHub repo URL and your workflow to `AGENTS.md` (section 4.3.1): every push uses its own typed branch such as `feat/...` or `chore/...`, then a PR into `dev`. No Claude co-author lines, no secrets in git, and you review or tell me to.

**27. Secret protection tightened.** Expanded `.gitignore` to ignore env files, keys, certificates, signing material and local Claude settings. I checked the project for such files before committing and found none. The first commit and push follow, so you can watch the pipeline run.

## 2026-11-23

**28. First pipeline run.** The first CI run failed because `ci.yml` used `hashFiles` in job conditions, which GitHub rejects. I removed them and the jobs ran. UI, Rust checks, secret scan and installer build passed. Only the dependency check failed, on a new advisory against the `url` crate.

**29. Vulnerable dependency fixed.** Advisory RUSTSEC-2026-0592 hit `url` 2.5.8. I updated it to 2.5.9 (and `form_urlencoded` 1.2.3). Locally `cargo deny` now passes, and fmt, clippy and tests are green. Pushed to PR #1 to re-run CI.

## 2026-11-24

**30. Resolver built (M1 start).** Added the resolver to `sp-core`: `Item`, `CueStack` (add, remove, eye, reorder) and `resolve`, which picks blackout, frozen frame, first ready item, or standby. 28 tests pass, including property tests. CI passed on the earlier PR; this work is on branch `feat/sl-6-resolver-in-sp-core`.

**31. Second advisory fixed.** `cargo deny` and `cargo audit` flagged `cookie` 0.18.2 (RUSTSEC-2026-0594). It cannot affect Windows, but CI would fail. Updated to 0.18.3. Also recorded one decision in ADR 0001: freeze with no frame to hold falls back to the normal rule.

## Open items

- Theme switch inside the real app window needs a quick manual check (run `npm run tauri dev` from `app/ui`).
- PR #1 (`chore/project-scaffold` into `dev`) is open and waiting for CI to go green and for your review.
- Visual Studio 2026 still needs repair so `dev-shell.cmd` can go.
- Replace the placeholder icon before release.
- Dev-only npm warning: `minimatch` inside ESLint. It does not ship in the app, and the fix is a breaking ESLint change.
- Disk: about 12 GB free.
