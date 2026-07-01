# 阶段3a：数据层基建 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 建立 `src/services/api/` 12 个域文件（vue-query composable）+ ofetch 客户端 + 响应类型 + QueryClient 调优，与旧 `api/index.ts` 双轨并存，不改任何 page/store，应用行为零变化。

**Architecture:** 新 `services/api/<domain>.api.ts` 每个导出 `useXxxQuery()`/`useXxxMutation()` composable（基于 vue-query），底层用 `services/fetch.ts` 的 `useApiFetch()`（ofetch，baseURL 来自 `@/utils/env`）。queryKey 用 `["<domain>"]` 前缀，mutation onSuccess 调 `invalidateQueries`。旧 `api/index.ts` + 13 store + refresh 总线原样保留（3b 迁 page、3c 删旧）。

**Tech Stack:** ofetch + @tanstack/vue-query（阶段0+1 已装并注册）+ Pinia（旧，不动）。

**Spec:** `docs/superpowers/specs/2026-07-01-shadcn-admin-stage3a-data-layer-design.md`

**关键前置事实（已验证）**：
- ofetch + vue-query 已装（阶段0+1），`VueQueryPlugin` + `QueryClient`(staleTime 5min) 已注册。
- `@/utils/env` 已建（阶段0+1），`env.VITE_API_BASE` 返回相对路径（dev 下空串→同源→vite proxy）。
- `SkillReq` 类型只在 `api/index.ts` 内部用（无外部 importer）→ 新 `tasks.api.ts` 自定义即可。
- report 类型（reportKinds/ReportKind/ReportFormat/ReportCatalogEntry）只被 `reports/index.vue` 导入（本期不动）→ 新 `reports.api.ts` 自定义。
- 旧 `api/index.ts` 的方法路径/body/HTTP 方法是权威来源（见 plan 内每个 task 的代码）。

---

## File Structure

**Create:**
- `src/services/fetch.ts` — ofetch 客户端 `useApiFetch()`。
- `src/services/types/response.type.ts` — `IResponse<T>`、分页请求类型。
- `src/services/api/projects.api.ts` — 5 方法（list/create/update/setStatus/delete）。
- `src/services/api/catalog.api.ts` — 4 方法（listSkills/ensureSkill/listTags/ensureTag）。
- `src/services/api/tasks.api.ts` — 7 方法 + `SkillReq` 类型。
- `src/services/api/resources.api.ts` — 8 方法。
- `src/services/api/workload.api.ts` — 4 方法（resourceSummary/teamSummary/overloads/projectBurn）。
- `src/services/api/config.api.ts` — 4 方法（getThresholds/getUnitConfig/getSettings/updateSettings）。
- `src/services/api/allocations.api.ts` — 4 方法（create/delete/list/update）。
- `src/services/api/calendar.api.ts` — 8 方法。
- `src/services/api/teams.api.ts` — 8 方法。
- `src/services/api/gantt.api.ts` — 4 方法。
- `src/services/api/optimization.api.ts` — 4 方法。
- `src/services/api/reports.api.ts` — getReportCatalog 查询 + exportReport/exportSnapshot 命令式 helper + report 类型。

**Modify:**
- `src/plugins/tanstack-vue-query/setup.ts` — QueryClient 加 `refetchOnWindowFocus: false`、`retry: 1`。

**Do NOT touch:** `src/api/index.ts`、`src/stores/**`、`src/pages/**`、`src/components/**`、`src/types.ts`。

---

## Task 1: 基建（fetch.ts + response.type.ts + QueryClient 调优）

**Files:**
- Create: `src/services/fetch.ts`
- Create: `src/services/types/response.type.ts`
- Modify: `src/plugins/tanstack-vue-query/setup.ts`

- [ ] **Step 1: 创建 services/fetch.ts**

Create `src/services/fetch.ts`:
```ts
import { ofetch } from "ofetch";
import env from "@/utils/env";

/**
 * ofetch 客户端。baseURL 来自 @/utils/env（相对路径，dev 走 vite proxy /api）。
 * 拦截器：onRequest 预留 auth header（kanban 无 auth，本期空）；
 * onResponseError 统一打日志。
 */
export function useApiFetch() {
  return {
    apiFetch: ofetch.create({
      baseURL: env.VITE_API_BASE,
      onRequest(_ctx) {
        // 预留：auth header 注入（kanban 当前无 auth）
      },
      onResponseError(_ctx) {
        console.error("[api] request failed:", _ctx.error);
      },
    }),
  };
}
```

- [ ] **Step 2: 创建 services/types/response.type.ts**

