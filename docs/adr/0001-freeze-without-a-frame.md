# 0001. Freeze with no frame to hold falls back to the normal rule

## Context

PRD section 4 says `freeze -> last_frame`. It does not say what happens when freeze is on
but there is no last frame yet (nothing has been shown since launch). The audience must
never see an error screen or a desktop (AGENTS.md section 3).

## Decision

If freeze is on and the caller has no frame to hold, `resolve` ignores freeze and applies
the normal rule (first visible, ready item, else standby). Blackout still beats freeze.

## Consequences

- The resolver stays total: every input gives a showable result.
- Freeze can never produce an empty or broken output.
- Once a frame exists, freeze holds it, even if the list changes underneath.
- The PRD wording is unchanged; this only fills a gap. Revisit if you want freeze to show
  standby instead.
