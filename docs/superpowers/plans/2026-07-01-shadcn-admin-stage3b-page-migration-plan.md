# Stage 3b — Page Migration (store → composable) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Each numbered file step is one independent migration unit; run `pnpm build` and `pnpm test` at the end of each task before committing.

**Goal:** Move all `src/pages/` and `src/components/` data access from legacy Pinia stores to the `services/api/` TanStack Query composables built in stage 3a, while keeping `useProjectsStore` and `useUnitStore` as thin UI-state stores and rewriting `AppLayout` bootstrap to use vue-query.

**Architecture:** Replace every `useXxxStore()` call with the corresponding `useXxxQuery()`/`useXxxMutation()` from `@/services/api/*.api.ts`. Delete all `watch` consumers of `useRefreshStore`; rely on mutation `onSuccess` invalidations configured in 3a. Preserve all snake_case field reads in templates. Move any store-derived computed logic (kanban columns, utilization band) into the page/component that owns it.

**Tech Stack:** Vue 3.5, Vite 8, TanStack Vue Query, Pinia, ofetch, Tailwind v4, shadcn-vue.

---

## Conventions

- Every changed page/component keeps its existing `<template>` unchanged unless explicitly noted.
- Import order in `<script setup>`: Vue → TanStack → services/api → stores (only thin projects/unit) → components → utils → types.
- `MaybeRef<T>` means `import { type MaybeRef, computed, toValue } from "vue";` and `enabled: () => id.value != null`.
- All mutation calls use `.mutate(...)` (or `.mutateAsync(...)` when the call site needs `await`).

---

## Task 0: API refinements — optional queries (`MaybeRef` + `enabled`)

Some pages need queries that only run when a project/resource/team/date is selected. The 3a composables currently take raw values; update them to accept `MaybeRef` and use `enabled`. This is a mechanical prerequisite, no behavior change.

**Files:**
- Modify: `src/services/api/tasks.api.ts`
- Modify: `src/services/api/allocations.api.ts`
- Modify: `src/services/api/gantt.api.ts`
- Modify: `src/services/api/workload.api.ts`
- Modify: `src/services/api/teams.api.ts`

### 0.1 `src/services/api/tasks.api.ts` — optional project IDs

Replace the two query functions at the top of the file.

```ts
import { type MaybeRef, computed, toValue } from "vue";

export function useListTasksQuery(projectId: MaybeRef<number | null>) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(projectId));
  return useQuery<Task[]>({
    queryKey: computed(() => ["tasks", id.value]),
    queryFn: () => apiFetch<Task[]>(`/api/projects/${id.value}/tasks`),
    enabled: () => id.value != null,
  });
}

export function useKanbanTasksQuery(projectId: MaybeRef<number | null>) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(projectId));
  return useQuery<KanbanTask[]>({
    queryKey: computed(() => ["kanban", id.value]),
    queryFn: () => apiFetch<KanbanTask[]>(`/api/projects/${id.value}/kanban`),
    enabled: () => id.value != null,
  });
}
```

Leave mutations unchanged.

### 0.2 `src/services/api/allocations.api.ts` — optional project ID

Replace `useListAllocationsQuery`:

```ts
import { type MaybeRef, computed, toValue } from "vue";

export function useListAllocationsQuery(projectId: MaybeRef<number | null>) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(projectId));
  return useQuery<AllocationView[]>({
    queryKey: computed(() => ["allocations", id.value]),
    queryFn: () => apiFetch<AllocationView[]>(`/api/projects/${id.value}/allocations`),
    enabled: () => id.value != null,
  });
}
```

Leave mutations unchanged.

### 0.3 `src/services/api/gantt.api.ts` — optional project/resource IDs and dates

Replace the query functions:

```ts
import { type MaybeRef, computed, toValue } from "vue";

export function useGanttProjectQuery(projectId: MaybeRef<number | null>) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(projectId));
  return useQuery<GanttBar[]>({
    queryKey: computed(() => ["gantt-project", id.value]),
    queryFn: () => apiFetch<GanttBar[]>(`/api/gantt/projects/${id.value}`),
    enabled: () => id.value != null,
  });
}

export function useGanttResourceQuery(resourceId: MaybeRef<number | null>) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(resourceId));
  return useQuery<GanttBar[]>({
    queryKey: computed(() => ["gantt-resource", id.value]),
    queryFn: () => apiFetch<GanttBar[]>(`/api/gantt/resources/${id.value}`),
    enabled: () => id.value != null,
  });
}

export function useDependenciesForProjectQuery(projectId: MaybeRef<number | null>) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(projectId));
  return useQuery<DepEdge[]>({
    queryKey: computed(() => ["dependencies", id.value]),
    queryFn: () => apiFetch<DepEdge[]>(`/api/projects/${id.value}/dependencies`),
    enabled: () => id.value != null,
  });
}

export function useDailyOccupancyQuery(start: MaybeRef<string>, end: MaybeRef<string>) {
  const { apiFetch } = useApiFetch();
  const s = computed(() => toValue(start));
  const e = computed(() => toValue(end));
  return useQuery<DayOccupancy[]>({
    queryKey: computed(() => ["occupancy", s.value, e.value]),
    queryFn: () =>
      apiFetch<DayOccupancy[]>(`/api/occupancy?start=${encodeURIComponent(s.value)}&end=${encodeURIComponent(e.value)}`),
    enabled: () => !!s.value && !!e.value,
  });
}
```

### 0.4 `src/services/api/workload.api.ts` — optional IDs and dates

Replace the query functions:

```ts
import { type MaybeRef, computed, toValue } from "vue";

export function useResourceSummaryQuery(
  resourceId: MaybeRef<number | null>,
  start: MaybeRef<string>,
  end: MaybeRef<string>,
) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(resourceId));
  const s = computed(() => toValue(start));
  const e = computed(() => toValue(end));
  return useQuery<ResourceSummary>({
    queryKey: computed(() => ["workload-resource", id.value, s.value, e.value]),
    queryFn: () =>
      apiFetch<ResourceSummary>(
        `/api/workload/resources/${id.value}?start=${encodeURIComponent(s.value)}&end=${encodeURIComponent(e.value)}`,
      ),
    enabled: () => id.value != null && !!s.value && !!e.value,
  });
}

export function useTeamSummaryQuery(
  teamId: MaybeRef<number | null>,
  start: MaybeRef<string>,
  end: MaybeRef<string>,
) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(teamId));
  const s = computed(() => toValue(start));
  const e = computed(() => toValue(end));
  return useQuery<TeamSummary>({
    queryKey: computed(() => ["workload-team", id.value, s.value, e.value]),
    queryFn: () =>
      apiFetch<TeamSummary>(
        `/api/workload/teams/${id.value}?start=${encodeURIComponent(s.value)}&end=${encodeURIComponent(e.value)}`,
      ),
    enabled: () => id.value != null && !!s.value && !!e.value,
  });
}

export function useOverloadsQuery(start: MaybeRef<string>, end: MaybeRef<string>) {
  const { apiFetch } = useApiFetch();
  const s = computed(() => toValue(start));
  const e = computed(() => toValue(end));
  return useQuery<ResourceSummary[]>({
    queryKey: computed(() => ["workload-overloads", s.value, e.value]),
    queryFn: () =>
      apiFetch<ResourceSummary[]>(
        `/api/workload/overloads?start=${encodeURIComponent(s.value)}&end=${encodeURIComponent(e.value)}`,
      ),
    enabled: () => !!s.value && !!e.value,
  });
}

export function useProjectBurnQuery(projectId: MaybeRef<number | null>) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(projectId));
  return useQuery<ProjectBurn>({
    queryKey: computed(() => ["workload-burn", id.value]),
    queryFn: () => apiFetch<ProjectBurn>(`/api/projects/${id.value}/burn`),
    enabled: () => id.value != null,
  });
}
```

### 0.5 `src/services/api/teams.api.ts` — optional team ID

Replace the two query functions:

```ts
import { type MaybeRef, computed, toValue } from "vue";

export function useListTeamMembersQuery(teamId: MaybeRef<number | null>) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(teamId));
  return useQuery<TeamMember[]>({
    queryKey: computed(() => ["team-members", id.value]),
    queryFn: () => apiFetch<TeamMember[]>(`/api/teams/${id.value}/members`),
    enabled: () => id.value != null,
  });
}

export function useGetTeamOverrideQuery(teamId: MaybeRef<number | null>) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(teamId));
  return useQuery<TeamOverride | null>({
    queryKey: computed(() => ["team-override", id.value]),
    queryFn: () => apiFetch<TeamOverride | null>(`/api/teams/${id.value}/override`),
    enabled: () => id.value != null,
  });
}
```

### 0.6 Verify & commit

- [ ] Run `pnpm build`.
- [ ] Run `pnpm test`.
- [ ] Commit:

```bash
git add src/services/api/tasks.api.ts src/services/api/allocations.api.ts src/services/api/gantt.api.ts src/services/api/workload.api.ts src/services/api/teams.api.ts
git commit -m "feat(api): accept MaybeRef and enabled for optional queries"
```

---

## Task 1: Rewrite thin stores

### 1.1 Rewrite `src/stores/projects.ts`

Replace the entire file with:

```ts
import { defineStore } from "pinia";
import { ref } from "vue";

export const useProjectsStore = defineStore("projects", () => {
  const current = ref<number | null>(null);

  function select(id: number) {
    current.value = id;
  }

  return { current, select };
});
```

### 1.2 Rewrite `src/stores/unit.ts`

Replace the entire file with:

```ts
import { defineStore } from "pinia";
import { ref, watch } from "vue";

const STORAGE_KEY = "dev-resource-kanban.unit";

function initialUnit(): "PD" | "PM" {
  const saved = globalThis.localStorage?.getItem(STORAGE_KEY);
  return saved === "PM" ? "PM" : "PD";
}

export const useUnitStore = defineStore("unit", () => {
  const unit = ref<"PD" | "PM">(initialUnit());

  watch(unit, (value) => {
    globalThis.localStorage?.setItem(STORAGE_KEY, value);
  });

  function formatPd(pd: number | null | undefined): string {
    if (pd == null) return "—";
    return unit.value === "PM" ? `${(pd / 20).toFixed(1)} PM` : `${pd.toFixed(1)} PD`;
  }

  function applyTeamOverride(pmWorkdays: number | null): number {
    return pmWorkdays ?? 20;
  }

  return { unit, formatPd, applyTeamOverride };
});
```

### 1.3 Verify & commit

- [ ] Run `pnpm build`.
- [ ] Run `pnpm test` (existing store tests may fail because the old `items`/`load` API is gone; those tests are deleted in 3c, so do not fix them in 3b).
- [ ] Commit:

```bash
git add src/stores/projects.ts src/stores/unit.ts
git commit -m "feat(stores): slim projects and unit stores to pure UI state"
```

---

## Task 2: Batch 1 — catalog & settings pages

### 2.1 Migrate `src/pages/catalog/index.vue`