Create `src/services/types/response.type.ts`:
```ts
/**
 * 后端响应封装类型（对齐模板）。
 * kanban 后端目前直返裸数据（如 Project[]），故 IResponse 本期为可选工具类型，
 * composable 暂不强制用它包裹。将来后端统一响应格式时启用。
 */
export interface IResponse<T, E = Record<string, unknown>> {
  data: T;
  extra: E;
  code: number;
  message: string;
  success: boolean;
}

export interface IPaginationRequestQuery {
  page?: number;
  pageSize?: number;
}

export type IRequestQuery<T extends Record<string, unknown>> = {
  page?: number;
  pageSize?: number;
} & {
  [K in keyof T]?: T[K];
};
```

- [ ] **Step 3: 调优 QueryClient（plugins/tanstack-vue-query/setup.ts）**

Read current `src/plugins/tanstack-vue-query/setup.ts`，把 `defaultOptions.queries` 加两行。修改后完整文件:
```ts
import type { App } from "vue";
import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
// NOTE: @tanstack/vue-query-devtools v6 exports VueQueryDevtools as a Vue
// *component* (not an installable plugin), so it can no longer be registered
// via app.use(). Devtools rendering is deferred to App.vue at wire-up (Task 5).

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5,
      refetchOnWindowFocus: false,
      retry: 1,
    },
  },
});

export function setupTanstackVueQuery(app: App) {
  app.use(VueQueryPlugin, { queryClient });
}
```

- [ ] **Step 4: typecheck**

Run:
```bash
pnpm exec vue-tsc -b
```
Expected: exit 0。

- [ ] **Step 5: Commit**

```bash
git add src/services/fetch.ts src/services/types/response.type.ts src/plugins/tanstack-vue-query/setup.ts
git commit -m "feat(services): 阶段3a 数据层基建（ofetch 客户端 + 响应类型 + QueryClient 调优）

- services/fetch.ts: useApiFetch()（ofetch，baseURL 来自 @/utils/env）
- services/types/response.type.ts: IResponse + 分页请求类型
- QueryClient: 加 refetchOnWindowFocus:false + retry:1
- 旧 api/index.ts 与 store 不动（双轨期开始）"
```

---

## Task 2: projects.api.ts

**Files:** Create `src/services/api/projects.api.ts`

- [ ] **Step 1: 创建文件**

Create `src/services/api/projects.api.ts`:
```ts
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { Project } from "@/types";

export function useListProjectsQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<Project[]>({
    queryKey: ["projects"],
    queryFn: () => apiFetch<Project[]>("/api/projects"),
  });
}

export function useCreateProjectMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<number, Error, { name: string; priority: number; budgetPd: number }>({
    mutationFn: (args) =>
      apiFetch<number>("/api/projects", {
        method: "POST",
        body: { name: args.name, priority: args.priority, budget_pd: args.budgetPd },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
  });
}

export function useUpdateProjectMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, {
    id: number;
    name: string; priority: number; budgetPd: number;
    description?: string | null; start?: string | null; end?: string | null;
  }>({
    mutationFn: (args) =>
      apiFetch<void>(`/api/projects/${args.id}`, {
        method: "PATCH",
        body: {
          name: args.name, description: args.description ?? null,
          start: args.start ?? null, end: args.end ?? null,
          priority: args.priority, budget_pd: args.budgetPd,
        },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
  });
}

export function useSetProjectStatusMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { id: number; status: string }>({
    mutationFn: (args) =>
      apiFetch<void>(`/api/projects/${args.id}/status`, { method: "PATCH", body: { status: args.status } }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
  });
}

export function useDeleteProjectMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, number>({
    mutationFn: (id) => apiFetch<void>(`/api/projects/${id}`, { method: "DELETE" }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
  });
}
```

- [ ] **Step 2: typecheck + commit**

```bash
pnpm exec vue-tsc -b && git add src/services/api/projects.api.ts && git commit -m "feat(services/api): projects.api.ts（5 composable）"
```
Expected: tsc exit 0。

---

## Task 3: catalog.api.ts

**Files:** Create `src/services/api/catalog.api.ts`

- [ ] **Step 1: 创建文件**

Create `src/services/api/catalog.api.ts`:
```ts
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { Skill, Tag } from "@/types";

export function useListSkillsQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<Skill[]>({
    queryKey: ["skills"],
    queryFn: () => apiFetch<Skill[]>("/api/skills"),
  });
}

export function useEnsureSkillMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<number, Error, string>({
    mutationFn: (name) => apiFetch<number>("/api/skills", { method: "POST", body: { name } }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["skills"] });
    },
  });
}

export function useListTagsQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<Tag[]>({
    queryKey: ["tags"],
    queryFn: () => apiFetch<Tag[]>("/api/tags"),
  });
}

export function useEnsureTagMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<number, Error, { name: string; color: string | null }>({
    mutationFn: (args) => apiFetch<number>("/api/tags", { method: "POST", body: { name: args.name, color: args.color } }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["tags"] });
    },
  });
}
```

- [ ] **Step 2: typecheck + commit**

