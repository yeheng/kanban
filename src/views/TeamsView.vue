<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { NH2, NH3, NSpace, NForm, NFormItem, NInput, NButton, NSelect, NList, NListItem, NThing, NTag, NPopconfirm, NText } from "naive-ui";
import { useTeamsStore } from "../stores/teams";
import { useResourcesStore } from "../stores/resources";

const teams = useTeamsStore();
const resources = useResourcesStore();
const teamName = ref("");
const selectedTeam = ref<number | null>(null);
const memberResource = ref<number | null>(null);
const memberRole = ref("");

const resourceOptions = computed(() =>
  resources.items.map((r) => ({ label: r.name, value: r.id })),
);

const teamOptions = computed(() =>
  teams.items.map((t) => ({ label: t.name, value: t.id })),
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
              {{ selectedTeam === t.id ? "已选中" : "查看成员" }}
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
  </template>
</template>
