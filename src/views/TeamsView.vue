<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { NH2, NH3, NSpace, NForm, NFormItem, NInput, NInputNumber, NButton, NSelect, NList, NListItem, NThing, NTag, NPopconfirm, NText, NDivider } from "naive-ui";
import { useTeamsStore } from "../stores/teams";
import { useResourcesStore } from "../stores/resources";
import type { TeamOverride } from "../types";

const teams = useTeamsStore();
const resources = useResourcesStore();
const teamName = ref("");
const selectedTeam = ref<number | null>(null);
const memberResource = ref<number | null>(null);
const memberRole = ref("");

// Override form state
const overrideOverload = ref<number | null>(null);
const overrideUnderload = ref<number | null>(null);
const overrideGreen = ref<number | null>(null);
const overrideYellow = ref<number | null>(null);

const resourceOptions = computed(() =>
  resources.items.map((r) => ({ label: r.name, value: r.id })),
);

const selectedTeamName = computed(() =>
  teams.items.find((t) => t.id === selectedTeam.value)?.name ?? null,
);

onMounted(async () => {
  await teams.load();
  await resources.load();
});

watch(selectedTeam, async (id) => {
  if (id != null) await teams.loadMembers(id);
  // Reset override form when switching teams
  overrideOverload.value = null;
  overrideUnderload.value = null;
  overrideGreen.value = null;
  overrideYellow.value = null;
});

async function createTeam() {
  if (!teamName.value.trim()) return;
  await teams.create(teamName.value, null);
  teamName.value = "";
}

async function addMember() {
  if (selectedTeam.value == null || memberResource.value == null) return;
  await teams.addMember(selectedTeam.value, memberResource.value, memberRole.value || null);
  memberResource.value = null;
  memberRole.value = "";
}

async function saveOverride() {
  if (selectedTeam.value == null) return;
  const override: TeamOverride = {
    team_id: selectedTeam.value,
    pd_hours: null,
    pm_workdays: null,
    overload_threshold: overrideOverload.value,
    underload_threshold: overrideUnderload.value,
    utilization_green: overrideGreen.value,
    utilization_yellow: overrideYellow.value,
  };
  await teams.setOverride(override);
}

function resourceName(id: number): string {
  return resources.items.find((r) => r.id === id)?.name ?? `#${id}`;
}
</script>

<template>
  <n-h2 style="margin-top: 0">团队 / Teams</n-h2>

  <n-h3>创建团队</n-h3>
  <n-form inline>
    <n-form-item label="团队名">
      <n-input v-model:value="teamName" placeholder="团队名" @keyup.enter="createTeam" />
    </n-form-item>
    <n-form-item>
      <n-button type="primary" @click="createTeam">创建团队</n-button>
    </n-form-item>
  </n-form>

  <n-h3>团队列表</n-h3>
  <n-list bordered>
    <n-list-item v-for="t in teams.items" :key="t.id">
      <n-thing :title="t.name" :description="t.description ?? ''">
        <template #suffix>
          <n-space :size="8">
            <n-button
              size="small"
              :type="selectedTeam === t.id ? 'primary' : 'default'"
              @click="selectedTeam = t.id"
            >
              {{ selectedTeam === t.id ? "已选中" : "管理" }}
            </n-button>
            <n-popconfirm @positive-click="teams.remove(t.id)">
              <template #trigger>
                <n-button size="small" type="error" quaternary>删除</n-button>
              </template>
              确定删除团队 "{{ t.name }}" 吗？
            </n-popconfirm>
          </n-space>
        </template>
      </n-thing>
    </n-list-item>
  </n-list>

  <template v-if="selectedTeam != null">
    <n-divider />
    <n-h3>{{ selectedTeamName }} — 成员管理</n-h3>
    <n-form inline>
      <n-form-item label="资源">
        <n-select
          v-model:value="memberResource"
          :options="resourceOptions"
          placeholder="选择资源"
          style="width: 200px"
        />
      </n-form-item>
      <n-form-item label="角色">
        <n-input v-model:value="memberRole" placeholder="角色 (可选)" style="width: 150px" />
      </n-form-item>
      <n-form-item>
        <n-button type="primary" @click="addMember">添加成员</n-button>
      </n-form-item>
    </n-form>

    <n-list bordered>
      <n-list-item v-for="m in teams.members" :key="`${m.team_id}-${m.resource_id}`">
        <n-thing :title="resourceName(m.resource_id)">
          <template #description>
            <n-tag v-if="m.role" size="small" :bordered="false">{{ m.role }}</n-tag>
            <n-text v-else depth="3" style="font-size: 12px">无角色</n-text>
          </template>
        </n-thing>
      </n-list-item>
    </n-list>
    <n-text v-if="!teams.members.length" depth="3">暂无成员。</n-text>

    <n-divider />
    <n-h3>团队阈值覆盖</n-h3>
    <n-text depth="3" style="font-size: 12px; margin-bottom: 8px; display: block">
      设置后该团队使用自己的阈值，覆盖全局设置。留空则不覆盖（使用全局值）。
    </n-text>
    <n-form inline>
      <n-form-item label="过载阈值">
        <n-input-number v-model:value="overrideOverload" :step="0.05" :min="0" placeholder="如 1.10" />
      </n-form-item>
      <n-form-item label="低载阈值">
        <n-input-number v-model:value="overrideUnderload" :step="0.05" :min="0" placeholder="如 0.70" />
      </n-form-item>
      <n-form-item label="绿灯利用率">
        <n-input-number v-model:value="overrideGreen" :step="0.05" :min="0" :max="1" placeholder="如 0.80" />
      </n-form-item>
      <n-form-item label="黄灯利用率">
        <n-input-number v-model:value="overrideYellow" :step="0.05" :min="0" :max="1" placeholder="如 0.95" />
      </n-form-item>
      <n-form-item>
        <n-button type="primary" @click="saveOverride">保存覆盖</n-button>
      </n-form-item>
    </n-form>
  </template>
</template>