Replace the entire `<script setup>` block:

```ts
import { ref } from "vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { useListSkillsQuery, useListTagsQuery, useEnsureSkillMutation, useEnsureTagMutation } from "@/services/api/catalog.api";

const skillsQuery = useListSkillsQuery();
const tagsQuery = useListTagsQuery();
const ensureSkill = useEnsureSkillMutation();
const ensureTag = useEnsureTagMutation();

const skillName = ref("");
const tagName = ref("");

async function addSkill() {
  if (!skillName.value.trim()) return;
  await ensureSkill.mutateAsync(skillName.value);
  skillName.value = "";
}

async function addTag() {
  if (!tagName.value.trim()) return;
  await ensureTag.mutateAsync({ name: tagName.value, color: null });
  tagName.value = "";
}
```

Template changes: replace `catalog.skills` with `skillsQuery.data.value ?? []` and `catalog.tags` with `tagsQuery.data.value ?? []`.

### 2.2 Migrate `src/pages/settings/index.vue`

Replace the entire `<script setup>` block:

```ts
import { ref, watch } from "vue";
import { toast } from "vue-sonner";
import { useGetSettingsQuery, useUpdateSettingsMutation } from "@/services/api/config.api";
import type { Settings } from "@/types";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  NumberField,
  NumberFieldContent,
  NumberFieldDecrement,
  NumberFieldIncrement,
  NumberFieldInput,
} from "@/components/ui/number-field";
import { Skeleton } from "@/components/ui/skeleton";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { Loader2Icon } from "@lucide/vue";

const settingsQuery = useGetSettingsQuery();
const updateSettings = useUpdateSettingsMutation();
const draft = ref<Settings | null>(null);

watch(
  () => settingsQuery.data.value,
  (s) => {
    if (s && !draft.value) {
      draft.value = { ...s };
    }
  },
  { immediate: true },
);

const unitOptions = [
  { label: "PD (人日)", value: "PD" },
  { label: "PM (人月)", value: "PM" },
];

const providerOptions = [
  { label: "Ollama", value: "ollama" },
  { label: "OpenAI", value: "openai" },
  { label: "Anthropic", value: "anthropic" },
  { label: "DeepSeek", value: "deepseek" },
];

const secretStoreOptions = [
  { label: "Keychain", value: "keychain" },
  { label: "Encrypted File", value: "encrypted_file" },
];

const solverOptions = [
  { label: "good_lp", value: "good_lp" },
  { label: "Greedy", value: "greedy" },
  { label: "Hungarian", value: "hungarian" },
];

async function save() {
  if (!draft.value) return;
  try {
    await updateSettings.mutateAsync(draft.value);
    toast.success("设置已保存");
  } catch (e) {
    toast.error(`保存失败: ${e instanceof Error ? e.message : String(e)}`);
  }
}

function reset() {
  if (settingsQuery.data.value) {
    draft.value = { ...settingsQuery.data.value };
  }
}

function updateNullableString(
  field: "ai_base_url" | "ai_api_key_enc" | "embed_base_url" | "embed_api_key_enc",
  value: string | number,
) {
  if (!draft.value) return;
  draft.value[field] = String(value || "");
}
```

Template changes: replace `settings.loading` with `settingsQuery.isLoading` and `settings.saving` with `updateSettings.isPending`.

### 2.3 Verify & commit

- [ ] Run `pnpm build`.
- [ ] Run `pnpm test`.
- [ ] Spot-check `/catalog` and `/settings` in `pnpm dev`.
- [ ] Commit:

```bash
git add src/pages/catalog/index.vue src/pages/settings/index.vue
git commit -m "feat(pages): migrate catalog and settings to vue-query"
```

---

## Task 3: Batch 2 — projects, resources, kanban + forms

### 3.1 Migrate `src/pages/projects/index.vue`

Replace the entire `<script setup>` block:

```ts
import { computed, ref } from "vue";
import { PlusIcon } from "@lucide/vue";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  NumberField,
  NumberFieldContent,
  NumberFieldDecrement,
  NumberFieldIncrement,
  NumberFieldInput,
} from "@/components/ui/number-field";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Textarea } from "@/components/ui/textarea";
import DateRangePicker from "@/components/DateRangePicker.vue";
import ListPage from "@/components/list/ListPage.vue";
import ListRowActions from "@/components/list/ListRowActions.vue";
import ListToolbar from "@/components/list/ListToolbar.vue";
import ProjectForm from "@/components/ProjectForm.vue";
import TaskForm from "@/components/TaskForm.vue";
import { useListProjectsQuery, useUpdateProjectMutation, useDeleteProjectMutation } from "@/services/api/projects.api";
import { useProjectsStore } from "@/stores/projects";
import { useUnitStore } from "@/stores/unit";
import { fmtDate, parseDate } from "@/utils/date";
import type { Project } from "@/types";

const projectsQuery = useListProjectsQuery();
const updateProject = useUpdateProjectMutation();
const deleteProject = useDeleteProjectMutation();
const projects = useProjectsStore();
const unit = useUnitStore();

// Filters
const filterName = ref("");
const filterStatus = ref("all");
const statusOptions = [
  { label: "active", value: "active" },
  { label: "done", value: "done" },
];

const isFiltered = computed(() => !!(filterName.value || filterStatus.value !== "all"));

const filteredProjects = computed(() => {
  return (projectsQuery.data.value ?? []).filter((p) => {
    const matchesName = !filterName.value || p.name.toLowerCase().includes(filterName.value.toLowerCase());
    const matchesStatus = filterStatus.value === "all" || p.status === filterStatus.value;
    return matchesName && matchesStatus;
  });
});

function resetFilters() {
  filterName.value = "";
  filterStatus.value = "all";
}

// Edit dialog state
const editVisible = ref(false);
const editingId = ref<number | null>(null);
const editName = ref("");
const editPriority = ref(5);
const editBudget = ref(0);
const editDateRange = ref<[number, number] | null>(null);
const editDescription = ref("");

const todayMs = computed(() => parseDate(fmtDate(Date.now())) ?? Date.now());

function openEdit(p: Project) {
  editingId.value = p.id;
  editName.value = p.name;
  editPriority.value = p.priority;
  editBudget.value = p.budget_pd;
  const start = parseDate(p.start_date);
  const end = parseDate(p.end_date);
  editDateRange.value = start != null && end != null ? [start, end] : null;
  editDescription.value = p.description ?? "";
  editVisible.value = true;
}

async function saveEdit() {
  if (editingId.value == null) return;
  await updateProject.mutateAsync({
    id: editingId.value,
    name: editName.value,
    priority: editPriority.value,
    budgetPd: editBudget.value,
    description: editDescription.value || null,
    start: editDateRange.value ? fmtDate(editDateRange.value[0]) : null,
    end: editDateRange.value ? fmtDate(editDateRange.value[1]) : null,
  });
  editVisible.value = false;
}

// Delete confirmation dialog state
const deleteId = ref<number | null>(null);
const deleteName = ref("");
const deleteOpen = computed({
  get: () => deleteId.value != null,
  set: (v) => {
    if (!v) {
      deleteId.value = null;
      deleteName.value = "";
    }
  },
});

function confirmDelete(id: number, name: string) {
  deleteId.value = id;
  deleteName.value = name;
}

async function doDelete() {
  if (deleteId.value == null) return;
  await deleteProject.mutateAsync(deleteId.value);
  if (projects.current === deleteId.value) {
    projects.select(0);
  }
  deleteId.value = null;
  deleteName.value = "";
}

// New project dialog
const createVisible = ref(false);
```

Template changes: replace `projects.items` with `projectsQuery.data.value ?? []`.

### 3.2 Migrate `src/pages/resources/index.vue`

Replace the entire `<script setup>` block:

```ts
import { computed, ref, watch } from "vue";
import { PlusIcon } from "@lucide/vue";
import { CalendarIcon } from "@lucide/vue";
import { CalendarDate, type DateValue } from "@internationalized/date";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import {
  NumberField,
  NumberFieldContent,
  NumberFieldDecrement,
  NumberFieldIncrement,
  NumberFieldInput,
} from "@/components/ui/number-field";
import { Calendar } from "@/components/ui/calendar";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import ResourceForm from "@/components/ResourceForm.vue";
import ListPage from "@/components/list/ListPage.vue";
import ListRowActions from "@/components/list/ListRowActions.vue";
import ListToolbar from "@/components/list/ListToolbar.vue";
import {
  useListResourcesQuery,
  useCreateResourceMutation,
  useUpdateResourceMutation,
  useDeleteResourceMutation,
  useGetResourceSkillsQuery,
  useSetResourceSkillsMutation,
  useGetResourceTagsQuery,
  useSetResourceTagsMutation,
} from "@/services/api/resources.api";
import { useListSkillsQuery, useListTagsQuery } from "@/services/api/catalog.api";
import { useApiFetch } from "@/services/fetch";
import { fmtDate, parseDate } from "@/utils/date";
import type { Resource } from "@/types";

const resourcesQuery = useListResourcesQuery();
const createResource = useCreateResourceMutation();
const updateResource = useUpdateResourceMutation();
const deleteResource = useDeleteResourceMutation();
const setResourceSkills = useSetResourceSkillsMutation();
const setResourceTags = useSetResourceTagsMutation();
const skillsQuery = useListSkillsQuery();
const tagsQuery = useListTagsQuery();
const { apiFetch } = useApiFetch();

// Filters
const filterName = ref("");
const filterStatus = ref("all");

const statusOptions = computed(() => {
  const set = new Set((resourcesQuery.data.value ?? []).map((r) => r.status));
  return Array.from(set).sort().map((s) => ({ label: s, value: s }));
});

const isFiltered = computed(() => !!(filterName.value || filterStatus.value !== "all"));

const filteredResources = computed(() => {
  return (resourcesQuery.data.value ?? []).filter((r) => {
    const matchesName = !filterName.value || r.name.toLowerCase().includes(filterName.value.toLowerCase());
    const matchesStatus = filterStatus.value === "all" || r.status === filterStatus.value;
    return matchesName && matchesStatus;
  });
});

function resetFilters() {
  filterName.value = "";
  filterStatus.value = "all";
}

// Create dialog
const createVisible = ref(false);

// Edit dialog state
const editVisible = ref(false);
const editingId = ref<number | null>(null);
const editName = ref("");
const editEmail = ref("");
const editAvailFrom = ref<number | null>(null);
const editAvailTo = ref<number | null>(null);
const editCapacity = ref<number | null>(null);
const editRate = ref<number | null>(null);
const editSkills = ref<{ skillId: number; proficiency: number }[]>([]);
const editTags = ref<number[]>([]);

const skillOptions = () => (skillsQuery.data.value ?? []).map((s) => ({ label: s.name, value: s.id }));
const tagOptions = () => (tagsQuery.data.value ?? []).map((t) => ({ label: t.name, value: t.id }));

const editSkillsQuery = useGetResourceSkillsQuery(computed(() => editingId.value));
const editTagsQuery = useGetResourceTagsQuery(computed(() => editingId.value));

watch(editSkillsQuery.data, (skills) => {
  if (editingId.value != null && skills) {
    editSkills.value = skills.map((s) => ({ skillId: s.skill_id, proficiency: s.proficiency }));
  }
});

watch(editTagsQuery.data, (tags) => {
  if (editingId.value != null && tags) {
    editTags.value = tags.map((t) => t.tag_id);
  }
});

function toDateValue(ms: number | null): DateValue | undefined {
  if (ms == null) return undefined;
  const s = fmtDate(ms);
  const [year, month, day] = s.split("-").map(Number);
  return new CalendarDate(year, month, day);
}

function fromDateValue(dv: DateValue): number {
  return parseDate(`${dv.year}-${String(dv.month).padStart(2, "0")}-${String(dv.day).padStart(2, "0")}`) ?? Date.now();
}

const editAvailFromDate = computed<DateValue | undefined>({
  get: () => toDateValue(editAvailFrom.value),
  set: (dv) => { editAvailFrom.value = dv ? fromDateValue(dv) : null; },
});

const editAvailToDate = computed<DateValue | undefined>({
  get: () => toDateValue(editAvailTo.value),
  set: (dv) => { editAvailTo.value = dv ? fromDateValue(dv) : null; },
});

const editCapacityModel = computed<number | undefined>({
  get: () => editCapacity.value ?? undefined,
  set: (v) => { editCapacity.value = v ?? null; },
});

const editRateModel = computed<number | undefined>({
  get: () => editRate.value ?? undefined,
  set: (v) => { editRate.value = v ?? null; },
});

function updateSelectedSkills(ids: number[]) {
  editSkills.value = ids.map((id) => {
    const existing = editSkills.value.find((s) => s.skillId === id);
    return existing ?? { skillId: id, proficiency: 3 };
  });
}

function onSkillSelect(value: unknown) {
  updateSelectedSkills(value as number[]);
}

function onTagSelect(value: unknown) {
  editTags.value = value as number[];
}

// Hover-cached inline display of skills/tags (preserves old UX without using the data store).
const skillCache = ref<Record<number, ResourceSkill[]>>({});
const tagCache = ref<Record<number, ResourceTag[]>>({});

async function loadDisplay(r: Resource) {
  if (!skillCache.value[r.id]) {
    skillCache.value[r.id] = await apiFetch<ResourceSkill[]>(`/api/resources/${r.id}/skills`);
  }
  if (!tagCache.value[r.id]) {
    tagCache.value[r.id] = await apiFetch<ResourceTag[]>(`/api/resources/${r.id}/tags`);
  }
}

function openEdit(r: Resource) {
  editingId.value = r.id;
  editName.value = r.name;
  editEmail.value = r.email ?? "";
  editAvailFrom.value = parseDate(r.available_from);
  editAvailTo.value = parseDate(r.available_to);
  editCapacity.value = r.daily_capacity_pd;
  editRate.value = r.daily_rate_pd;
  editVisible.value = true;
}

async function saveEdit() {
  if (editingId.value == null) return;
  await updateResource.mutateAsync({
    id: editingId.value,
    name: editName.value,
    email: editEmail.value || null,
    availableFrom: editAvailFrom.value != null ? fmtDate(editAvailFrom.value) : null,
    availableTo: editAvailTo.value != null ? fmtDate(editAvailTo.value) : null,
    dailyCapacityPd: editCapacity.value,
    dailyRatePd: editRate.value,
  });
  await setResourceSkills.mutateAsync({ id: editingId.value, skills: editSkills.value.map((s) => [s.skillId, s.proficiency]) });
  await setResourceTags.mutateAsync({ id: editingId.value, tagIds: editTags.value });
  editVisible.value = false;
  editingId.value = null;
}

// Delete confirmation dialog state
const deleteDialogOpen = ref(false);
const deletingId = ref<number | null>(null);
const deletingName = ref("");

function openDelete(r: Resource) {
  deletingId.value = r.id;
  deletingName.value = r.name;
  deleteDialogOpen.value = true;
}

async function confirmDelete() {
  if (deletingId.value == null) return;
  await deleteResource.mutateAsync(deletingId.value);
  deleteDialogOpen.value = false;
  deletingId.value = null;
  deletingName.value = "";
}
```

Template changes: replace `resources.items` with `filteredResources`. Keep the `skillCache` and `tagCache` inline display cells exactly as they are; they now read from local hover-populated caches populated by `loadDisplay`. No other template changes needed.

### 3.3 Migrate `src/pages/kanban/index.vue`

Replace the entire `<script setup>` block:

```ts
import { computed, ref } from "vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { NumberField, NumberFieldContent, NumberFieldDecrement, NumberFieldIncrement, NumberFieldInput } from "@/components/ui/number-field";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { Alert, AlertDescription } from "@/components/ui/alert";
import {
  useKanbanTasksQuery,
  useUpdateTaskMutation,
  useDeleteTaskMutation,
  useSetTaskStatusMutation,
  useAddDependencyMutation,
} from "@/services/api/tasks.api";
import { useProjectsStore } from "@/stores/projects";
import KanbanColumn from "@/components/KanbanColumn.vue";
import DateRangePicker from "@/components/DateRangePicker.vue";
import { fmtDate, parseDate } from "@/utils/date";
import type { KanbanTask, TaskStatus } from "@/types";

const COLUMNS: TaskStatus[] = ["todo", "in_progress", "blocked", "review", "done"];

const projects = useProjectsStore();
const kanbanQuery = useKanbanTasksQuery(computed(() => projects.current));
const updateTask = useUpdateTaskMutation();
const deleteTask = useDeleteTaskMutation();
const setTaskStatus = useSetTaskStatusMutation();
const addDependency = useAddDependencyMutation();
const draggingId = ref<number | null>(null);

const tasks = computed(() => kanbanQuery.data.value ?? []);

// Edit modal state
const editVisible = ref(false);
const editing = ref<KanbanTask | null>(null);
const editTitle = ref("");
const editEstimate = ref(0);
const editDateRange = ref<[number, number] | null>(null);
const editDescription = ref("");
const depPredecessor = ref<number | null>(null);
const depLag = ref(0);
const editError = ref<string | null>(null);

/** Pull the human-readable `detail` out of the backend's `{code,detail}` error body. */
function errText(e: unknown): string {
  const raw = e instanceof Error ? e.message : String(e);
  try {
    const j = JSON.parse(raw);
    if (j && typeof j.detail === "string") return j.detail;
  } catch { /* not a JSON error body */ }
  return raw;
}

const otherTasks = computed(() =>
  tasks.value.filter((t) => t.id !== editing.value?.id),
);
const predecessorOptions = computed(() =>
  otherTasks.value.map((t) => ({ label: t.title, value: t.id })),
);

function byStatus(status: TaskStatus): KanbanTask[] {
  return tasks.value.filter((t) => t.status === status).sort((a, b) => a.sort_order - b.sort_order);
}

function onDrop(status: TaskStatus) {
  if (draggingId.value == null) return;
  const task = tasks.value.find((t) => t.id === draggingId.value);
  if (!task) return;
  const prevStatus = task.status;
  task.status = status; // optimistic
  setTaskStatus.mutate(
    { id: draggingId.value, status, projectId: projects.current ?? undefined },
    {
      onError: () => {
        task.status = prevStatus;
      },
    },
  );
  draggingId.value = null;
}

function onEdit(task: KanbanTask) {
  editing.value = task;
  editTitle.value = task.title;
  editEstimate.value = task.estimate_pd;
  const start = parseDate(task.start_date);
  const end = parseDate(task.end_date);
  editDateRange.value = start != null && end != null ? [start, end] : null;
  editDescription.value = task.description ?? "";
  depPredecessor.value = null;
  depLag.value = 0;
  editError.value = null;
  editVisible.value = true;
}

async function saveEdit() {
  if (!editing.value || !projects.current) return;
  editError.value = null;
  try {
    await updateTask.mutateAsync({
      id: editing.value.id,
      projectId: projects.current,
      title: editTitle.value,
      estimatePd: editEstimate.value,
      start: editDateRange.value ? fmtDate(editDateRange.value[0]) : null,
      end: editDateRange.value ? fmtDate(editDateRange.value[1]) : null,
      description: editDescription.value || null,
    });
    if (depPredecessor.value != null) {
      await addDependency.mutateAsync({
        taskId: editing.value.id,
        predecessorId: depPredecessor.value,
        lagDays: depLag.value,
        projectId: projects.current,
      });
    }
    editVisible.value = false;
  } catch (e: unknown) {
    editError.value = errText(e);
  }
}

async function onDelete(id: number) {
  await deleteTask.mutateAsync({ id, projectId: projects.current ?? undefined });
}

function onPredecessorChange(value: unknown) {
  depPredecessor.value = value == null ? null : (value as number);
}
```

Template changes: replace `tasks.columns` with `COLUMNS`, `tasks.byStatus(col)` with `byStatus(col)`, and `tasks.tasks` with `tasks`. Remove the refresh-bus watcher.

### 3.4 Migrate `src/components/ProjectForm.vue`

Replace the entire `<script setup>` block:

```ts
import { ref } from "vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  NumberField,
  NumberFieldContent,
  NumberFieldDecrement,
  NumberFieldIncrement,
  NumberFieldInput,
} from "@/components/ui/number-field";
import { useCreateProjectMutation } from "@/services/api/projects.api";

const createProject = useCreateProjectMutation();
const name = ref("");
const priority = ref(5);
const budget = ref(0);

async function submit() {
  if (!name.value.trim()) return;
  await createProject.mutateAsync({ name: name.value, priority: priority.value, budgetPd: budget.value });
  name.value = "";
}
```

### 3.5 Migrate `src/components/TaskForm.vue`

Replace the entire `<script setup>` block:

