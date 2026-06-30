<script setup lang="ts">
import { ref } from "vue";
import { NH2, NList, NListItem, NThing, NText, NTag, NPopconfirm, NButton, NSpace, NModal, NForm, NFormItem, NInput, NDatePicker, NInputNumber, NEmpty, NSelect } from "naive-ui";
import ResourceForm from "../components/ResourceForm.vue";
import { useResourcesStore } from "../stores/resources";
import { useCatalogStore } from "../stores/catalog";
import { onMounted } from "vue";
import type { Resource, ResourceSkill, ResourceTag } from "../types";

const resources = useResourcesStore();
const catalog = useCatalogStore();
onMounted(() => { resources.load(); catalog.load(); });

// Edit modal state
const editVisible = ref(false);
const editingId = ref<number | null>(null);
const editName = ref("");
const editEmail = ref("");
const editAvailFrom = ref<number | null>(null);
const editAvailTo = ref<number | null>(null);
const editCapacity = ref<number | null>(null);
const editRate = ref<number | null>(null);
// Skills: selected skill ids with per-skill proficiency (1..5).
const editSkills = ref<{ skillId: number; proficiency: number }[]>([]);
// Tags: selected tag ids.
const editTags = ref<number[]>([]);

const skillOptions = () => catalog.skills.map(s => ({ label: s.name, value: s.id }));
const tagOptions = () => catalog.tags.map(t => ({ label: t.name, value: t.id }));

function updateSelectedSkills(ids: number[]) {
  editSkills.value = ids.map((id) => {
    const existing = editSkills.value.find((s) => s.skillId === id);
    return existing ?? { skillId: id, proficiency: 3 };
  });
}

// Display: per-resource skills/tags fetched lazily for the list.
const skillCache = ref<Record<number, ResourceSkill[]>>({});
const tagCache = ref<Record<number, ResourceTag[]>>({});
async function loadDisplay(r: Resource) {
  if (!skillCache.value[r.id]) skillCache.value[r.id] = await resources.loadSkills(r.id);
  if (!tagCache.value[r.id]) tagCache.value[r.id] = await resources.loadTags(r.id);
}

function fmtDate(ms: number): string {
  const d = new Date(ms);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}
function parseDate(s: string | null): number | null {
  if (!s) return null;
  return Date.parse(s);
}

async function openEdit(r: Resource) {
  editingId.value = r.id;
  editName.value = r.name;
  editEmail.value = r.email ?? "";
  editAvailFrom.value = parseDate(r.available_from);
  editAvailTo.value = parseDate(r.available_to);
  editCapacity.value = r.daily_capacity_pd;
  editRate.value = r.daily_rate_pd;
  // Load existing skills (with proficiency) + tags into the editor.
  const [skills, tags] = await Promise.all([resources.loadSkills(r.id), resources.loadTags(r.id)]);
  editSkills.value = skills.map(s => ({ skillId: s.skill_id, proficiency: s.proficiency }));
  editTags.value = tags.map(t => t.tag_id);
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
  await resources.saveSkills(editingId.value, editSkills.value.map(s => [s.skillId, s.proficiency]));
  await resources.saveTags(editingId.value, editTags.value);
  // Refresh display cache for this resource.
  delete skillCache.value[editingId.value];
  delete tagCache.value[editingId.value];
  editVisible.value = false;
}
</script>

<template>
  <n-h2>资源 / Resources</n-h2>
  <ResourceForm />
  <n-list v-if="resources.items.length" bordered hoverable>
    <n-list-item v-for="r in resources.items" :key="r.id" @mouseenter="loadDisplay(r)">
      <n-thing :title="r.name">
        <template #description>
          <n-space :size="4" align="center">
            <n-text v-if="r.email" depth="3" style="font-size: 12px">{{ r.email }}</n-text>
            <n-tag v-if="r.daily_capacity_pd" size="tiny" :bordered="false">{{ r.daily_capacity_pd }} PD/天</n-tag>
            <n-tag v-if="r.daily_rate_pd" size="tiny" :bordered="false" type="info">{{ r.daily_rate_pd }}/天</n-tag>
            <n-text v-if="r.available_from" depth="3" style="font-size: 12px">从 {{ r.available_from }}</n-text>
            <n-tag
              v-for="s in (skillCache[r.id] || [])"
              :key="'sk' + s.skill_id"
              size="tiny"
              :bordered="false"
              type="success"
            >{{ s.skill_name }} {{ s.proficiency }}</n-tag>
            <n-tag
              v-for="t in (tagCache[r.id] || [])"
              :key="'tg' + t.tag_id"
              size="tiny"
              :bordered="false"
              :color="{ color: t.color || undefined }"
            >{{ t.tag_name }}</n-tag>
          </n-space>
        </template>
        <template #action>
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
      <n-form-item label="技能">
        <n-space vertical style="width: 100%">
          <n-select
            multiple
            :options="skillOptions()"
            :value="editSkills.map(s => s.skillId)"
            placeholder="选择技能"
            @update:value="updateSelectedSkills"
          />
          <n-space v-if="editSkills.length" :size="4" vertical>
            <n-space v-for="s in editSkills" :key="s.skillId" align="center" :size="8">
              <n-text style="width: 90px; font-size: 12px">
                {{ catalog.skills.find(sk => sk.id === s.skillId)?.name ?? s.skillId }}
              </n-text>
              <n-input-number
                v-model:value="s.proficiency"
                :min="1"
                :max="5"
                :step="1"
                size="small"
                style="width: 90px"
              />
              <n-text depth="3" style="font-size: 11px">熟练度 1-5</n-text>
            </n-space>
          </n-space>
        </n-space>
      </n-form-item>
      <n-form-item label="标签">
        <n-select
          multiple
          :options="tagOptions()"
          v-model:value="editTags"
          placeholder="选择标签"
        />
      </n-form-item>
      <n-space justify="end">
        <n-button @click="editVisible = false">取消</n-button>
        <n-button type="primary" @click="saveEdit">保存</n-button>
      </n-space>
    </n-form>
  </n-modal>
</template>
