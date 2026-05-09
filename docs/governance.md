# Governance

## Commit Policy

- Each major implementation phase should end in one verification-oriented git commit.
- The commit message should summarize why the phase changed the project, not just which files changed.
- Verification notes should reference the commands or manual checks used to accept the phase.

## Evidence Policy

- Do not archive the OpenSpec change until build, test, and manual validation evidence is collected.
- Keep command output and manual QA notes in a single acceptance evidence log.
- Manual validations that cannot be automated yet must be called out explicitly before release.
