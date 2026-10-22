// 生成文件，不要手改。来源：contracts/job-events.v2.schema.json 的 endpoints。
// 重新生成：npm --prefix frontend/packages/api run generate:client

import type {
  JobEventListView,
  ListJobEventsQuery,
} from "@retainpdf/contracts/job-events";
import { encodePathParam, request, type RequestOptions } from "../request.js";

/** GET /api/v1/jobs/:job_id/progress/events */
export function listJobEvents(
  apiPrefix: string | undefined,
  args: { job_id: string; query?: ListJobEventsQuery },
  options?: RequestOptions,
): Promise<JobEventListView> {
  return request<JobEventListView>({ apiPrefix, method: "GET", path: `jobs/${encodePathParam(args.job_id)}/progress/events`, query: args.query, options });
}