```ts
import { computed, ref } from "vue";
import { CalendarDate, type DateValue } from "@internationalized/date";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  NumberField,
  NumberFieldContent,
  NumberFieldDecrement,
  NumberFieldIncrement,
  NumberFieldInput,
} from "@/components/ui/number-field";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Calendar } from "@/components/ui/calendar";
import { useCreateTaskMutation, type SkillReq, useListTasksQuery } from "@/services/api/tasks.api";
import { useListSkillsQuery, useListTagsQuery } from "@/services/api/catalog.api";
import { useProjectsStore } from "@/stores/projects";
import { fmtDate, fmtDateOrNull, parseDate } from "@/utils/date";

const createTask = useCreateTaskMutation();
const projects = useProjectsStore();
const skillsQuery = useListSkillsQuery();
const tagsQuery = useListTagsQuery();
const tasksQuery = useListTasksQuery(computed(() => projects.current));

const title = ref("");
const estimate = ref(1);
const selectedSkills = ref<number[]>([]);
const selectedTags = ref<number[]>([]);
const isLongTerm = ref(false);
const segmentKind = ref<string | null>(null);
const parentTaskId = ref<number | null>(null);
const startMs = ref<number | null>(null);
const endMs = ref<number | null>(null);

const skillOptions = computed(() =>
  (skillsQuery.data.value ?? []).map((s) => ({ label: s.name, value: s.id })),
);
const tagOptions = computed(() =>
  (tagsQuery.data.value ?? []).map((t) => ({ label: t.name, value: t.id })),
);

const parentOptions = computed(() =>
  (tasksQuery.data.value ?? [])
    .filter((t) => t.title)
    .map((t) => ({ label: t.title, value: t.id })),
);
const segmentKindOptions = [
  { label: "阶段 phase", value: "phase" },
  { label: "里程碑 milestone", value: "milestone" },
  { label: "分段 segment", value: "segment" },
];

function toCalendarDate(ms: number): DateValue {
  const [y, m, d] = fmtDate(ms).split("-").map(Number);
  return new CalendarDate(y, m, d);
}

function dateValueToMs(v: DateValue | null | undefined): number | null {
  if (!v) return null;
  return parseDate(`${v.year}-${String(v.month).padStart(2, "0")}-${String(v.day).padStart(2, "0")}`);
}

const startDate = computed<DateValue | undefined>({
  get: () => (startMs.value ? toCalendarDate(startMs.value) : undefined),
  set: (v) => { startMs.value = dateValueToMs(v); },
});

const endDate = computed<DateValue | undefined>({
  get: () => (endMs.value ? toCalendarDate(endMs.value) : undefined),
  set: (v) => { endMs.value = dateValueToMs(v); },
});

function handleSkillUpdate(v: unknown) {
  selectedSkills.value = (v as number[] | undefined) ?? [];
}

function handleTagUpdate(v: unknown) {
  selectedTags.value = (v as number[] | undefined) ?? [];
}

function handleSegmentKindUpdate(v: unknown) {
  segmentKind.value = (v as string | undefined) || null;
  if (!segmentKind.value) parentTaskId.value = null;
}

function handleParentUpdate(v: unknown) {
  parentTaskId.value = (v as number | undefined) ?? null;
}

async function submit() {
  if (!title.value.trim() || !projects.current) return;
  const skillReqs = selectedSkills.value.map((id) => [id, 3, true, 1] as SkillReq);
  await createTask.mutateAsync({
    projectId: projects.current,
    title: title.value,
    estimatePd: estimate.value,
    start: fmtDateOrNull(startMs.value),
    end: fmtDateOrNull(endMs.value),
    skillReqs,
    tagIds: selectedTags.value,
    isLongTerm: isLongTerm.value,
    parentTaskId: segmentKind.value ? parentTaskId.value : null,
    segmentKind: segmentKind.value,
  });
  title.value = "";
  estimate.value = 1;
  selectedSkills.value = [];
  selectedTags.value = [];
  isLongTerm.value = false;
  segmentKind.value = null;
  parentTaskId.value = null;
  startMs.value = null;
  endMs.value = null;
}
```

### 3.6 Migrate `src/components/ResourceForm.vue`

Replace the entire `<script setup>` block:

```ts
import { ref } from "vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useCreateResourceMutation } from "@/services/api/resources.api";

const createResource = useCreateResourceMutation();
const name = ref("");
const email = ref("");

async function submit() {
  if (!name.value.trim()) return;
  await createResource.mutateAsync({ name: name.value, email: email.value || null });
  name.value = "";
  email.value = "";
}
```

### 3.7 Migrate `src/components/TaskCard.vue`

No code changes needed. The component already imports `useUnitStore` from `@/stores/unit`; the store is retained. Verify the import path is `@/stores/unit` (it is).

### 3.8 Verify & commit

- [ ] Run `pnpm build`.
- [ ] Run `pnpm test`.
- [ ] Spot-check `/projects`, `/resources`, `/kanban` in `pnpm dev`.
- [ ] Commit:

```bash
git add src/pages/projects/index.vue src/pages/resources/index.vue src/pages/kanban/index.vue src/components/ProjectForm.vue src/components/TaskForm.vue src/components/ResourceForm.vue
git commit -m "feat(pages): migrate projects, resources, kanban and forms to vue-query"
```

---

## Task 4: Batch 3 — allocations, dashboard, gantt, calendar + components

### 4.1 Migrate `src/pages/allocations/index.vue`

Replace the entire `<script setup>` block:

```ts
import { computed, ref } from "vue";
import { useListAllocationsQuery, useUpdateAllocationMutation, useDeleteAllocationMutation } from "@/services/api/allocations.api";
import { useListResourcesQuery } from "@/services/api/resources.api";
import { useProjectsStore } from "@/stores/projects";
import AllocationForm from "@/components/AllocationForm.vue";
import DateRangePicker from "@/components/DateRangePicker.vue";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { NumberField, NumberFieldContent, NumberFieldDecrement, NumberFieldIncrement, NumberFieldInput } from "@/components/ui/number-field";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { fmtDate, parseDateStrict } from "@/utils/date";
import type { AllocationView } from "@/types";

const allocationsQuery = useListAllocationsQuery(computed(() => projects.current));
const updateAllocation = useUpdateAllocationMutation();
const deleteAllocation = useDeleteAllocationMutation();
const resourcesQuery = useListResourcesQuery();
const projects = useProjectsStore();

// Edit modal state
const editVisible = ref(false);
const editingId = ref<number | null>(null);
const editDateRange = ref<[number, number]>([0, 0]);
const editPercent = ref(0.5);

function openEdit(row: AllocationView) {
  editingId.value = row.id;
  editDateRange.value = [parseDateStrict(row.start_date), parseDateStrict(row.end_date)];
  editPercent.value = row.percent;
  editVisible.value = true;
}

async function saveEdit() {
  if (editingId.value == null || projects.current == null) return;
  await updateAllocation.mutateAsync({
    id: editingId.value,
    start: fmtDate(editDateRange.value[0]),
    end: fmtDate(editDateRange.value[1]),
    percent: editPercent.value,
    projectId: projects.current,
  });
  editVisible.value = false;
}

// Delete confirmation state
const deleteVisible = ref(false);
const deleteTargetId = ref<number | null>(null);

function openDelete(row: AllocationView) {
  deleteTargetId.value = row.id;
  deleteVisible.value = true;
}

async function confirmDelete() {
  if (deleteTargetId.value == null || projects.current == null) return;
  await deleteAllocation.mutateAsync({ id: deleteTargetId.value, projectId: projects.current });
  deleteVisible.value = false;
  deleteTargetId.value = null;
}

const tableData = computed(() => allocationsQuery.data.value ?? []);
```

Template: replace `allocations.items` with `tableData`. Remove `resources.load()` onMounted.

### 4.2 Migrate `src/components/AllocationForm.vue`

Replace the entire `<script setup>` block:

```ts
import { computed, ref } from "vue";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  NumberField,
  NumberFieldContent,
  NumberFieldDecrement,
  NumberFieldIncrement,
  NumberFieldInput,
} from "@/components/ui/number-field";
import { Badge } from "@/components/ui/badge";
import DateRangePicker from "@/components/DateRangePicker.vue";
import { useCreateAllocationMutation } from "@/services/api/allocations.api";
import { useListResourcesQuery } from "@/services/api/resources.api";
import { useListTasksQuery } from "@/services/api/tasks.api";
import { useResourceSummaryQuery } from "@/services/api/workload.api";
import { useProjectsStore } from "@/stores/projects";
import { fmtDate, parseDateStrict } from "@/utils/date";

const createAllocation = useCreateAllocationMutation();
const resourcesQuery = useListResourcesQuery();
const projects = useProjectsStore();
const tasksQuery = useListTasksQuery(computed(() => projects.current));

const resourceId = ref<number | null>(null);
const taskId = ref<number | null>(null);
const dateRange = ref<[number, number]>([parseDateStrict("2026-06-29"), parseDateStrict("2026-07-03")]);
const percent = ref(0.5);
const impact = ref<{ utilization: number; overloaded: boolean } | null>(null);
const error = ref<string | null>(null);

const resourceOptions = computed(() =>
  (resourcesQuery.data.value ?? []).map((r) => ({ label: r.name, value: r.id })),
);
const taskOptions = computed(() =>
  (tasksQuery.data.value ?? []).map((t) => ({ label: t.title, value: t.id })),
);

const resourceIdSelect = computed<string | number | undefined>({
  get: () => resourceId.value ?? undefined,
  set: (v) => { resourceId.value = v == null ? null : Number(v); },
});

const taskIdSelect = computed<string | number | undefined>({
  get: () => taskId.value ?? undefined,
  set: (v) => { taskId.value = v == null ? null : Number(v); },
});

const startStr = computed(() => fmtDate(dateRange.value[0]));
const endStr = computed(() => fmtDate(dateRange.value[1]));
const resourceSummaryQuery = useResourceSummaryQuery(resourceId, startStr, endStr);

async function submit() {
  error.value = null;
  if (resourceId.value == null || taskId.value == null || projects.current == null) return;
  const start = startStr.value;
  const end = endStr.value;
  try {
    await createAllocation.mutateAsync({
      resourceId: resourceId.value,
      taskId: taskId.value,
      start,
      end,
      percent: percent.value,
      projectId: projects.current,
    });
    const s = resourceSummaryQuery.data.value;
    impact.value = s ? { utilization: s.utilization, overloaded: s.overloaded } : null;
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}
```

Remove the old `watch(() => projects.current, ...)` and `resources.load()` calls.

### 4.3 Migrate `src/pages/dashboard/index.vue`

This is the most complex page. Replace the entire `<script setup>` block:

```ts
import { computed, ref, watch } from "vue";
import { useQueries } from "@tanstack/vue-query";
import { useApiFetch } from "@/services/fetch";
import { useListResourcesQuery } from "@/services/api/resources.api";
import { useListTeamsQuery } from "@/services/api/teams.api";
import { useListProjectsQuery } from "@/services/api/projects.api";
import {
  useProjectBurnQuery,
  useTeamSummaryQuery,
  useOverloadsQuery,
} from "@/services/api/workload.api";
import { useGetThresholdsQuery } from "@/services/api/config.api";
import { useGetTeamOverrideQuery } from "@/services/api/teams.api";
import { useProjectsStore } from "@/stores/projects";
import { useUnitStore } from "@/stores/unit";
import { parseDateStrict } from "@/utils/date";
import type { ResourceSummary } from "@/types";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Skeleton } from "@/components/ui/skeleton";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { TriangleAlertIcon, RefreshCwIcon, BriefcaseIcon, TrendingUpIcon, PanelRightCloseIcon, PanelRightOpenIcon } from "@lucide/vue";
import DateRangePicker from "@/components/DateRangePicker.vue";
import { fmtDate } from "@/utils/date";

const { apiFetch } = useApiFetch();
const resourcesQuery = useListResourcesQuery();
const projectsQuery = useListProjectsQuery();
const teamsQuery = useListTeamsQuery();
const thresholdsQuery = useGetThresholdsQuery();
const unit = useUnitStore();
const projects = useProjectsStore();

const dateRange = ref<[number, number]>([parseDateStrict("2026-06-29"), parseDateStrict("2026-07-03")]);
const selectedTeam = ref<number | null>(null);
const allTeamsValue = "__all__";
const rightPanelOpen = ref(true);
const activeTab = ref("overview");

const tabs = [
  { label: "Overview", value: "overview" },
  { label: "Analytics", value: "analytics", disabled: true },
  { label: "Reports", value: "reports", disabled: true },
  { label: "Notifications", value: "notifications", disabled: true },
];

const startStr = computed(() => fmtDate(dateRange.value[0]));
const endStr = computed(() => fmtDate(dateRange.value[1]));

const teamOptions = computed(() => [
  { label: "全部团队", value: allTeamsValue },
  ...(teamsQuery.data.value ?? []).map((t) => ({ label: t.name, value: String(t.id) })),
]);

const selectedTeamValue = computed(() =>
  selectedTeam.value == null ? allTeamsValue : String(selectedTeam.value),
);

const selectedTeamLabel = computed(
  () => teamOptions.value.find((o) => o.value === selectedTeamValue.value)?.label ?? "全部团队",
);

const resourceNameById = computed(() => {
  const map = new Map<number, string>();
  for (const r of resourcesQuery.data.value ?? []) {
    map.set(r.id, r.name);
  }
  return map;
});

function resourceLabel(id: number): string {
  return resourceNameById.value.get(id) ?? `资源 #${id}`;
}

const resourceSummaryQueries = useQueries({
  queries: computed(() => {
    const start = startStr.value;
    const end = endStr.value;
    const resources = resourcesQuery.data.value ?? [];
    return resources.map((r) => ({
      queryKey: ["workload-resource", r.id, start, end],
      queryFn: () =>
        apiFetch<ResourceSummary>(
          `/api/workload/resources/${r.id}?start=${encodeURIComponent(start)}&end=${encodeURIComponent(end)}`,
        ),
      enabled: () => !!start && !!end,
    }));
  }),
});

const resourceSummaries = computed<ResourceSummary[]>(() =>
  resourceSummaryQueries.value.filter((q) => q.isSuccess && q.data != null).map((q) => q.data!),
);

const averageUtilization = computed(() => {
  if (!resourceSummaries.value.length) return 0;
  const total = resourceSummaries.value.reduce((sum, s) => sum + s.utilization, 0);
  return total / resourceSummaries.value.length;
});

const overloadsQuery = useOverloadsQuery(startStr, endStr);
const projectBurnQuery = useProjectBurnQuery(computed(() => projects.current));
const teamSummaryQuery = useTeamSummaryQuery(computed(() => selectedTeam.value), startStr, endStr);
const teamOverrideQuery = useGetTeamOverrideQuery(computed(() => selectedTeam.value));

const loading = ref(false);

async function refresh() {
  loading.value = true;
  try {
    await Promise.all([
      resourcesQuery.refetch(),
      projectsQuery.refetch(),
      teamsQuery.refetch(),
      thresholdsQuery.refetch(),
      overloadsQuery.refetch(),
      projectBurnQuery.refetch(),
      teamSummaryQuery.refetch(),
      teamOverrideQuery.refetch(),
      ...resourceSummaryQueries.value.map((q) => q.refetch()),
    ]);
  } finally {
    loading.value = false;
  }
}

watch(
  () => teamOverrideQuery.data.value,
  (ov) => {
    unit.applyTeamOverride(ov?.pm_workdays ?? null);
  },
  { immediate: true },
);

async function updateSelectedTeam(value: unknown) {
  const str = String(value);
  selectedTeam.value = str === allTeamsValue ? null : Number(str);
  await refresh();
}
```

Template changes:
- Keep the manual refresh button bound to `refresh()` and the `loading` ref. The `loading` references in the template remain unchanged.
- Replace `wl.resourceSummaries` with `resourceSummaries`.
- Replace `wl.overloads` with `overloadsQuery.data.value ?? []`.
- Replace `wl.projectBurn` with `projectBurnQuery.data.value`.
- Replace `wl.teamSummary` with `teamSummaryQuery.data.value`.
- Remove the old local `fmtDate` function; the imported `fmtDate` from `@/utils/date` is used.

### 4.4 Migrate `src/pages/gantt/index.vue`

Replace the entire `<script setup>` block:

```ts
import { computed, ref, watch } from "vue";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { useGanttProjectQuery, useGanttResourceQuery, useDependenciesForProjectQuery } from "@/services/api/gantt.api";
import { useUpdateAllocationMutation } from "@/services/api/allocations.api";
import { useListProjectsQuery } from "@/services/api/projects.api";
import { useListResourcesQuery } from "@/services/api/resources.api";
import { useProjectsStore } from "@/stores/projects";
import GanttTimeline from "@/components/GanttTimeline.vue";

const projects = useProjectsStore();
const projectsQuery = useListProjectsQuery();
const resourcesQuery = useListResourcesQuery();
const updateAllocation = useUpdateAllocationMutation();
const mode = ref<"project" | "resource">("project");
const focusId = ref<number | null>(null);
const err = ref<string | null>(null);
const start = ref("2026-06-29");
const end = ref("2026-08-09");
const resourceSelect = ref<number | null>(null);

const projectOptions = computed(() =>
  (projectsQuery.data.value ?? []).map((p) => ({ label: p.name, value: p.id })),
);
const resourceOptions = computed(() =>
  (resourcesQuery.data.value ?? []).map((r) => ({ label: r.name, value: r.id })),
);

const ganttProjectQuery = useGanttProjectQuery(computed(() => (mode.value === "project" ? focusId.value : null)));
const ganttResourceQuery = useGanttResourceQuery(computed(() => (mode.value === "resource" ? focusId.value : null)));
const depsQuery = useDependenciesForProjectQuery(computed(() => (mode.value === "project" ? focusId.value : null)));

const bars = computed(() => {
  if (mode.value === "resource") return ganttResourceQuery.data.value ?? [];
  return ganttProjectQuery.data.value ?? [];
});
const deps = computed(() => depsQuery.data.value ?? []);
const activeError = computed(() => ganttProjectQuery.error ?? ganttResourceQuery.error ?? depsQuery.error);

watch(
  () => activeError.value,
  (e) => {
    err.value = e ? (e instanceof Error ? e.message : String(e)) : null;
  },
);

watch(
  () => projects.current,
  (id) => {
    if (mode.value === "project" && id != null) {
      focusId.value = id;
    }
  },
  { immediate: true },
);

async function onProjectChange(value: unknown) {
  const id = value as number | undefined;
  if (id != null) {
    projects.select(id);
    mode.value = "project";
    focusId.value = id;
  }
}

async function onResource(value: unknown) {
  const id = value as number | undefined;
  if (id == null) return;
  mode.value = "resource";
  focusId.value = id;
  resourceSelect.value = id;
}

function toProjectMode() {
  mode.value = "project";
  if (projects.current != null) focusId.value = projects.current;
}

async function onBarUpdate(id: number, start: string, end: string, percent: number) {
  await updateAllocation.mutateAsync({ id, start, end, percent });
}
```

Template changes:
- Bind `GanttTimeline` with `:bars="bars" :deps="deps" @update="onBarUpdate"`.
- Bind the alert to `err`.

### 4.5 Migrate `src/components/GanttTimeline.vue`

This component needs to stop reading from the gantt store and accept `bars` and `deps` as props. It emits an event for drag/resize. Replace the `<script setup>` block:

```ts
import { computed, ref } from "vue";
import { fmtDate, parseDateStrict } from "../utils/date";
import type { GanttBar, DepEdge } from "../types";

const DAY_W = 28; // px per day
const props = defineProps<{ start: string; end: string; bars: GanttBar[]; deps: DepEdge[] }>();
const emit = defineEmits<{
  (e: "update", id: number, start: string, end: string, percent: number): void;
}>();

const startMs = computed(() => parseDateStrict(props.start));
const totalDays = computed(() => Math.max(1, Math.round((parseDateStrict(props.end) - startMs.value) / 86400000) + 1));
const days = computed(() => {
  const out: string[] = [];
  const d = new Date(startMs.value);
  for (let i = 0; i < totalDays.value; i++) { out.push(fmtDate(d.getTime())); d.setDate(d.getDate() + 1); }
  return out;
});

function dayIndexOf(dateStr: string) { return Math.round((parseDateStrict(dateStr) - startMs.value) / 86400000); }
function barLeft(b: GanttBar) { return dayIndexOf(b.start_date) * DAY_W; }
function barWidth(b: GanttBar) { return (dayIndexOf(b.end_date) - dayIndexOf(b.start_date) + 1) * DAY_W; }

const rows = computed(() => {
  const m = new Map<number, { resource_id: number; resource_name: string; bars: GanttBar[] }>();
  for (const b of props.bars) {
    if (!m.has(b.resource_id)) m.set(b.resource_id, { resource_id: b.resource_id, resource_name: b.resource_name, bars: [] });
    m.get(b.resource_id)!.bars.push(b);
  }
  return [...m.values()];
});

type Drag = { id: number; mode: "move" | "resize"; startX: number; origStart: string; origEnd: string; percent: number };
const drag = ref<Drag | null>(null);
const previewDelta = ref(0);

function toStr(ms: number) { return fmtDate(ms); }
function onDown(e: PointerEvent, b: GanttBar, mode: "move" | "resize") {
  const target = e.target as HTMLElement;
  if (mode === "resize" && !target.classList.contains("gantt-timeline__resize")) return;
  (e.target as HTMLElement).setPointerCapture(e.pointerId);
  drag.value = { id: b.allocation_id, mode, startX: e.clientX, origStart: b.start_date, origEnd: b.end_date, percent: b.percent };
  previewDelta.value = 0;
}
function onMove(e: PointerEvent) {
  if (!drag.value) return;
  previewDelta.value = Math.round((e.clientX - drag.value.startX) / DAY_W);
}
function onUp() {
  const d = drag.value; if (!d) return;
  const deltaMs = previewDelta.value * 86400000;
  const newStart = d.mode === "move" ? toStr(parseDateStrict(d.origStart) + deltaMs) : d.origStart;
  const newEnd = toStr(parseDateStrict(d.origEnd) + deltaMs);
  drag.value = null; previewDelta.value = 0;
  if ((newStart !== d.origStart || newEnd !== d.origEnd) && newStart <= newEnd) {
    emit("update", d.id, newStart, newEnd, d.percent);
  }
}

