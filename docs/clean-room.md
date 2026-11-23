# Clean-room policy

Written for: anyone implementing or reviewing an algorithm in this repository.

Goal: every algorithm here is implemented from published papers or open specifications, and
never from GPL, LGPL, AGPL or proprietary source code. This keeps the MIT/Apache licensing
clean.

## Rules

1. **Sources allowed:** peer-reviewed papers, textbooks, public standards, and public
   documentation of datasets.
2. **Sources forbidden:** source code of any GPL/LGPL/AGPL/SSPL/EUPL project, any proprietary
   implementation, and any reference implementation whose license would restrict reuse
   (including Apache-2.0 reference code whose text we would then be copying). Reading such code
   to understand a bug report or to test behavior is also disallowed for the implementing
   contributor.
3. **Write from the description.** Implement from equations and pseudocode in the cited source.
   Name variables after the paper's notation where practical.
4. **Record the reference.** Each algorithm crate has `docs/clean-room-notes/<crate>.md` listing
   the references, the equations or pseudocode implemented, and design decisions.
5. **Record provenance.** Each algorithm crate has `docs/provenance/<crate>.md` built from
   [provenance/TEMPLATE.md](provenance/TEMPLATE.md).
6. **Be honest about verification.** If a reference was worked from memory or from a secondary
   summary, the notes say so, and a maintainer must re-read the original before release.
7. **Datasets** are fetched only on demand, never committed, and their terms are checked before
   any workflow is documented. See [datasets.md](datasets.md).

## Review checklist (for every algorithm PR)

- [ ] Clean-room notes and provenance entry exist and are current.
- [ ] No code was copied from a non-permissive source (check the PR description for the source
      used).
- [ ] References cited with authors, year, title and venue.
- [ ] Any "worked from memory" statement has been replaced by a re-read of the original, or the
      limitation is still listed under "Verification status".
