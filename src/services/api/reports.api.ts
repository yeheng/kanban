import { useQuery } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";

/** A report catalog entry from the backend. */
export interface ReportCatalogEntry {
  kind: string;
  title: string;
  description: string;
  formats: string[];
  accepts_project_id: boolean;
  mvp: boolean;
}

export const reportKinds = ["ResourceUtilization", "TeamUtilization", "ProjectBurn", "AiDecisions", "Cost"] as const;
export type ReportKind = typeof reportKinds[number];
export type ReportFormat = "csv" | "xlsx" | "pdf";

export function useGetReportCatalogQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<ReportCatalogEntry[]>({
    queryKey: ["report-catalog"],
    queryFn: () => apiFetch<ReportCatalogEntry[]>("/api/reports/catalog"),
  });
}

/** Trigger a browser file download from a Blob. */
function triggerDownload(blob: Blob, filename: string): void {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  // Defer revocation: a.click() only queues the download as a separate task.
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

/** Fetch a report file and trigger a browser download. Imperative (not a mutation) — has download side effect. */
export async function exportReport(
  apiFetch: ReturnType<typeof useApiFetch>["apiFetch"],
  kind: ReportKind,
  projectId: number | null,
  start: string,
  end: string,
  format: ReportFormat,
): Promise<boolean> {
  const params = new URLSearchParams({ start, end, format });
  if (projectId != null) params.set("project_id", String(projectId));
  // 注：不带显式 <Blob> 类型实参 —— ofetch 的 R(responseType) 必须由 options 推断；
  // 显式传 <Blob> 会让 TS 跳过推断、R 回落到默认 "json"，从而拒绝 "blob"。
  // R="blob" 时返回类型经 MappedResponseType 自动解析为 Blob。
  const blob = await apiFetch(`/api/reports/${kind}?${params}`, { responseType: "blob" });
  triggerDownload(blob, `${kind}.${format}`);
  return true;
}

/** Fetch a workforce snapshot JSON and trigger download. Imperative. */
export async function exportSnapshot(
  apiFetch: ReturnType<typeof useApiFetch>["apiFetch"],
  start: string,
  end: string,
): Promise<boolean> {
  const params = new URLSearchParams({ start, end });
  const blob = await apiFetch(`/api/reports/snapshot?${params}`, { responseType: "blob" });
  triggerDownload(blob, "workforce-snapshot.json");
  return true;
}