```bash
pnpm exec vue-tsc -b && git add src/services/api/catalog.api.ts && git commit -m "feat(services/api): catalog.api.ts（skills/tags 4 composable）"
```

---

## Task 4: tasks.api.ts

**Files:** Create `src/services/api/tasks.api.ts`

- [ ] **Step 1: 创建文件（含 SkillReq 类型）**

Create `src/services/api/tasks.api.ts`:
```ts
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { KanbanTask, Task, TaskStatus } from "@/types";

/** [skillId, proficiency, required, weight] — 与旧 api/index.ts 的 SkillReq 一致。 */
export type SkillReq = [number, number, boolean, number];

export function useListTasksQuery(projectId: number) {
  const { apiFetch } = useApiFetch();
  return useQuery<Task[]>({
    queryKey: ["tasks", projectId],
    queryFn: () => apiFetch<Task[]>(`/api/projects/${projectId}/tasks`),
  });
}

export function useKanbanTasksQuery(projectId: number) {
  const { apiFetch } = useApiFetch();
  return useQuery<KanbanTask[]>({
    queryKey: ["kanban", projectId],
    queryFn: () => apiFetch<KanbanTask[]>(`/api/projects/${projectId}/kanban`),
  });
}

export function useCreateTaskMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<number, Error, {
    projectId: number; title: string; estimatePd: number;
    start: string | null; end: string | null;
    skillReqs: SkillReq[]; tagIds: number[];
    description?: string | null;
    isLongTerm?: boolean; parentTaskId?: number | null; segmentKind?: string | null;
  }>({
    mutationFn: (args) =>
      apiFetch<number>("/api/tasks", {
        method: "POST",
        body: {
          project_id: args.projectId,
          title: args.title,
          estimate_pd: args.estimatePd,
          start: args.start,
          end: args.end,
          skill_reqs: args.skillReqs,
          tag_ids: args.tagIds,
          description: args.description ?? null,
          is_long_term: args.isLongTerm ?? false,
          parent_task_id: args.parentTaskId ?? null,
          segment_kind: args.segmentKind ?? null,
          sort_order: 0,
        },
      }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ["tasks", variables.projectId] });
      queryClient.invalidateQueries({ queryKey: ["kanban", variables.projectId] });
    },
  });
}

export function useUpdateTaskMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, {
    id: number; projectId?: number;
    title: string; estimatePd: number;
    start: string | null; end: string | null;
    description?: string | null;
    isLongTerm?: boolean; parentTaskId?: number | null; segmentKind?: string | null;
  }>({
    mutationFn: (args) =>
      apiFetch<void>(`/api/tasks/${args.id}`, {
        method: "PATCH",
        body: {
          title: args.title,
          description: args.description ?? null,
          estimate_pd: args.estimatePd,
          start: args.start,
          end: args.end,
          is_long_term: args.isLongTerm ?? false,
          parent_task_id: args.parentTaskId ?? null,
          segment_kind: args.segmentKind ?? null,
        },
      }),
    onSuccess: (_data, variables) => {
      if (variables.projectId != null) {
        queryClient.invalidateQueries({ queryKey: ["tasks", variables.projectId] });
        queryClient.invalidateQueries({ queryKey: ["kanban", variables.projectId] });
      }
    },
  });
}

export function useDeleteTaskMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { id: number; projectId?: number }>({
    mutationFn: (args) => apiFetch<void>(`/api/tasks/${args.id}`, { method: "DELETE" }),
    onSuccess: (_data, variables) => {
      if (variables.projectId != null) {
        queryClient.invalidateQueries({ queryKey: ["tasks", variables.projectId] });
        queryClient.invalidateQueries({ queryKey: ["kanban", variables.projectId] });
      }
    },
  });
}

export function useSetTaskStatusMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { id: number; status: TaskStatus; projectId?: number }>({
    mutationFn: (args) =>
      apiFetch<void>(`/api/tasks/${args.id}/status`, { method: "PATCH", body: { status: args.status } }),
    onSuccess: (_data, variables) => {
      if (variables.projectId != null) {
        queryClient.invalidateQueries({ queryKey: ["tasks", variables.projectId] });
        queryClient.invalidateQueries({ queryKey: ["kanban", variables.projectId] });
      }
    },
  });
}

export function useAddDependencyMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { taskId: number; predecessorId: number; lagDays?: number; projectId?: number }>({
    mutationFn: (args) =>
      apiFetch<void>(`/api/tasks/${args.taskId}/dependencies`, {
        method: "POST",
        body: { predecessor_id: args.predecessorId, lag_days: args.lagDays ?? 0, dep_type: "finish_to_start" },
      }),
    onSuccess: (_data, variables) => {
      if (variables.projectId != null) {
        queryClient.invalidateQueries({ queryKey: ["dependencies", variables.projectId] });
      }
    },
  });
}
```

