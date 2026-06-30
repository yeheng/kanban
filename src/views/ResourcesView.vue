<script setup lang="ts">
import { ref } from "vue";
import { NH2, NList, NListItem, NThing, NText, NTag, NPopconfirm, NButton, NSpace, NModal, NForm, NFormItem, NInput, NDatePicker, NInputNumber, NEmpty } from "naive-ui";
import ResourceForm from "../components/ResourceForm.vue";
import { useResourcesStore } from "../stores/resources";
import { onMounted } from "vue";
import type { Resource } from "../types";

const resources = useResourcesStore();
onMounted(() => resources.load());

// Edit modal state
const editVisible = ref(false);
const editingId = ref<number | null>(null);
const editName = ref("");
const editEmail = ref("");
const editAvailFrom = ref<number | null>(null);
const editAvailTo = ref<number | null>(null);
const editCapacity = ref<number | null>(null);
const editRate = ref<number | null>(null);

function fmtDate(ms: number): string {
  const d = new Date(ms);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}
function parseDate(s: string | null): number | null {
  if (!s) return null;
  return Date.parse(s);
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
  await resources.update(editingId.value, {
    name: editName.value,
    email: editEmail.value || null,
    availableFrom: editAvailFrom.value != null ? fmtDate(editAvailFrom.value) : null,
    availableTo: editAvailTo.value != null ? fmtDate(editAvailTo.value) : null,
    dailyCapacityPd: editCapacity.value,
    dailyRatePd: editRate.value,
  });
  editVisible.value = false;
}
</script>

<template>
  <n-h2>资源 / Resources</n-h2>
  <ResourceForm />
  <n-list v-if="resources.items.length" bordered hoverable>
    <n-list-item v-for="r in resources.items" :key="r.id">
      <n-thing :title="r.name">
        <template #description>
          <n-space :size="4" align="center">
            <n-text v-if="r.email" depth="3" style="font-size: 12px">{{ r.email }}</n-text>
            <n-tag v-if="r.daily_capacity_pd" size="tiny" :bordered="false">{{ r.daily_capacity_pd }} PD/天</n-tag>
            <n-tag v-if="r.daily_rate_pd" size="tiny" :bordered="false" type="info">{{ r.daily_rate_pd }}/天</n-tag>
            <n-text v-if="r.available_from" depth="3" style="font-size: 12px">从 {{ r.available_from }}</n-text>
          </n-space>
        </template>
        <template #suffix>
          <n-space :size="4">
            <n-button size="small" @click="openEdit(r)">编辑</n-button>
            <n-popconfirm @positive-click="resources.remove(r.id)">
              <template #trigger>
                <n-button size="small" type="error" quaternary>删除</n-button>
              </template>
              确定删除资源 "{{ r.name }}" 吗？
            </n-popconfirm>
          </n-space>
        </template>
      </n-thing>
    </n-list-item>
  </n-list>
  <n-empty v-else description="暂无资源" />

  <n-modal v-model:show="editVisible" preset="card" title="编辑资源" style="width: 520px">
    <n-form>
      <n-form-item label="姓名">
        <n-input v-model:value="editName" />
      </n-form-item>
      <n-form-item label="邮箱">
        <n-input v-model:value="editEmail" placeholder="email (可选)" />
      </n-form-item>
      <n-form-item label="可用起始日">
        <n-date-picker v-model:value="editAvailFrom" type="date" clearable />
      </n-form-item>
      <n-form-item label="可用截止日">
        <n-date-picker v-model:value="editAvailTo" type="date" clearable />
      </n-form-item>
      <n-form-item label="日容量 (PD)">
        <n-input-number v-model:value="editCapacity" :min="0" :step="0.5" placeholder="如 1.0" />
      </n-form-item>
      <n-form-item label="日费率">
        <n-input-number v-model:value="editRate" :min="0" :step="100" placeholder="如 800" />
      </n-form-item>
      <n-space justify="end">
        <n-button @click="editVisible = false">取消</n-button>
        <n-button type="primary" @click="saveEdit">保存</n-button>
      </n-space>
    </n-form>
  </n-modal>
</template>