type Arrow = { x1: number; y1: number; x2: number; y2: number };
const arrows = computed<Arrow[]>(() => {
  const pos = new Map<number, { startX: number; endX: number; y: number; startMs: number }>();
  let rowIdx = 0;
  for (const r of rows.value) {
    for (const b of r.bars) {
      const left = barLeft(b);
      const startMs = parseDateStrict(b.start_date);
      const prev = pos.get(b.task_id);
      if (!prev || startMs < prev.startMs) {
        pos.set(b.task_id, { startX: left, endX: left + barWidth(b), y: rowIdx * 32 + 16, startMs });
      }
    }
    rowIdx++;
  }
  const out: Arrow[] = [];
  for (const e of props.deps) {
    const p = pos.get(e.predecessor_id); const s = pos.get(e.task_id);
    if (p && s) out.push({ x1: p.endX, y1: p.y, x2: s.startX, y2: s.y });
  }
  return out;
});
```

### 4.6 Migrate `src/pages/calendar/index.vue`

Replace the entire `<script setup>` block:

```ts
import { useListResourcesQuery } from "@/services/api/resources.api";
import WorkWeekEditor from "@/components/WorkWeekEditor.vue";
import HolidayList from "@/components/HolidayList.vue";
import TimeOffList from "@/components/TimeOffList.vue";

const resourcesQuery = useListResourcesQuery();
```

Template change: pass resources to `TimeOffList`:

```vue
<TimeOffList :resources="resourcesQuery.data.value ?? []" />
```

### 4.7 Migrate `src/components/WorkWeekEditor.vue`

Replace the entire `<script setup>` block:

```ts
import { computed } from "vue";
import { Button } from "@/components/ui/button";
import { useListWorkWeeksQuery, useSetGlobalWorkWeekMutation } from "@/services/api/calendar.api";

const weekQuery = useListWorkWeeksQuery();
const setWeekMutation = useSetGlobalWorkWeekMutation();

const labels = ["一", "二", "三", "四", "五", "六", "日"];

const week = computed(() => {
  const rows = weekQuery.data.value ?? [];
  const global = rows.find((r) => r.scope === "global");
  if (!global) return [1, 1, 1, 1, 1, 0, 0];
  const f = (bit: number, frac: number) => (bit ? frac : 0);
  return [
    f(global.mon, global.mon_frac),
    f(global.tue, global.tue_frac),
    f(global.wed, global.wed_frac),
    f(global.thu, global.thu_frac),
    f(global.fri, global.fri_frac),
    f(global.sat, global.sat_frac),
    f(global.sun, global.sun_frac),
  ];
});

async function cycle(i: number) {
  const cur = week.value[i];
  const next = cur >= 1 ? 0 : cur >= 0.5 ? 1 : 0.5;
  const w = [...week.value];
  w[i] = next;
  await setWeekMutation.mutateAsync(w);
}

function dayType(f: number): "default" | "secondary" | "outline" {
  if (f === 0) return "outline";
  if (f === 0.5) return "secondary";
  return "default";
}
```

### 4.8 Migrate `src/components/HolidayList.vue`

Replace the entire `<script setup>` block:

```ts
import { computed, ref } from "vue";
import { CalendarDate, getLocalTimeZone } from "@internationalized/date";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Calendar } from "@/components/ui/calendar";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { useListHolidaysQuery, useAddHolidayMutation, useDeleteHolidayMutation } from "@/services/api/calendar.api";
import { fmtDate } from "@/utils/date";
import type { DateValue } from "@internationalized/date";

const holidaysQuery = useListHolidaysQuery();
const addHoliday = useAddHolidayMutation();
const deleteHoliday = useDeleteHolidayMutation();

const day = ref<number | null>(null);
const frac = ref(1);
const name = ref("");

const fracOptions = [
  { label: "全天", value: 1 },
  { label: "半天", value: 0.5 },
];

const dayDate = computed<DateValue | undefined>({
  get() {
    if (day.value == null) return undefined;
    const [y, m, d] = fmtDate(day.value).split("-").map(Number);
    return new CalendarDate(y, m, d);
  },
  set(date) {
    day.value = date?.toDate(getLocalTimeZone()).getTime() ?? null;
  },
});

function onSelectFrac(value: unknown) {
  frac.value = Number(value);
}

async function add() {
  if (day.value == null) return;
  await addHoliday.mutateAsync({ projectId: null, day: fmtDate(day.value), fraction: frac.value, name: name.value || null });
  day.value = null;
  name.value = "";
}

const confirmOpen = ref<Record<number, boolean>>({});
```

Template changes: replace `cal.holidays` with `holidaysQuery.data.value ?? []` and `cal.removeHoliday(h.id)` with `deleteHoliday.mutate(h.id)`.

### 4.9 Migrate `src/components/TimeOffList.vue`

Add a `resources` prop and replace store access with the prop and queries. Replace the entire `<script setup>` block:

```ts
import { computed, ref } from "vue";
import { CalendarIcon } from "@lucide/vue";
import { CalendarDate, type DateValue } from "@internationalized/date";
import { useListTimeOffQuery, useAddTimeOffMutation, useDeleteTimeOffMutation } from "@/services/api/calendar.api";
import { fmtDate, parseDate } from "@/utils/date";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Calendar } from "@/components/ui/calendar";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { Resource } from "@/types";

const props = defineProps<{ resources: Resource[] }>();

const timeOffQuery = useListTimeOffQuery();
const addTimeOff = useAddTimeOffMutation();
const deleteTimeOff = useDeleteTimeOffMutation();

const rid = ref<number | null>(null);
const day = ref<number | null>(null);
const frac = ref(1);
const reason = ref("");

const resourceOptions = computed(() => props.resources.map((r) => ({ label: r.name, value: r.id })));
const fracOptions = [
  { label: "全天", value: 1 },
  { label: "半天", value: 0.5 },
];

function resourceName(id: number): string {
  return props.resources.find((r) => r.id === id)?.name ?? `#${id}`;
}

function toDateValue(ms: number): DateValue {
  const s = fmtDate(ms);
  const [year, month, dayOfMonth] = s.split("-").map(Number);
  return new CalendarDate(year, month, dayOfMonth);
}
function fromDateValue(dv: DateValue): number {
  return (
    parseDate(`${dv.year}-${String(dv.month).padStart(2, "0")}-${String(dv.day).padStart(2, "0")}`) ??
    Date.now()
  );
}

const dateValue = computed<DateValue | undefined>({
  get: () => (day.value == null ? undefined : toDateValue(day.value)),
  set: (dv) => {
    day.value = dv ? fromDateValue(dv) : null;
  },
});

const dateDisplay = computed(() =>
  day.value == null ? "选择日期" : fmtDate(day.value),
);

function updateRid(value: unknown) {
  rid.value = typeof value === "number" ? value : null;
}

function updateFrac(value: unknown) {
  frac.value = typeof value === "number" ? value : 1;
}

async function add() {
  if (rid.value == null || day.value == null) return;
  await addTimeOff.mutateAsync({ resourceId: rid.value, day: fmtDate(day.value), fraction: frac.value, reason: reason.value || null });
  day.value = null;
  reason.value = "";
}
```

Template changes: replace `cal.timeOff` with `timeOffQuery.data.value ?? []` and `cal.removeTimeOff(t.id)` with `deleteTimeOff.mutate(t.id)`.

### 4.10 Migrate `src/pages/calendar-grid/index.vue`

Replace the entire `<script setup>` block:

```ts
import { computed, ref, watch } from "vue";
import { Button } from "@/components/ui/button";
import DateRangePicker from "@/components/DateRangePicker.vue";
import { useListResourcesQuery } from "@/services/api/resources.api";
import { useDailyOccupancyQuery } from "@/services/api/gantt.api";
import OccupancyGrid from "@/components/OccupancyGrid.vue";
import { fmtDate, parseDateStrict } from "@/utils/date";

const resourcesQuery = useListResourcesQuery();
const dateRange = ref<[number, number]>([parseDateStrict("2026-06-29"), parseDateStrict("2026-07-12")]);
const days = ref<string[]>([]);

const startStr = computed(() => fmtDate(dateRange.value[0]));
const endStr = computed(() => fmtDate(dateRange.value[1]));
const occupancyQuery = useDailyOccupancyQuery(startStr, endStr);

function buildDays() {
  const out: string[] = [];
  const d0 = new Date(fmtDate(dateRange.value[0]) + "T00:00:00");
  const d1 = new Date(fmtDate(dateRange.value[1]) + "T00:00:00");
  for (let d = new Date(d0); d <= d1; d.setDate(d.getDate() + 1)) {
    out.push(fmtDate(d.getTime()));
  }
  days.value = out;
}

function refresh() {
  buildDays();
  occupancyQuery.refetch();
}

watch(dateRange, () => {
  buildDays();
}, { immediate: true });
```

Template changes: replace `items` with `occupancyQuery.data.value ?? []`, `resources.items` with `resourcesQuery.data.value ?? []`, and remove the `wl.loadThresholds()` call.

### 4.11 Migrate `src/components/OccupancyGrid.vue`

Move the `band` function from the workload store into this component. Replace the entire `<script setup>` block:

```ts
import type { DayOccupancy } from "../types";
const props = defineProps<{ items: DayOccupancy[]; days: string[]; resources: { id: number; name: string }[] }>();

function cell(rid: number, day: string) {
  return props.items.find((o) => o.resource_id === rid && o.date === day);
}

function band(util: number): "under" | "green" | "yellow" | "red" {
  // Hardcoded thresholds matching the old workload store defaults.
  if (util >= 1.1) return "red";
  if (util >= 1.0) return "yellow";
  if (util >= 0.7) return "green";
  return "under";
}

function bg(o?: DayOccupancy) {
  if (!o) return "#f7f7fa";
  const b = o.status ?? band(o.utilization);
  return ({ under: "#e0e0e6", green: "#9ad19a", yellow: "#f0d070", red: "#e08090" } as const)[b];
}
```

### 4.12 Verify & commit

- [ ] Run `pnpm build`.
- [ ] Run `pnpm test`.
- [ ] Spot-check `/allocations`, `/dashboard`, `/gantt`, `/calendar`, `/calendar-grid` in `pnpm dev`.
- [ ] Commit:

```bash
git add src/pages/allocations/index.vue src/pages/dashboard/index.vue src/pages/gantt/index.vue src/pages/calendar/index.vue src/pages/calendar-grid/index.vue src/components/AllocationForm.vue src/components/GanttTimeline.vue src/components/WorkWeekEditor.vue src/components/HolidayList.vue src/components/TimeOffList.vue src/components/OccupancyGrid.vue
git commit -m "feat(pages): migrate allocations, dashboard, gantt, calendar and components to vue-query"
```

---

## Task 5: Batch 4 — teams, ai, reports, AppLayout

### 5.1 Add missing `useGetOptimizationRunQuery` to `src/services/api/optimization.api.ts`

Append this function to the end of the file (after the existing imports; add `MaybeRef` import at the top):

```ts
import { type MaybeRef, computed, toValue } from "vue";