注意：mutation 入参带可选 `projectId`，用于 onSuccess 精确失效（旧 api 无此参数因 store 自己 reload；新 composable 靠 invalidate）。`projectId` 可选——3b 接入时 page 会传。

- [ ] **Step 2: typecheck + commit**

```bash
pnpm exec vue-tsc -b && git add src/services/api/tasks.api.ts && git commit -m "feat(services/api): tasks.api.ts（7 composable + SkillReq 类型）"
```

---

## Task 5: resources.api.ts

**Files:** Create `src/services/api/resources.api.ts`

- [ ] **Step 1: 创建文件**

Create `src/services/api/resources.api.ts`:
```ts
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { Resource, ResourceSkill, ResourceTag } from "@/types";

export function useListResourcesQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<Resource[]>({
    queryKey: ["resources"],
    queryFn: () => apiFetch<Resource[]>("/api/resources"),
  });
}

export function useCreateResourceMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<number, Error, { name: string; email: string | null }>({
    mutationFn: (args) => apiFetch<number>("/api/resources", { method: "POST", body: { name: args.name, email: args.email } }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["resources"] });
    },
  });
}

export function useUpdateResourceMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, {
    id: number; name: string; email: string | null;
    availableFrom?: string | null; availableTo?: string | null;
    dailyCapacityPd?: number | null; dailyRatePd?: number | null;
  }>({
    mutationFn: (args) =>
      apiFetch<void>(`/api/resources/${args.id}`, {
        method: "PATCH",
        body: {
          name: args.name, email: args.email,
          available_from: args.availableFrom ?? null, available_to: args.availableTo ?? null,
          daily_capacity_pd: args.dailyCapacityPd ?? null, daily_rate_pd: args.dailyRatePd ?? null,
        },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["resources"] });
    },
  });
}

export function useDeleteResourceMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, number>({
    mutationFn: (id) => apiFetch<void>(`/api/resources/${id}`, { method: "DELETE" }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["resources"] });
    },
  });
}

export function useGetResourceSkillsQuery(id: number) {
  const { apiFetch } = useApiFetch();
  return useQuery<ResourceSkill[]>({
    queryKey: ["resource-skills", id],
    queryFn: () => apiFetch<ResourceSkill[]>(`/api/resources/${id}/skills`),
  });
}

export function useSetResourceSkillsMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { id: number; skills: [number, number][] }>({
    mutationFn: (args) => apiFetch<void>(`/api/resources/${args.id}/skills`, { method: "PUT", body: { skills: args.skills } }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ["resource-skills", variables.id] });
      queryClient.invalidateQueries({ queryKey: ["resources"] });
    },
  });
}

export function useGetResourceTagsQuery(id: number) {
  const { apiFetch } = useApiFetch();
  return useQuery<ResourceTag[]>({
    queryKey: ["resource-tags", id],
    queryFn: () => apiFetch<ResourceTag[]>(`/api/resources/${id}/tags`),
  });
}

export function useSetResourceTagsMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { id: number; tagIds: number[] }>({
    mutationFn: (args) => apiFetch<void>(`/api/resources/${args.id}/tags`, { method: "PUT", body: { tag_ids: args.tagIds } }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ["resource-tags", variables.id] });
      queryClient.invalidateQueries({ queryKey: ["resources"] });
    },
  });
}
```

- [ ] **Step 2: typecheck + commit**

```bash
pnpm exec vue-tsc -b && git add src/services/api/resources.api.ts && git commit -m "feat(services/api): resources.api.ts（8 composable）"
```

---

## Task 6: workload.api.ts

**Files:** Create `src/services/api/workload.api.ts`

- [ ] **Step 1: 创建文件**

Create `src/services/api/workload.api.ts`:
```ts
import { useQuery } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { ProjectBurn, ResourceSummary, TeamSummary } from "@/types";

export function useResourceSummaryQuery(resourceId: number, start: string, end: string) {
  const { apiFetch } = useApiFetch();
  return useQuery<ResourceSummary>({
    queryKey: ["workload-resource", resourceId, start, end],
    queryFn: () =>
      apiFetch<ResourceSummary>(
        `/api/workload/resources/${resourceId}?start=${encodeURIComponent(start)}&end=${encodeURIComponent(end)}`,
      ),
  });
}

export function useTeamSummaryQuery(teamId: number, start: string, end: string) {
  const { apiFetch } = useApiFetch();
  return useQuery<TeamSummary>({
    queryKey: ["workload-team", teamId, start, end],
    queryFn: () =>
      apiFetch<TeamSummary>(
        `/api/workload/teams/${teamId}?start=${encodeURIComponent(start)}&end=${encodeURIComponent(end)}`,
      ),
  });
}

export function useOverloadsQuery(start: string, end: string) {
  const { apiFetch } = useApiFetch();
  return useQuery<ResourceSummary[]>({
    queryKey: ["workload-overloads", start, end],
    queryFn: () =>
      apiFetch<ResourceSummary[]>(
        `/api/workload/overloads?start=${encodeURIComponent(start)}&end=${encodeURIComponent(end)}`,
      ),
  });
}

export function useProjectBurnQuery(projectId: number) {
  const { apiFetch } = useApiFetch();
  return useQuery<ProjectBurn>({
    queryKey: ["workload-burn", projectId],
    queryFn: () => apiFetch<ProjectBurn>(`/api/projects/${projectId}/burn`),
  });
}
```

