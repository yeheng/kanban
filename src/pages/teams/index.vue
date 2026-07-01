<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useTeamsStore } from "@/stores/teams";
import { useResourcesStore } from "@/stores/resources";
import type { TeamOverride } from "@/types";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Badge } from "@/components/ui/badge";
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
  DialogTrigger,
} from "@/components/ui/dialog";

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
const overridePdHours = ref<number | null>(null);
const overridePmWorkdays = ref<number | null>(null);

// Confirm dialog state
const deleteDialogOpen = ref(false);
const deleteTargetId = ref<number | null>(null);
const deleteTargetName = computed(() => teams.items.find((t) => t.id === deleteTargetId.value)?.name ?? "");
const removeDialogOpen = ref(false);
const removeTargetId = ref<number | null>(null);
const removeTargetName = computed(() => resourceName(removeTargetId.value ?? 0));

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
  overrideOverload.value = null;
  overrideUnderload.value = null;
  overrideGreen.value = null;
  overrideYellow.value = null;
  overridePdHours.value = null;
  overridePmWorkdays.value = null;
  if (id != null) {
    const existing = await teams.getOverride(id);
    if (existing) {
      overrideOverload.value = existing.overload_threshold;
      overrideUnderload.value = existing.underload_threshold;
      overrideGreen.value = existing.utilization_green;
      overrideYellow.value = existing.utilization_yellow;
      overridePdHours.value = existing.pd_hours;
      overridePmWorkdays.value = existing.pm_workdays;
    }
  }
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

async function removeMember(resourceId: number) {
  if (selectedTeam.value == null) return;
  await teams.removeMember(selectedTeam.value, resourceId);
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
  await teams.setOverride(override);
}

function resourceName(id: number): string {
  return resources.items.find((r) => r.id === id)?.name ?? `#${id}`;
}

function openDeleteDialog(id: number) {
  deleteTargetId.value = id;
  deleteDialogOpen.value = true;
}

async function confirmDelete() {
  if (deleteTargetId.value == null) return;
  await teams.remove(deleteTargetId.value);
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
  <h2 class="text-2xl font-bold tracking-tight">团队 / Teams</h2>

  <h3 class="mt-6 text-xl font-semibold tracking-tight">创建团队</h3>
  <div class="mt-2 flex flex-wrap items-end gap-4">
    <div class="grid gap-2">
      <Label>团队名</Label>
      <Input v-model="teamName" placeholder="团队名" class="w-64" @keyup.enter="createTeam" />
    </div>
    <div class="grid gap-2">
      <Button @click="createTeam">创建团队</Button>
    </div>
  </div>

  <h3 class="mt-6 text-xl font-semibold tracking-tight">团队列表</h3>
  <div class="mt-2 divide-y rounded-lg border">
    <div v-for="t in teams.items" :key="t.id" class="p-4">
      <div class="flex items-start justify-between gap-4">
        <div class="grid gap-1">
          <div class="font-medium">{{ t.name }}</div>
          <div class="text-sm text-muted-foreground">{{ t.description ?? "" }}</div>
        </div>
        <div class="flex items-center gap-2">
          <Button
            size="sm"
            :variant="selectedTeam === t.id ? 'default' : 'outline'"
            @click="selectedTeam = t.id"
          >
            {{ selectedTeam === t.id ? "已选中" : "管理" }}
          </Button>
          <Dialog v-model:open="deleteDialogOpen">
            <DialogTrigger as-child>
              <Button size="sm" variant="destructive" @click="openDeleteDialog(t.id)">删除</Button>
            </DialogTrigger>
            <DialogContent>
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
        </div>
      </div>
    </div>
  </div>

  <template v-if="selectedTeam != null">
    <Separator class="my-6" />

    <h3 class="text-xl font-semibold tracking-tight">{{ selectedTeamName }} — 成员管理</h3>
    <div class="mt-2 flex flex-wrap items-end gap-4">
      <div class="grid gap-2">
        <Label>资源</Label>
        <Select
          :model-value="memberResource ?? undefined"
          @update:model-value="(v) => memberResource = (v as number | undefined) ?? null"
        >
          <SelectTrigger class="w-52">
            <SelectValue placeholder="选择资源" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem v-for="o in resourceOptions" :key="o.value" :value="o.value">
              {{ o.label }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>
      <div class="grid gap-2">
        <Label>角色</Label>
        <Input v-model="memberRole" placeholder="角色 (可选)" class="w-40" />
      </div>
      <div class="grid gap-2">
        <Button @click="addMember">添加成员</Button>
      </div>
    </div>

    <div class="mt-4 divide-y rounded-lg border">
      <div v-for="m in teams.members" :key="`${m.team_id}-${m.resource_id}`" class="p-4">
        <div class="flex items-center justify-between gap-4">
          <div class="font-medium">{{ resourceName(m.resource_id) }}</div>
          <div class="flex items-center gap-2">
            <Badge v-if="m.role" variant="secondary">{{ m.role }}</Badge>
            <span v-else class="text-xs text-muted-foreground">无角色</span>
            <Dialog v-model:open="removeDialogOpen">
              <DialogTrigger as-child>
                <Button size="sm" variant="destructive" @click="openRemoveDialog(m.resource_id)">移除</Button>
              </DialogTrigger>
              <DialogContent>
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
          </div>
        </div>
      </div>
    </div>
    <p v-if="!teams.members.length" class="mt-2 text-sm text-muted-foreground">暂无成员。</p>

    <Separator class="my-6" />

    <h3 class="text-xl font-semibold tracking-tight">团队阈值覆盖</h3>
    <p class="mt-2 text-xs text-muted-foreground">
      设置后该团队使用自己的阈值，覆盖全局设置。留空则不覆盖（使用全局值）。
    </p>
    <div class="mt-4 flex flex-wrap items-end gap-4">
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
      <div class="grid gap-2">
        <Button @click="saveOverride">保存覆盖</Button>
      </div>
    </div>
  </template>
</template>