export function useGetOptimizationRunQuery(runId: MaybeRef<number | null>) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(runId));
  return useQuery<RunResult>({
    queryKey: computed(() => ["optimization-run", id.value]),
    queryFn: () => apiFetch<RunResult>(`/api/optimization/runs/${id.value}`),
    enabled: () => id.value != null,
  });
}
```

### 5.2 Migrate `src/pages/teams/index.vue`

Replace the entire `<script setup>` block:

```ts
import { computed, ref, watch } from "vue";
import { PlusIcon } from "@lucide/vue";
import {
  useListTeamsQuery,
  useCreateTeamMutation,
  useDeleteTeamMutation,
  useListTeamMembersQuery,
  useAddTeamMemberMutation,
  useRemoveTeamMemberMutation,
  useSetTeamOverrideMutation,
  useGetTeamOverrideQuery,
} from "@/services/api/teams.api";
import { useListResourcesQuery } from "@/services/api/resources.api";
import type { TeamOverride } from "@/types";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  NumberField,
  NumberFieldContent,
  NumberFieldDecrement,
  NumberFieldIncrement,
  NumberFieldInput,
} from "@/components/ui/number-field";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import ListPage from "@/components/list/ListPage.vue";
import ListRowActions from "@/components/list/ListRowActions.vue";
import ListToolbar from "@/components/list/ListToolbar.vue";

const teamsQuery = useListTeamsQuery();
const createTeam = useCreateTeamMutation();
const deleteTeam = useDeleteTeamMutation();
const resourcesQuery = useListResourcesQuery();

const teamName = ref("");
const filterName = ref("");
const selectedTeam = ref<number | null>(null);
const memberResource = ref<number | null>(null);
const memberRole = ref("");

const overrideOverload = ref<number | null>(null);
const overrideUnderload = ref<number | null>(null);
const overrideGreen = ref<number | null>(null);
const overrideYellow = ref<number | null>(null);
const overridePdHours = ref<number | null>(null);
const overridePmWorkdays = ref<number | null>(null);

const deleteDialogOpen = ref(false);
const deleteTargetId = ref<number | null>(null);
const deleteTargetName = computed(() => (teamsQuery.data.value ?? []).find((t) => t.id === deleteTargetId.value)?.name ?? "");
const removeDialogOpen = ref(false);
const removeTargetId = ref<number | null>(null);
const removeTargetName = computed(() => resourceName(removeTargetId.value ?? 0));

const resourceOptions = computed(() =>
  (resourcesQuery.data.value ?? []).map((r) => ({ label: r.name, value: r.id })),
);

const selectedTeamName = computed(() =>
  (teamsQuery.data.value ?? []).find((t) => t.id === selectedTeam.value)?.name ?? null,
);

const filteredTeams = computed(() => {
  return (teamsQuery.data.value ?? []).filter((t) => {
    if (!filterName.value) return true;
    return t.name.toLowerCase().includes(filterName.value.toLowerCase());
  });
});

const teamMembersQuery = useListTeamMembersQuery(computed(() => selectedTeam.value));
const teamOverrideQuery = useGetTeamOverrideQuery(computed(() => selectedTeam.value));
const addTeamMember = useAddTeamMemberMutation();
const removeTeamMember = useRemoveTeamMemberMutation();
const setTeamOverride = useSetTeamOverrideMutation();

watch(
  () => teamOverrideQuery.data.value,
  (existing) => {
    overrideOverload.value = existing?.overload_threshold ?? null;
    overrideUnderload.value = existing?.underload_threshold ?? null;
    overrideGreen.value = existing?.utilization_green ?? null;
    overrideYellow.value = existing?.utilization_yellow ?? null;
    overridePdHours.value = existing?.pd_hours ?? null;
    overridePmWorkdays.value = existing?.pm_workdays ?? null;
  },
  { immediate: true },
);

async function createTeam() {
  if (!teamName.value.trim()) return;
  await createTeam.mutateAsync({ name: teamName.value, description: null });
  teamName.value = "";
}

async function addMember() {
  if (selectedTeam.value == null || memberResource.value == null) return;
  await addTeamMember.mutateAsync({ teamId: selectedTeam.value, resourceId: memberResource.value, role: memberRole.value || null });
  memberResource.value = null;
  memberRole.value = "";
}

async function removeMember(resourceId: number) {
  if (selectedTeam.value == null) return;
  await removeTeamMember.mutateAsync({ teamId: selectedTeam.value, resourceId });
}

async function saveOverride() {
  if (selectedTeam.value == null) return;
  const override: TeamOverride = {
    team_id: selectedTeam.value,
    pd_hours: overridePdHours.value,
    pm_workdays: overridePmWorkdays.value,
    overload_threshold: overrideOverload.value,
    underload_threshold: overrideUnderload.value,
    utilization_green: overrideGreen.value,
    utilization_yellow: overrideYellow.value,
  };
  await setTeamOverride.mutateAsync(override);
}

function resourceName(id: number): string {
  return (resourcesQuery.data.value ?? []).find((r) => r.id === id)?.name ?? `#${id}`;
}

function openDeleteDialog(id: number) {
  deleteTargetId.value = id;
  deleteDialogOpen.value = true;
}

async function confirmDelete() {
  if (deleteTargetId.value == null) return;
  await deleteTeam.mutateAsync(deleteTargetId.value);
  deleteDialogOpen.value = false;
  deleteTargetId.value = null;
}

function openRemoveDialog(resourceId: number) {
  removeTargetId.value = resourceId;
  removeDialogOpen.value = true;
}

async function confirmRemove() {
  if (removeTargetId.value == null) return;
  await removeMember(removeTargetId.value);
  removeDialogOpen.value = false;
  removeTargetId.value = null;
}
```

Template changes: replace `teams.items` with `teamsQuery.data.value ?? []`, `teams.members` with `teamMembersQuery.data.value ?? []`, `resources.items` with `resourcesQuery.data.value ?? []`.

### 5.3 Migrate `src/components/WeightsPanel.vue`

Replace the entire `<script setup>` block:

```ts
import { Slider } from "@/components/ui/slider";
import type { ObjectiveWeights } from "@/types";

const weights = defineModel<ObjectiveWeights>({ required: true });

type WeightKey = "skill_fit" | "balance" | "budget";
const labels: WeightKey[] = ["skill_fit", "balance", "budget"];
const cn: Record<WeightKey, string> = { skill_fit: "技能最优", balance: "负载均衡", budget: "预算" };

function updateWeight(key: WeightKey, value: number) {
  const next = { ...weights.value, [key]: value };
  const s = next.skill_fit + next.balance + next.budget;
  if (s > 0) {
    next.skill_fit /= s;
    next.balance /= s;
    next.budget /= s;
  }
  weights.value = next;
}
```

Template update: replace the `@update:model-value` handler with a direct call:

```vue
<Slider
  :model-value="[weights[k]]"
  :min="0"
  :max="1"
  :step="0.05"
  class="w-40"
  @update:model-value="(v: number[] | undefined) => { if (v && v[0] !== undefined) updateWeight(k, v[0]); }"
/>
```

### 5.4 Migrate `src/components/PlanReview.vue`

Replace the entire `<script setup>` block:

```ts
import { computed } from "vue";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { RunResult, ScoredAssignment } from "@/types";

const props = defineProps<{ run: RunResult }>();
const emit = defineEmits<{
  (e: "accept", runId: number): void;
  (e: "reject", runId: number): void;
}>();

function pct(v: number) { return Math.round(v) + "%"; }

interface StatItem { label: string; value: string; }
const stats = computed<StatItem[]>(() => [
  { label: "综合评分", value: pct(props.run.plan.solution.metrics.overall) },
  { label: "技能", value: pct(props.run.plan.solution.metrics.skill_fit) },
  { label: "排期覆盖", value: pct(props.run.plan.solution.metrics.scheduled_ratio) },
]);

function assignmentKey(a: ScoredAssignment, i: number) {
  return `${a.resource_id}-${a.task_id}-${a.start}-${a.end}-${i}`;
}
```

Template changes: replace all `opt.current` with `run` and bind the buttons to emit events:

```vue
<Button @click="emit('accept', run.run_id)">✓ 采纳（写入分配）</Button>
<Button variant="destructive" @click="emit('reject', run.run_id)">✗ 拒绝</Button>
```

### 5.5 Migrate `src/pages/ai/index.vue`

Replace the entire `<script setup>` block:

```ts
import { computed, ref, watch } from "vue";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  useRunOptimizationMutation,
  useListOptimizationRunsQuery,
  useGetOptimizationRunQuery,
  useApplySolutionMutation,
  useRejectSolutionMutation,
} from "@/services/api/optimization.api";
import { useProjectsStore } from "@/stores/projects";
import WeightsPanel from "@/components/WeightsPanel.vue";
import PlanReview from "@/components/PlanReview.vue";
import type { ObjectiveWeights, RunResult } from "@/types";

const projects = useProjectsStore();
const runsQuery = useListOptimizationRunsQuery(null);
const runOptimization = useRunOptimizationMutation();
const applySolution = useApplySolutionMutation();
const rejectSolution = useRejectSolutionMutation();

const weights = ref<ObjectiveWeights>({ skill_fit: 0.4, balance: 0.4, budget: 0.2 });
const currentRun = ref<RunResult | null>(null);
const viewingRunId = ref<number | null>(null);
const page = ref(1);
const pageSize = ref(10);

const viewRunQuery = useGetOptimizationRunQuery(viewingRunId);

watch(() => viewRunQuery.data.value, (run) => {
  if (run) currentRun.value = run;
});

const totalPages = computed(() =>
  Math.max(1, Math.ceil((runsQuery.data.value?.length ?? 0) / pageSize.value)),
);

const displayedRows = computed(() => {
  const rows = runsQuery.data.value ?? [];
  const start = (page.value - 1) * pageSize.value;
  return rows.slice(start, start + pageSize.value);
});

async function runForCurrent() {
  if (projects.current == null) return;
  const result = await runOptimization.mutateAsync({ projectId: projects.current, weights: weights.value });
  currentRun.value = result;
  viewingRunId.value = null;
}

async function loadRun(id: number) {
  viewingRunId.value = id;
  // The watch on viewRunQuery.data will set currentRun when data arrives.
}

async function accept(runId: number) {
  await applySolution.mutateAsync(runId);
  currentRun.value = null;
  viewingRunId.value = null;
}

async function reject(runId: number) {
  await rejectSolution.mutateAsync(runId);
  currentRun.value = null;
  viewingRunId.value = null;
}

function setPage(n: number) {
  page.value = Math.max(1, Math.min(n, totalPages.value));
}