注意：workload 域不含 occupancy（它在 gantt 域），故只 import `ProjectBurn, ResourceSummary, TeamSummary`。

- [ ] **Step 2: typecheck + commit**

```bash
pnpm exec vue-tsc -b && git add src/services/api/workload.api.ts && git commit -m "feat(services/api): workload.api.ts（4 query）"
```

---

## Task 7: config.api.ts

**Files:** Create `src/services/api/config.api.ts`

- [ ] **Step 1: 创建文件**

Create `src/services/api/config.api.ts`:
```ts
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { Settings, Thresholds } from "@/types";

export function useGetThresholdsQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<Thresholds>({
    queryKey: ["thresholds"],
    queryFn: () => apiFetch<Thresholds>("/api/thresholds"),
  });
}

export function useGetUnitConfigQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<{ pd_hours: number; pm_workdays: number }>({
    queryKey: ["unit-config"],
    queryFn: () => apiFetch<{ pd_hours: number; pm_workdays: number }>("/api/config/units"),
  });
}

export function useGetSettingsQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<Settings>({
    queryKey: ["settings"],
    queryFn: () => apiFetch<Settings>("/api/settings"),
  });
}

export function useUpdateSettingsMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, Settings>({
    mutationFn: (settings) => apiFetch<void>("/api/settings", { method: "PUT", body: settings }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["settings"] });
    },
  });
}
```

- [ ] **Step 2: typecheck + commit**

```bash
pnpm exec vue-tsc -b && git add src/services/api/config.api.ts && git commit -m "feat(services/api): config.api.ts（thresholds/units/settings 4 composable）"
```

---

## Task 8: allocations.api.ts

**Files:** Create `src/services/api/allocations.api.ts`

- [ ] **Step 1: 创建文件**

Create `src/services/api/allocations.api.ts`:
```ts
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { AllocationView } from "@/types";

export function useListAllocationsQuery(projectId: number) {
  const { apiFetch } = useApiFetch();
  return useQuery<AllocationView[]>({
    queryKey: ["allocations", projectId],
    queryFn: () => apiFetch<AllocationView[]>(`/api/projects/${projectId}/allocations`),
  });
}

export function useCreateAllocationMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<number, Error, {
    resourceId: number; taskId: number; start: string; end: string; percent: number; projectId?: number;
  }>({
    mutationFn: (args) =>
      apiFetch<number>("/api/allocations", {
        method: "POST",
        body: { resource_id: args.resourceId, task_id: args.taskId, start: args.start, end: args.end, percent: args.percent },
      }),
    onSuccess: (_data, variables) => {
      // allocation 写入失效所有 allocation 衍生视图（对应旧 refresh bump 的多 scope）
      queryClient.invalidateQueries({ queryKey: ["allocations"] });
      queryClient.invalidateQueries({ queryKey: ["workload"] });
      queryClient.invalidateQueries({ queryKey: ["gantt"] });
      queryClient.invalidateQueries({ queryKey: ["kanban"] });
      queryClient.invalidateQueries({ queryKey: ["calendar"] });
    },
  });
}

export function useUpdateAllocationMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { id: number; start: string; end: string; percent: number; projectId?: number }>({
    mutationFn: (args) =>
      apiFetch<void>(`/api/allocations/${args.id}`, { method: "PUT", body: { start: args.start, end: args.end, percent: args.percent } }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["allocations"] });
      queryClient.invalidateQueries({ queryKey: ["workload"] });
      queryClient.invalidateQueries({ queryKey: ["gantt"] });
      queryClient.invalidateQueries({ queryKey: ["kanban"] });
      queryClient.invalidateQueries({ queryKey: ["calendar"] });
    },
  });
}

export function useDeleteAllocationMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { id: number; projectId?: number }>({
    mutationFn: (args) => apiFetch<void>(`/api/allocations/${args.id}`, { method: "DELETE" }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["allocations"] });
      queryClient.invalidateQueries({ queryKey: ["workload"] });
      queryClient.invalidateQueries({ queryKey: ["gantt"] });
      queryClient.invalidateQueries({ queryKey: ["kanban"] });
      queryClient.invalidateQueries({ queryKey: ["calendar"] });
    },
  });
}
```

注意：onSuccess 用 domain 前缀失效（`invalidateQueries({ queryKey: ["allocations"] })` 失效所有以 allocations 开头的 key），对应旧 `bump("allocations","workload","gantt","kanban","calendar")`。

