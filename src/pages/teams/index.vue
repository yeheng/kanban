<script setup lang="ts">
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
const createTeamMutation = useCreateTeamMutation();
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
  await createTeamMutation.mutateAsync({ name: teamName.value, description: null });
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
  if (selectedTeam.value === deleteTargetId.value) {
    selectedTeam.value = null;
  }
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
</script>

<template>
  <ListPage title="团队 / Teams" description="管理团队、成员与阈值覆盖">
    <template #actions>
      <Button @click="selectedTeam = null">
        <PlusIcon class="mr-2 h-4 w-4" />
        新建团队
      </Button>
    </template>

    <Card>
      <CardHeader class="pb-3">
        <CardTitle class="text-base">团队列表</CardTitle>
      </CardHeader>
      <CardContent class="space-y-4">
        <ListToolbar
          v-model:search="filterName"
          search-placeholder="搜索团队名..."
        />

        <div class="rounded-md border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>团队名</TableHead>
                <TableHead class="hidden md:table-cell">描述</TableHead>
                <TableHead class="w-[60px]" />
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow
                v-for="t in filteredTeams"
                :key="t.id"
                :class="{ 'bg-muted/50': selectedTeam === t.id }"
              >
                <TableCell class="font-medium">
                  <button
                    class="text-left hover:underline"
                    :class="{ 'font-bold': selectedTeam === t.id }"
                    @click="selectedTeam = t.id"
                  >
                    {{ t.name }}
                  </button>
                </TableCell>
                <TableCell class="hidden md:table-cell text-muted-foreground">
                  {{ t.description || "-" }}
                </TableCell>
                <TableCell class="text-right">
                  <ListRowActions
                    @edit="selectedTeam = t.id"
                    @delete="openDeleteDialog(t.id)"
                  />
                </TableCell>
              </TableRow>
              <TableRow v-if="!filteredTeams.length">
                <TableCell colspan="3" class="text-center text-muted-foreground py-6">
                  无匹配团队
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>

    <template v-if="selectedTeam != null">
      <Card>
        <CardHeader class="pb-3">
          <CardTitle class="text-base">{{ selectedTeamName }} — 成员管理</CardTitle>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex flex-col gap-3 md:flex-row md:items-end">
            <div class="grid gap-2 flex-1">
              <Label>资源</Label>
              <Select
                :model-value="memberResource ?? undefined"
                @update:model-value="(v) => memberResource = (v as number | undefined) ?? null"
              >
                <SelectTrigger>
                  <SelectValue placeholder="选择资源" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem v-for="o in resourceOptions" :key="o.value" :value="o.value">
                    {{ o.label }}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div class="grid gap-2 md:w-48">
              <Label>角色</Label>
              <Input v-model="memberRole" placeholder="角色 (可选)" />
            </div>
            <div class="flex items-end">
              <Button @click="addMember">添加成员</Button>
            </div>
          </div>

          <div class="rounded-md border">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>成员</TableHead>
                  <TableHead>角色</TableHead>
                  <TableHead class="w-[60px]" />
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow v-for="m in teamMembersQuery.data.value ?? []" :key="`${m.team_id}-${m.resource_id}`">
                  <TableCell class="font-medium">{{ resourceName(m.resource_id) }}</TableCell>
                  <TableCell>
                    <Badge v-if="m.role" variant="secondary">{{ m.role }}</Badge>
                    <span v-else class="text-xs text-muted-foreground">无角色</span>
                  </TableCell>
                  <TableCell class="text-right">
                    <ListRowActions @delete="openRemoveDialog(m.resource_id)" />
                  </TableCell>
                </TableRow>
                <TableRow v-if="!(teamMembersQuery.data.value ?? []).length">
                  <TableCell colspan="3" class="text-center text-muted-foreground py-6">
                    暂无成员
                  </TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader class="pb-3">
          <CardTitle class="text-base">团队阈值覆盖</CardTitle>
        </CardHeader>
        <CardContent class="space-y-4">
          <p class="text-xs text-muted-foreground">
            设置后该团队使用自己的阈值，覆盖全局设置。留空则不覆盖（使用全局值）。
          </p>
          <div class="flex flex-wrap items-end gap-4">
            <div class="grid gap-2">
              <Label>每 PD 小时</Label>
              <NumberField
                :model-value="overridePdHours ?? undefined"
                :step="0.5"
                :min="0.5"
                @update:model-value="(v) => overridePdHours = (v as number | undefined) ?? null"
              >
                <NumberFieldContent>
                  <NumberFieldDecrement />
                  <NumberFieldInput />
                  <NumberFieldIncrement />
                </NumberFieldContent>
              </NumberField>
            </div>
            <div class="grid gap-2">
              <Label>每 PM 人日</Label>
              <NumberField
                :model-value="overridePmWorkdays ?? undefined"
                :step="1"
                :min="1"
                @update:model-value="(v) => overridePmWorkdays = (v as number | undefined) ?? null"
              >
                <NumberFieldContent>
                  <NumberFieldDecrement />
                  <NumberFieldInput />
                  <NumberFieldIncrement />
                </NumberFieldContent>
              </NumberField>
            </div>
            <div class="grid gap-2">
              <Label>过载阈值</Label>
              <NumberField
                :model-value="overrideOverload ?? undefined"
                :step="0.05"
                :min="0"
                @update:model-value="(v) => overrideOverload = (v as number | undefined) ?? null"
              >
                <NumberFieldContent>
                  <NumberFieldDecrement />
                  <NumberFieldInput />
                  <NumberFieldIncrement />
                </NumberFieldContent>
              </NumberField>
            </div>
            <div class="grid gap-2">
              <Label>低载阈值</Label>
              <NumberField
                :model-value="overrideUnderload ?? undefined"
                :step="0.05"
                :min="0"
                @update:model-value="(v) => overrideUnderload = (v as number | undefined) ?? null"
              >
                <NumberFieldContent>
                  <NumberFieldDecrement />
                  <NumberFieldInput />
                  <NumberFieldIncrement />
                </NumberFieldContent>
              </NumberField>
            </div>
            <div class="grid gap-2">
              <Label>绿灯利用率</Label>
              <NumberField
                :model-value="overrideGreen ?? undefined"
                :step="0.05"
                :min="0"
                :max="1"
                @update:model-value="(v) => overrideGreen = (v as number | undefined) ?? null"
              >
                <NumberFieldContent>
                  <NumberFieldDecrement />
                  <NumberFieldInput />
                  <NumberFieldIncrement />
                </NumberFieldContent>
              </NumberField>
            </div>
            <div class="grid gap-2">
              <Label>黄灯利用率</Label>
              <NumberField
                :model-value="overrideYellow ?? undefined"
                :step="0.05"
                :min="0"
                :max="1"
                @update:model-value="(v) => overrideYellow = (v as number | undefined) ?? null"
              >
                <NumberFieldContent>
                  <NumberFieldDecrement />
                  <NumberFieldInput />
                  <NumberFieldIncrement />
                </NumberFieldContent>
              </NumberField>
            </div>
            <div class="flex items-end">
              <Button @click="saveOverride">保存覆盖</Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </template>

    <template v-else>
      <Card>
        <CardHeader>
          <CardTitle class="text-base">新建团队</CardTitle>
        </CardHeader>
        <CardContent>
          <div class="flex flex-wrap items-end gap-4">
            <div class="grid gap-2 flex-1">
              <Label>团队名</Label>
              <Input v-model="teamName" placeholder="团队名" @keyup.enter="createTeam" />
            </div>
            <div class="flex items-end">
              <Button @click="createTeam">创建团队</Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </template>

    <Dialog v-model:open="deleteDialogOpen">
      <DialogContent class="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>删除团队</DialogTitle>
          <DialogDescription>确定删除团队 "{{ deleteTargetName }}" 吗？</DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" @click="deleteDialogOpen = false">取消</Button>
          <Button variant="destructive" @click="confirmDelete">确定</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <Dialog v-model:open="removeDialogOpen">
      <DialogContent class="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>移除成员</DialogTitle>
          <DialogDescription>确定将 "{{ removeTargetName }}" 移出团队吗？</DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" @click="removeDialogOpen = false">取消</Button>
          <Button variant="destructive" @click="confirmRemove">确定</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </ListPage>
</template>
