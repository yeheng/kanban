<script setup lang="ts">
import { ref } from "vue";
import { NH2, NH3, NList, NListItem, NThing, NTag, NDivider, NText, NPopconfirm, NButton, NSpace, NModal, NForm, NFormItem, NInput, NInputNumber, NDatePicker, NEmpty } from "naive-ui";
import ProjectForm from "../components/ProjectForm.vue";
import TaskForm from "../components/TaskForm.vue";
import { useProjectsStore } from "../stores/projects";
import { useUnitStore } from "../stores/unit";
import type { Project } from "../types";

const projects = useProjectsStore();
const unit = useUnitStore();

// Edit modal state
const editVisible = ref(false);
const editingId = ref<number | null>(null);
const editName = ref("");
const editPriority = ref(5);
const editBudget = ref(0);
const editDateRange = ref<[number, number] | null>(null);
const editDescription = ref("");

function fmtDate(ms: number): string {
  const d = new Date(ms);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}
function parseDate(s: string | null): number | null {
  if (!s) return null;
  return Date.parse(s);
}

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
</script>

<template>
  <n-h2>项目 / Projects</n-h2>
  <ProjectForm />
  <n-list v-if="projects.items.length" bordered hoverable>
    <n-list-item v-for="p in projects.items" :key="p.id">
      <n-thing>
        <template #header>
          <n-text :strong="p.id === projects.current" @click="projects.select(p.id)" style="cursor: pointer">
            {{ p.name }}
          </n-text>
        </template>
        <template #description>
          <n-space :size="4" align="center">
            <n-tag size="small" :bordered="false" :type="p.status === 'done' ? 'warning' : 'success'">
              {{ p.status }}
            </n-tag>
            <n-tag size="small" :bordered="false">优先级 {{ p.priority }}</n-tag>
            <n-tag size="small" :bordered="false" type="info">预算 {{ unit.formatPd(p.budget_pd) }}</n-tag>
          </n-space>
        </template>
        <template #action>
          <n-space :size="4">
            <n-button
              v-if="p.status === 'active'"
              size="small"
              @click="projects.setStatus(p.id, 'done')"
            >完成</n-button>
            <n-button
              v-else
              size="small"
              @click="projects.setStatus(p.id, 'active')"
            >激活</n-button>
            <n-button size="small" @click="openEdit(p)">编辑</n-button>
            <n-popconfirm @positive-click="projects.remove(p.id)">
              <template #trigger>
                <n-button size="small" type="error" quaternary>删除</n-button>
              </template>
              确定删除项目 "{{ p.name }}" 吗？
            </n-popconfirm>
          </n-space>
        </template>
      </n-thing>
    </n-list-item>
  </n-list>
  <n-empty v-else description="暂无项目" />

  <n-divider />
  <n-h3>在当前项目新建任务</n-h3>
  <TaskForm v-if="projects.current" />
  <n-text v-else depth="3">请先选择一个项目。</n-text>

  <n-modal v-model:show="editVisible" preset="card" title="编辑项目" style="width: 520px">
    <n-form>
      <n-form-item label="项目名">
        <n-input v-model:value="editName" />
      </n-form-item>
      <n-form-item label="描述">
        <n-input v-model:value="editDescription" type="textarea" :rows="2" placeholder="项目描述 (可选)" />
      </n-form-item>
      <n-form-item label="优先级">
        <n-input-number v-model:value="editPriority" :min="1" :max="9" />
      </n-form-item>
      <n-form-item label="预算 PD">
        <n-input-number v-model:value="editBudget" :min="0" />
      </n-form-item>
      <n-form-item label="项目周期">
        <n-date-picker v-model:value="editDateRange" type="daterange" clearable />
      </n-form-item>
      <n-space justify="end">
        <n-button @click="editVisible = false">取消</n-button>
        <n-button type="primary" @click="saveEdit">保存</n-button>
      </n-space>
    </n-form>
  </n-modal>
</template>