- [ ] **Step 2: typecheck + commit**

```bash
pnpm exec vue-tsc -b && git add src/services/api/allocations.api.ts && git commit -m "feat(services/api): allocations.api.ts（4 composable，跨域失效对应旧 refresh bump）"
```

---

## Task 9: calendar.api.ts

**Files:** Create `src/services/api/calendar.api.ts`

- [ ] **Step 1: 创建文件**

Create `src/services/api/calendar.api.ts`:
```ts
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { Holiday, TimeOff, WeekTemplate } from "@/types";

export function useListWorkWeeksQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<WeekTemplate[]>({
    queryKey: ["work-weeks"],
    queryFn: () => apiFetch<WeekTemplate[]>("/api/calendar/work-week"),
  });
}

export function useSetGlobalWorkWeekMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, number[]>({
    mutationFn: (week) => apiFetch<void>("/api/calendar/work-week", { method: "POST", body: { week } }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["work-weeks"] });
      queryClient.invalidateQueries({ queryKey: ["calendar"] });
    },
  });
}

export function useListHolidaysQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<Holiday[]>({
    queryKey: ["holidays"],
    queryFn: () => apiFetch<Holiday[]>("/api/calendar/holidays"),
  });
}

export function useAddHolidayMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<number, Error, { projectId: number | null; day: string; fraction: number | null; name: string | null }>({
    mutationFn: (args) =>
      apiFetch<number>("/api/calendar/holidays", {
        method: "POST",
        body: { project_id: args.projectId, day: args.day, fraction: args.fraction, name: args.name },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["holidays"] });
      queryClient.invalidateQueries({ queryKey: ["calendar"] });
    },
  });
}

export function useDeleteHolidayMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, number>({
    mutationFn: (id) => apiFetch<void>(`/api/calendar/holidays/${id}`, { method: "DELETE" }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["holidays"] });
      queryClient.invalidateQueries({ queryKey: ["calendar"] });
    },
  });
}

export function useListTimeOffQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<TimeOff[]>({
    queryKey: ["time-off"],
    queryFn: () => apiFetch<TimeOff[]>("/api/calendar/time-off"),
  });
}

export function useAddTimeOffMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<number, Error, { resourceId: number; day: string; fraction: number | null; reason: string | null }>({
    mutationFn: (args) =>
      apiFetch<number>("/api/calendar/time-off", {
        method: "POST",
        body: { resource_id: args.resourceId, day: args.day, fraction: args.fraction, reason: args.reason },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["time-off"] });
      queryClient.invalidateQueries({ queryKey: ["calendar"] });
    },
  });
}

export function useDeleteTimeOffMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, number>({
    mutationFn: (id) => apiFetch<void>(`/api/calendar/time-off/${id}`, { method: "DELETE" }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["time-off"] });
      queryClient.invalidateQueries({ queryKey: ["calendar"] });
    },
  });
}
```

- [ ] **Step 2: typecheck + commit**

```bash
pnpm exec vue-tsc -b && git add src/services/api/calendar.api.ts && git commit -m "feat(services/api): calendar.api.ts（work-week/holidays/time-off 8 composable）"
```

---

## Task 10: teams.api.ts

**Files:** Create `src/services/api/teams.api.ts`

- [ ] **Step 1: 创建文件**

Create `src/services/api/teams.api.ts`:
```ts
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { Team, TeamMember, TeamOverride } from "@/types";

export function useListTeamsQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<Team[]>({
    queryKey: ["teams"],
    queryFn: () => apiFetch<Team[]>("/api/teams"),
  });
}

export function useCreateTeamMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<number, Error, { name: string; description: string | null }>({
    mutationFn: (args) => apiFetch<number>("/api/teams", { method: "POST", body: { name: args.name, description: args.description } }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["teams"] });
    },
  });
}

export function useDeleteTeamMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, number>({
    mutationFn: (id) => apiFetch<void>(`/api/teams/${id}`, { method: "DELETE" }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["teams"] });
    },
  });
}

export function useListTeamMembersQuery(teamId: number) {
  const { apiFetch } = useApiFetch();
  return useQuery<TeamMember[]>({
    queryKey: ["team-members", teamId],
    queryFn: () => apiFetch<TeamMember[]>(`/api/teams/${teamId}/members`),
  });
}

export function useAddTeamMemberMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { teamId: number; resourceId: number; role: string | null }>({
    mutationFn: (args) =>
      apiFetch<void>(`/api/teams/${args.teamId}/members`, { method: "POST", body: { resource_id: args.resourceId, role: args.role } }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ["team-members", variables.teamId] });
    },
  });
}

export function useRemoveTeamMemberMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { teamId: number; resourceId: number }>({
    mutationFn: (args) => apiFetch<void>(`/api/teams/${args.teamId}/members/${args.resourceId}`, { method: "DELETE" }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ["team-members", variables.teamId] });
    },
  });
}

export function useSetTeamOverrideMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, TeamOverride>({
    mutationFn: (override) => apiFetch<void>("/api/teams/overrides", { method: "PUT", body: override }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["team-override"] });
    },
  });
}

export function useGetTeamOverrideQuery(teamId: number) {
  const { apiFetch } = useApiFetch();
  return useQuery<TeamOverride | null>({
    queryKey: ["team-override", teamId],
    queryFn: () => apiFetch<TeamOverride | null>(`/api/teams/${teamId}/override`),
  });
}
```

