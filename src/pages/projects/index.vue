<script setup lang="ts">
import { computed, ref } from "vue";
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
import { Textarea } from "@/components/ui/textarea";
import DateRangePicker from "@/components/DateRangePicker.vue";
import ProjectForm from "@/components/ProjectForm.vue";
import TaskForm from "@/components/TaskForm.vue";
import { useProjectsStore } from "@/stores/projects";
import { useUnitStore } from "@/stores/unit";
import { fmtDate, parseDate } from "@/utils/date";
import type { Project } from "@/types";

const projects = useProjectsStore();
const unit = useUnitStore();

// Filters
const filterName = ref("");
const filterStatus = ref<string>("all");
const filterPriority = ref<string>("all");

const filteredProjects = computed(() => {
  return projects.items.filter((p) => {
    const matchesName = !filterName.value || p.name.toLowerCase().includes(filterName.value.toLowerCase());
    const matchesStatus = filterStatus.value === "all" || p.status === filterStatus.value;
    const matchesPriority = filterPriority.value === "all" || String(p.priority) === filterPriority.value;
    return matchesName && matchesStatus && matchesPriority;
  });
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
  await projects.update(editingId.value, {
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
  await projects.remove(deleteId.value);
  deleteId.value = null;
  deleteName.value = "";
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex flex-col gap-2 md:flex-row md:items-start md:justify-between">
      <div>
        <h1 class="text-2xl font-bold tracking-tight">项目 / Projects</h1>
        <p class="text-muted-foreground">管理项目并维护预算与优先级</p>
      </div>
    </div>

    <Card>
      <CardHeader>
        <CardTitle class="text-base">新建项目</CardTitle>
      </CardHeader>
      <CardContent>
        <ProjectForm />
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle class="text-base">项目列表</CardTitle>
      </CardHeader>
      <CardContent class="space-y-4">
        <div class="flex flex-col gap-3 md:flex-row md:items-end">
          <div class="grid gap-2 flex-1">
            <Label class="text-xs">搜索项目名</Label>
            <Input v-model="filterName" placeholder="输入项目名过滤" />
          </div>
          <div class="grid gap-2 w-full md:w-40">
            <Label class="text-xs">状态</Label>
            <Select v-model="filterStatus">
              <SelectTrigger>
                <SelectValue placeholder="全部状态" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部状态</SelectItem>
                <SelectItem value="active">active</SelectItem>
                <SelectItem value="done">done</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div class="grid gap-2 w-full md:w-40">
            <Label class="text-xs">优先级</Label>
            <Select v-model="filterPriority">
              <SelectTrigger>
                <SelectValue placeholder="全部优先级" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部优先级</SelectItem>
                <SelectItem v-for="n in 9" :key="n" :value="String(n)">{{ n }}</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        <div class="rounded-md border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>项目名</TableHead>
                <TableHead>状态</TableHead>
                <TableHead>优先级</TableHead>
                <TableHead>预算</TableHead>
                <TableHead class="hidden md:table-cell">周期</TableHead>
                <TableHead class="hidden lg:table-cell">描述</TableHead>
                <TableHead class="w-[180px] text-right">操作</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow
                v-for="p in filteredProjects"
                :key="p.id"
                :class="{ 'bg-muted/50': p.id === projects.current }"
              >
                <TableCell class="font-medium">
                  <button
                    class="text-left hover:underline"
                    :class="{ 'font-bold': p.id === projects.current }"
                    @click="projects.select(p.id)"
                  >
                    {{ p.name }}
                  </button>
                </TableCell>
                <TableCell>
                  <Badge :variant="p.status === 'done' ? 'secondary' : 'default'">
                    {{ p.status }}
                  </Badge>
                </TableCell>
                <TableCell>{{ p.priority }}</TableCell>
                <TableCell>{{ unit.formatPd(p.budget_pd) }}</TableCell>
                <TableCell class="hidden md:table-cell text-muted-foreground whitespace-nowrap">
                  <span v-if="p.start_date && p.end_date">
                    {{ p.start_date }} ~ {{ p.end_date }}
                  </span>
                  <span v-else>-</span>
                </TableCell>
                <TableCell class="hidden lg:table-cell max-w-xs truncate text-muted-foreground">
                  {{ p.description || "-" }}
                </TableCell>
                <TableCell class="text-right">
                  <div class="flex items-center justify-end gap-2">
                    <Button
                      v-if="p.status === 'active'"
                      size="sm"
                      variant="outline"
                      @click="projects.setStatus(p.id, 'done')"
                    >
                      完成
                    </Button>
                    <Button
                      v-else
                      size="sm"
                      variant="outline"
                      @click="projects.setStatus(p.id, 'active')"
                    >
                      激活
                    </Button>
                    <Button size="sm" variant="outline" @click="openEdit(p)">
                      编辑
                    </Button>
                    <Button
                      size="sm"
                      variant="destructive"
                      @click="confirmDelete(p.id, p.name)"
                    >
                      删除
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
              <TableRow v-if="!filteredProjects.length">
                <TableCell colspan="7" class="text-center text-muted-foreground py-6">
                  无匹配项目
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle class="text-base">在当前项目新建任务</CardTitle>
      </CardHeader>
      <CardContent>
        <TaskForm v-if="projects.current" />
        <span v-else class="text-muted-foreground">请先选择一个项目。</span>
      </CardContent>
    </Card>

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
  </div>
</template>
