<script setup lang="ts">
import type { ColumnDef } from "@tanstack/vue-table";
import { computed, h, ref } from "vue";
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
import { Textarea } from "@/components/ui/textarea";
import DateRangePicker from "@/components/DateRangePicker.vue";
import { BasicPage } from "@/components/global-layout";
import {
  DataTable,
  DataTableColumnHeader,
  useGenerateVueTable,
} from "@/components/data-table";
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

const loading = computed(() => projectsQuery.isLoading.value);

function rowClass(p: Project) {
  return p.id === projects.current ? "bg-muted/50" : "";
}

// Column definitions
// Cast generic component so h() can infer props without manual type args.
const ColumnHeader = DataTableColumnHeader as any;

const columns: ColumnDef<Project>[] = [
  {
    accessorKey: "name",
    header: ({ column }) => h(ColumnHeader, { column, title: "项目名" }),
    cell: ({ row }) =>
      h(
        "button",
        {
          class: [
            "text-left hover:underline",
            row.original.id === projects.current ? "font-bold" : "font-medium",
          ],
          onClick: () => projects.select(row.original.id),
        },
        row.original.name,
      ),
  },
  {
    accessorKey: "status",
    header: ({ column }) => h(ColumnHeader, { column, title: "状态" }),
    cell: ({ row }) =>
      h(
        Badge,
        { variant: row.original.status === "done" ? "secondary" : "default" },
        () => row.original.status,
      ),
  },
  {
    accessorKey: "priority",
    header: ({ column }) => h(ColumnHeader, { column, title: "优先级" }),
    cell: ({ row }) => row.original.priority,
  },
  {
    accessorKey: "budget_pd",
    header: ({ column }) => h(ColumnHeader, { column, title: "预算" }),
    cell: ({ row }) => unit.formatPd(row.original.budget_pd),
  },
  {
    id: "period",
    header: ({ column }) => h(ColumnHeader, { column, title: "周期" }),
    cell: ({ row }) => {
      const p = row.original;
      return p.start_date && p.end_date ? `${p.start_date} ~ ${p.end_date}` : "-";
    },
  },
  {
    accessorKey: "description",
    header: ({ column }) => h(ColumnHeader, { column, title: "描述" }),
    cell: ({ row }) => row.original.description || "-",
  },
  {
    id: "actions",
    cell: ({ row }) =>
      h(ListRowActions, {
        onEdit: () => openEdit(row.original),
        onDelete: () => confirmDelete(row.original.id, row.original.name),
      }),
  },
];

const table = useGenerateVueTable<Project>({
  columns,
  get data() {
    return filteredProjects.value;
  },
});

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
</script>

<template>
  <BasicPage title="项目 / Projects" description="管理项目并维护预算与优先级">
    <template #actions>
      <Button @click="createVisible = true">
        <PlusIcon class="mr-2 h-4 w-4" />
        新建项目
      </Button>
    </template>

    <DataTable
      :table="table"
      :columns="columns"
      :loading="loading"
      :row-class="rowClass"
    >
      <template #toolbar>
        <ListToolbar
          v-model:search="filterName"
          v-model:filter="filterStatus"
          :show-reset="isFiltered"
          search-placeholder="搜索项目名..."
          filter-label="状态"
          :filter-options="statusOptions"
          @reset="resetFilters"
        />
      </template>
    </DataTable>

    <Card>
      <CardHeader>
        <CardTitle class="text-base">在当前项目新建任务</CardTitle>
      </CardHeader>
      <CardContent>
        <TaskForm v-if="projects.current" />
        <span v-else class="text-muted-foreground">请先选择一个项目。</span>
      </CardContent>
    </Card>

    <!-- Create project dialog -->
    <Dialog v-model:open="createVisible">
      <DialogContent class="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>新建项目</DialogTitle>
        </DialogHeader>
        <CardContent class="pt-4">
          <ProjectForm />
        </CardContent>
      </DialogContent>
    </Dialog>

    <!-- Edit project dialog -->
    <Dialog v-model:open="editVisible">
      <DialogContent class="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>编辑项目</DialogTitle>
        </DialogHeader>
        <div class="grid gap-4 py-4">
          <div class="grid gap-2">
            <Label>项目名</Label>
            <Input v-model="editName" />
          </div>
          <div class="grid gap-2">
            <Label>描述</Label>
            <Textarea
              v-model="editDescription"
              :rows="2"
              placeholder="项目描述 (可选)"
            />
          </div>
          <div class="grid gap-2">
            <Label>优先级</Label>
            <NumberField v-model="editPriority" :min="1" :max="9">
              <NumberFieldContent>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
              </NumberFieldContent>
            </NumberField>
          </div>
          <div class="grid gap-2">
            <Label>预算 PD</Label>
            <NumberField v-model="editBudget" :min="0">
              <NumberFieldContent>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
              </NumberFieldContent>
            </NumberField>
          </div>
          <div class="grid gap-2">
            <Label>项目周期</Label>
            <div v-if="editDateRange" class="flex items-center gap-2">
              <DateRangePicker
                :model-value="editDateRange"
                class="flex-1"
                @update:model-value="editDateRange = $event as [number, number]"
              />
              <Button
                variant="ghost"
                size="sm"
                @click="editDateRange = null"
              >
                清除
              </Button>
            </div>
            <Button
              v-else
              variant="outline"
              class="w-auto justify-start"
              @click="editDateRange = [todayMs, todayMs]"
            >
              选择日期范围
            </Button>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" @click="editVisible = false">
            取消
          </Button>
          <Button @click="saveEdit">保存</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Delete confirmation dialog -->
    <Dialog v-model:open="deleteOpen">
      <DialogContent class="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>确认删除</DialogTitle>
          <DialogDescription>
            确定删除项目 "{{ deleteName }}" 吗？
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" @click="deleteId = null">取消</Button>
          <Button variant="destructive" @click="doDelete">删除</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </BasicPage>
</template>