- [ ] **Step 2: typecheck + commit**

```bash
pnpm exec vue-tsc -b && git add src/services/api/teams.api.ts && git commit -m "feat(services/api): teams.api.ts（8 composable）"
```

---

## Task 11: gantt.api.ts

**Files:** Create `src/services/api/gantt.api.ts`

- [ ] **Step 1: 创建文件**

Create `src/services/api/gantt.api.ts`:
```ts
import { useQuery } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { DayOccupancy, DepEdge, GanttBar } from "@/types";

export function useGanttProjectQuery(projectId: number) {
  const { apiFetch } = useApiFetch();
  return useQuery<GanttBar[]>({
    queryKey: ["gantt-project", projectId],
    queryFn: () => apiFetch<GanttBar[]>(`/api/gantt/projects/${projectId}`),
  });
}

export function useGanttResourceQuery(resourceId: number) {
  const { apiFetch } = useApiFetch();
  return useQuery<GanttBar[]>({
    queryKey: ["gantt-resource", resourceId],
    queryFn: () => apiFetch<GanttBar[]>(`/api/gantt/resources/${resourceId}`),
  });
}

export function useDependenciesForProjectQuery(projectId: number) {
  const { apiFetch } = useApiFetch();
  return useQuery<DepEdge[]>({
    queryKey: ["dependencies", projectId],
    queryFn: () => apiFetch<DepEdge[]>(`/api/projects/${projectId}/dependencies`),
  });
}

export function useDailyOccupancyQuery(start: string, end: string) {
  const { apiFetch } = useApiFetch();
  return useQuery<DayOccupancy[]>({
    queryKey: ["occupancy", start, end],
    queryFn: () =>
      apiFetch<DayOccupancy[]>(`/api/occupancy?start=${encodeURIComponent(start)}&end=${encodeURIComponent(end)}`),
  });
}
```

- [ ] **Step 2: typecheck + commit**

```bash
pnpm exec vue-tsc -b && git add src/services/api/gantt.api.ts && git commit -m "feat(services/api): gantt.api.ts（4 query）"
```

---

## Task 12: optimization.api.ts

**Files:** Create `src/services/api/optimization.api.ts`

- [ ] **Step 1: 创建文件**

Create `src/services/api/optimization.api.ts`:
```ts
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { ObjectiveWeights, RunResult, RunRow } from "@/types";

export function useRunOptimizationMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<RunResult, Error, { projectId: number; weights: ObjectiveWeights | null }>({
    mutationFn: (args) =>
      apiFetch<RunResult>(`/api/optimization/run/${args.projectId}`, {
        method: "POST",
        body: args.weights ?? undefined,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["optimization-runs"] });
      // 接受方案后衍生的 allocation/workload/gantt 等也会变，applySolution 会负责失效
    },
  });
}

export function useListOptimizationRunsQuery(limit: number | null) {
  const { apiFetch } = useApiFetch();
  return useQuery<RunRow[]>({
    queryKey: ["optimization-runs", limit],
    queryFn: () =>
      apiFetch<RunRow[]>(`/api/optimization/runs${limit != null ? `?limit=${limit}` : ""}`),
  });
}

export function useApplySolutionMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<number, Error, number>({
    mutationFn: (runId) => apiFetch<number>(`/api/optimization/runs/${runId}/apply`, { method: "POST" }),
    onSuccess: () => {
      // 接受方案会改 allocations → 失效所有 allocation 衍生视图
      queryClient.invalidateQueries({ queryKey: ["allocations"] });
      queryClient.invalidateQueries({ queryKey: ["workload"] });
      queryClient.invalidateQueries({ queryKey: ["gantt"] });
      queryClient.invalidateQueries({ queryKey: ["kanban"] });
      queryClient.invalidateQueries({ queryKey: ["calendar"] });
      queryClient.invalidateQueries({ queryKey: ["optimization-runs"] });
    },
  });
}

export function useRejectSolutionMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, number>({
    mutationFn: (runId) => apiFetch<void>(`/api/optimization/runs/${runId}/reject`, { method: "POST" }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["optimization-runs"] });
    },
  });
}
```

- [ ] **Step 2: typecheck + commit**