function setPageSize(n: number) {
  pageSize.value = Math.max(1, n);
  page.value = 1;
}
```

Template changes:
- Replace `<WeightsPanel />` with `<WeightsPanel v-model="weights" />`.
- Replace `<PlanReview v-if="opt.current" />` with `<PlanReview v-if="currentRun" :run="currentRun" @accept="accept" @reject="reject" />`.
- Replace `opt.history.rows` with `displayedRows`, `opt.history.total` with `runsQuery.data.value?.length ?? 0`, `opt.page` with `page`, `opt.pageSize` with `pageSize`.
- Replace `opt.loadRun(row.id)` with `loadRun(row.id)`.
- Replace `opt.busy` with `runOptimization.isPending`.
- Replace `opt.setPage`/`opt.setPageSize` with local `setPage`/`setPageSize`.

### 5.6 Migrate `src/pages/reports/index.vue`

Replace the entire `<script setup>` block:

```ts
import { computed, ref, watch } from "vue";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import DateRangePicker from "@/components/DateRangePicker.vue";
import { useGetReportCatalogQuery, reportKinds, exportReport, exportSnapshot, type ReportKind } from "@/services/api/reports.api";
import { useApiFetch } from "@/services/fetch";
import { useListProjectsQuery } from "@/services/api/projects.api";
import { fmtDate, parseDateStrict } from "@/utils/date";

const { apiFetch } = useApiFetch();
const projectsQuery = useListProjectsQuery();
const catalogQuery = useGetReportCatalogQuery();
const kind = ref<ReportKind>("ResourceUtilization");
const dateRange = ref<[number, number]>([parseDateStrict("2026-06-29"), parseDateStrict("2026-07-12")]);
const fmt = ref<string>("csv");
const projectId = ref<number | null>(null);
const msg = ref("");
const busy = ref(false);
const allProjectsValue = "__all__";

const cn: Record<ReportKind, string> = {
  ResourceUtilization: "资源利用率",
  TeamUtilization: "团队利用率",
  ProjectBurn: "项目预算消耗",
  AiDecisions: "AI 决策记录",
  Cost: "成本",
};

const kindOptions = reportKinds.map((k) => ({ label: cn[k], value: k }));
const projectOptions = computed(() => [
  { label: "全部项目", value: allProjectsValue },
  ...(projectsQuery.data.value ?? []).map((p) => ({ label: p.name, value: String(p.id) })),
]);
const projectValue = computed(() => projectId.value == null ? allProjectsValue : String(projectId.value));

const fmtOptions = computed(() => {
  const entry = catalogQuery.data.value?.find((e) => e.kind === kind.value);
  const formats = entry?.formats ?? ["csv", "xlsx"];
  return formats.map((f) => ({ label: f.toUpperCase(), value: f }));
});

const acceptsProject = computed(() => {
  const entry = catalogQuery.data.value?.find((e) => e.kind === kind.value);
  return entry?.accepts_project_id ?? false;
});

watch(kind, () => {
  const formats = fmtOptions.value.map((o) => o.value);
  if (!formats.includes(fmt.value)) fmt.value = formats[0] ?? "csv";
});

async function doExport() {
  busy.value = true;
  msg.value = "";
  try {
    const start = fmtDate(dateRange.value[0]);
    const end = fmtDate(dateRange.value[1]);
    const pid = acceptsProject.value ? projectId.value : null;
    const ok = await exportReport(apiFetch, kind.value, pid, start, end, fmt.value as "csv" | "xlsx" | "pdf");
    msg.value = ok ? `已导出 ${kind.value}.${fmt.value}` : "导出失败";
  } catch (e: unknown) {
    msg.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}

async function doSnapshot() {
  busy.value = true;
  msg.value = "";
  try {
    const start = fmtDate(dateRange.value[0]);
    const end = fmtDate(dateRange.value[1]);
    const ok = await exportSnapshot(apiFetch, start, end);
    msg.value = ok ? "已导出快照 JSON" : "导出失败";
  } catch (e: unknown) {
    msg.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}

function updateProject(value: unknown) {
  const s = String(value);
  projectId.value = s === allProjectsValue ? null : Number(s);
}
```

Template: no field changes; the `projects.items` references are now `projectsQuery.data.value ?? []`.

### 5.7 Migrate `src/layouts/default.vue` (AppLayout)

Replace the entire `<script setup>` block:

```ts
import { computed, watch } from "vue";
import { useRoute, RouterLink } from "vue-router";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarInset,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
  SidebarRail,
  SidebarTrigger,
} from "@/components/ui/sidebar";
import {
  LayoutDashboardIcon,
  KanbanIcon,
  FolderKanbanIcon,
  UsersIcon,
  TagsIcon,
  UsersRoundIcon,
  ListChecksIcon,
  CalendarIcon,
  BarChart3Icon,
  LayoutGridIcon,
  SparklesIcon,
  FileTextIcon,
  SettingsIcon,
} from "@lucide/vue";
import { useProjectsStore } from "@/stores/projects";
import { useUnitStore } from "@/stores/unit";
import { useListProjectsQuery } from "@/services/api/projects.api";
import { useListSkillsQuery, useListTagsQuery } from "@/services/api/catalog.api";
import { useGetUnitConfigQuery } from "@/services/api/config.api";

const projects = useProjectsStore();
const unit = useUnitStore();
const route = useRoute();

const projectsQuery = useListProjectsQuery();
const skillsQuery = useListSkillsQuery();
const tagsQuery = useListTagsQuery();
const unitConfigQuery = useGetUnitConfigQuery();

const ready = computed(() =>
  projectsQuery.isSuccess && skillsQuery.isSuccess && tagsQuery.isSuccess && unitConfigQuery.isSuccess,
);

watch(() => projectsQuery.data.value, (items) => {
  if (projects.current == null && items && items.length > 0) {
    projects.select(items[0].id);
  }
});

const navItems = [
  { to: "/dashboard", label: "仪表盘 Dashboard", icon: LayoutDashboardIcon },
  { to: "/kanban", label: "看板 Kanban", icon: KanbanIcon },
  { to: "/projects", label: "项目 Projects", icon: FolderKanbanIcon },
  { to: "/resources", label: "资源 Resources", icon: UsersIcon },
  { to: "/teams", label: "团队 Teams", icon: UsersRoundIcon },
  { to: "/allocations", label: "分配 Allocations", icon: ListChecksIcon },
  { to: "/calendar", label: "日历 Calendar", icon: CalendarIcon },
  { to: "/gantt", label: "甘特图 Gantt", icon: BarChart3Icon },
  { to: "/calendar-grid", label: "占用网格 Calendar Grid", icon: LayoutGridIcon },
  { to: "/catalog", label: "技能标签 Catalog", icon: TagsIcon },
  { to: "/ai", label: "AI 优化 Optimization", icon: SparklesIcon },
  { to: "/reports", label: "报表 Reports", icon: FileTextIcon },
  { to: "/settings", label: "设置 Settings", icon: SettingsIcon },
];

const activePath = computed(() => route.path);

const projectOptions = computed(() =>
  (projectsQuery.data.value ?? []).map((p) => ({ label: p.name, value: String(p.id) })),
);

function onProjectChange(value: unknown) {
  const id = Number(value);
  if (!Number.isNaN(id)) projects.select(id);
}
```

Template changes: none needed; `projectOptions` is now computed from the query.

### 5.8 Verify & commit

- [ ] Run `pnpm build`.
- [ ] Run `pnpm test`.
- [ ] Spot-check `/teams`, `/ai`, `/reports` and AppLayout bootstrap in `pnpm dev`.
- [ ] Commit:

```bash
git add src/services/api/optimization.api.ts src/pages/teams/index.vue src/components/WeightsPanel.vue src/components/PlanReview.vue src/pages/ai/index.vue src/pages/reports/index.vue src/layouts/default.vue
git commit -m "feat(pages): migrate teams, ai, reports and AppLayout to vue-query"
```

---

## Task 6: Final verification

- [ ] Run `pnpm build` and confirm zero TypeScript errors.
- [ ] Run `pnpm test` and confirm all tests pass (ignore any remaining old store tests that are removed in 3c).
- [ ] Run `pnpm dev` and smoke-test every route: `/dashboard`, `/kanban`, `/projects`, `/resources`, `/teams`, `/allocations`, `/calendar`, `/gantt`, `/calendar-grid`, `/catalog`, `/ai`, `/reports`, `/settings`.
- [ ] Verify that no page/component still imports a data store (except thin `useProjectsStore`/`useUnitStore`):

```bash
grep -r "useAllocationsStore\|useCatalogStore\|useTasksStore\|useWorkloadStore\|useGanttStore\|useCalendarStore\|useOptimizationStore\|useTeamsStore\|useResourcesStore\|useSettingsStore" src/pages/ src/components/ || true
```

Expected: empty output (or only matches in comment/docs).

- [ ] Verify no direct `@/api` imports remain:

```bash
grep -r 'from "@/api"' src/pages/ src/components/ || true
```

Expected: empty output.

- [ ] Verify no `useRefreshStore` watchers remain:

```bash
grep -r "useRefreshStore" src/pages/ src/components/ || true
```

Expected: empty output.

- [ ] Verify AppLayout has no manual bootstrap loop:

```bash
grep -n "for (let i = 0; i < 40" src/layouts/default.vue || true
```

Expected: empty output.

---

## Task 7: PR preparation

This plan spans 5 commits/tasks. Recommended PR strategy:

1. **PR #3b-1** — Task 0 + Task 1: API `MaybeRef`/`enabled` + thin stores.
2. **PR #3b-2** — Task 2: catalog + settings.
3. **PR #3b-3** — Task 3: projects, resources, kanban + forms.
4. **PR #3b-4** — Task 4: allocations, dashboard, gantt, calendar + components.
5. **PR #3b-5** — Task 5: teams, ai, reports, AppLayout.

Each PR can be reviewed and merged independently. After the last PR merges, stage 3b is complete and stage 3c can begin (deleting old data stores + refresh bus).

- [ ] Push branch(es) and open PR(s) with the verification output pasted into the description.

---

## Self-review checklist

| Spec requirement | Task that implements it |
|---|---|
| 13 pages + 12 components move to `services/api/` composables | Tasks 2–5 |
| `useProjectsStore` thinned to `current` + `select` | Task 1.1 |
| `useUnitStore` thinned to `unit` + `formatPd` + `applyTeamOverride` | Task 1.2 |
| AppLayout bootstrap uses vue-query | Task 5.7 |
| All `useRefreshStore` watchers removed | Tasks 2–5 (no refresh store imports) |
| No direct `@/api` imports in pages/components | Tasks 2–5 |
| snake_case preserved in templates/types | All tasks (no template field changes) |
| Derived data moved into pages (kanban columns, utilization band) | Task 3.3, Task 4.11 |
| Build + test pass | Tasks 0–5 verification steps |

Known simplifications carried forward (as documented in spec):
- `formatPd` uses hardcoded `20` for PM conversion; real `pm_workdays` from `useGetUnitConfigQuery` is deferred to 3c/6.
- `OccupancyGrid` uses hardcoded default thresholds; real thresholds from `useGetThresholdsQuery` deferred to 3c/6.
- AI history is paginated client-side because `useListOptimizationRunsQuery` returns all rows.

No placeholders remain in this plan.
