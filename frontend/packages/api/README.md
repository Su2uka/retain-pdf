# @retainpdf/api

Typed API clients using generated wire DTOs from `@retainpdf/contracts`.

- `jobs.ts` → `fetchJobPayload`, `fetchJobList`, `fetchJobsByIds` (batch live state for library cards via `GET jobs?job_ids=`)
- `jobs-actions.ts` → `deleteJob` (`DELETE jobs/:id`), cancel / resume / retry-stage
- `types.ts` → generated Job/library DTO re-exports plus the generic `ApiResponse` envelope

Build: `npm --prefix frontend/packages/api run build` → `dist/`
Usage: `import { fetchJobsByIds } from "@retainpdf/api/jobs"`

The frozen shelf API (`library/books`) is no longer called; `contracts/library-books` is only used for the delete result shape `LibraryDeleteResultView`.

Browser-aware runtime configuration helpers are public at `@retainpdf/api/runtime`.
The old `@retainpdf/api/internal/runtime` entry remains available only as a
compatibility path for existing consumers.