```bash
pnpm exec vue-tsc -b && git add src/services/api/optimization.api.ts && git commit -m "feat(services/api): optimization.api.ts（4 composable，applySolution 跨域失效）"
```

---

## Task 13: reports.api.ts（特殊：query + blob 导出 helper + 类型）

**Files:** Create `src/services/api/reports.api.ts`

> reports 的 `exportReport`/`exportSnapshot` 是 blob 下载（触发浏览器下载副作用），不适合 useMutation，故建模为命令式 async helper（内部用 apiFetch 取 blob）。`getReportCatalog` 是普通 query。

- [ ] **Step 1: 创建文件**

Create `src/services/api/reports.api.ts`:
```ts
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
  const blob = await apiFetch<Blob>(`/api/reports/${kind}?${params}`, { responseType: "blob" });
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
  const blob = await apiFetch<Blob>(`/api/reports/snapshot?${params}`, { responseType: "blob" });
  triggerDownload(blob, "workforce-snapshot.json");
  return true;
}
```

注意：
- `exportReport`/`exportSnapshot` 接收 `apiFetch` 作参数（而非内部调 `useApiFetch()`），因为它们是普通函数不是 composable（不能在模块顶层调 useApiFetch）。3b 接入 page 时，page 在 setup 内 `const { apiFetch } = useApiFetch()` 后传进来。
- ofetch 的 blob：用 `responseType: "blob"`。
- report 类型/常量在本文件自定义（与旧 api/index.ts 重复，3c 删旧时去重）。

- [ ] **Step 2: typecheck + commit**

```bash
pnpm exec vue-tsc -b && git add src/services/api/reports.api.ts && git commit -m "feat(services/api): reports.api.ts（catalog query + exportReport/exportSnapshot blob helper + 类型）"
```

---

## Task 14: 完整验证

**Files:** 无修改，纯验证。

- [ ] **Step 1: 完整构建**

Run:
```bash
pnpm build 2>&1 | tail -10
```
Expected: `vue-tsc -b && vite build` 成功，dist 发出，exit 0。

- [ ] **Step 2: 单元测试**

Run:
```bash
pnpm test 2>&1 | tail -6
```
Expected: 16/16（旧测试不动；新 composable 无消费者，本期不加新测试）。

- [ ] **Step 3: dev 行为零变化验证（端口可能被占，用备用端口）**

```bash
pnpm exec vite --port 1430 --strictPort false > /tmp/s3dev.log 2>&1 &
sleep 6
curl -s -o /dev/null -w "root HTTP %{http_code}\n" http://localhost:1430/
# 新 services 文件能被请求（说明编译通过）
for f in services/fetch.ts services/api/projects.api.ts services/api/reports.api.ts; do
  code=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:1430/src/$f")
  echo "  /src/$f -> HTTP $code"
done
grep -iE "error|fail" /tmp/s3dev.log | grep -viE "INVALID_ANNOTATION|@vueuse|devtools" | head -5 || echo "(no errors)"
pkill -f "vite --port 1430" 2>/dev/null
```
Expected: 全 200，无错误。app 仍走旧 store/api（新 composable 未被 import）。

- [ ] **Step 4: 确认旧 api/index.ts / store / page 未被改动**

```bash
git diff main..HEAD --stat -- src/api/index.ts src/stores/ src/pages/ src/components/ src/types.ts
```
Expected: 空（这些文件本期零改动）。

- [ ] **Step 5: 确认 services/ 全部就位**

```bash
find src/services -type f | sort
```
Expected: 14 文件（fetch.ts + types/response.type.ts + api/*.api.ts × 12）。

---

## 完成判据（Definition of Done）

对应 spec §5：
1. ✅ `pnpm build` 通过（Task 14 Step 1）
2. ✅ `pnpm test` 16/16（Task 14 Step 2）
3. ✅ `pnpm dev` 行为零变化（Task 14 Step 3）
4. ✅ `services/api/` 12 文件 + fetch.ts + response.type.ts 就位，类型检查通过（Task 14 Step 5）
5. ✅ QueryClient 含 refetchOnWindowFocus:false + retry:1（Task 1 Step 3）
6. ✅ `@/utils/env` 被 services/fetch.ts 消费（Task 1 Step 1）
7. ✅ 旧 api/store/page 零改动（Task 14 Step 4）

## 风险速查（详见 spec §6）

- ofetch 相对 baseURL → 空串同源，dev 走 vite proxy（Task 14 Step 3 验证）
- 55+ 方法逐一迁移遗漏 → 以旧 api/index.ts 为权威，逐方法对照；tsc 捕获类型偏差
- composable 无消费者被 tree-shake → 不影响正确性（3b 接入后保留）
- queryKey 约定与 3b 实际需求不匹配 → 3a 建 `["<domain>"]` 基础，3b 按需细化
- ofetch blob responseType → reports.api.ts 用 `responseType: "blob"`（Task 13）
