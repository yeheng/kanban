<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
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
import { useResourcesStore } from "@/stores/resources";
import { useCatalogStore } from "@/stores/catalog";
import { fmtDate, parseDate } from "@/utils/date";
import type { Resource, ResourceSkill, ResourceTag } from "@/types";

const resources = useResourcesStore();
const catalog = useCatalogStore();
onMounted(() => { resources.load(); catalog.load(); });

// Filters
const filterName = ref("");
const filterStatus = ref("all");

const statusOptions = computed(() => {
  const set = new Set(resources.items.map((r) => r.status));
  return Array.from(set).sort().map((s) => ({ label: s, value: s }));
});

const isFiltered = computed(() => !!(filterName.value || filterStatus.value !== "all"));

const filteredResources = computed(() => {
  return resources.items.filter((r) => {
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

const skillOptions = () => catalog.skills.map((s) => ({ label: s.name, value: s.id }));
const tagOptions = () => catalog.tags.map((t) => ({ label: t.name, value: t.id }));

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

const skillCache = ref<Record<number, ResourceSkill[]>>({});
const tagCache = ref<Record<number, ResourceTag[]>>({});
async function loadDisplay(r: Resource) {
  if (!skillCache.value[r.id]) skillCache.value[r.id] = await resources.loadSkills(r.id);
  if (!tagCache.value[r.id]) tagCache.value[r.id] = await resources.loadTags(r.id);
}

async function openEdit(r: Resource) {
  editingId.value = r.id;
  editName.value = r.name;
  editEmail.value = r.email ?? "";
  editAvailFrom.value = parseDate(r.available_from);
  editAvailTo.value = parseDate(r.available_to);
  editCapacity.value = r.daily_capacity_pd;
  editRate.value = r.daily_rate_pd;
  const [skills, tags] = await Promise.all([resources.loadSkills(r.id), resources.loadTags(r.id)]);
  editSkills.value = skills.map((s) => ({ skillId: s.skill_id, proficiency: s.proficiency }));
  editTags.value = tags.map((t) => t.tag_id);
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
  await resources.saveSkills(editingId.value, editSkills.value.map((s) => [s.skillId, s.proficiency]));
  await resources.saveTags(editingId.value, editTags.value);
  delete skillCache.value[editingId.value];
  delete tagCache.value[editingId.value];
  editVisible.value = false;
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
  await resources.remove(deletingId.value);
  deleteDialogOpen.value = false;
  deletingId.value = null;
  deletingName.value = "";
}
</script>

<template>
  <ListPage title="资源 / Resources" description="管理人员、技能与标签">
    <template #actions>
      <Button @click="createVisible = true">
        <PlusIcon class="mr-2 h-4 w-4" />
        新建资源
      </Button>
    </template>

    <Card>
      <CardHeader class="pb-3">
        <CardTitle class="text-base">资源列表</CardTitle>
      </CardHeader>
      <CardContent class="space-y-4">
        <ListToolbar
          v-model:search="filterName"
          v-model:filter="filterStatus"
          :show-reset="isFiltered"
          search-placeholder="搜索姓名..."
          filter-label="状态"
          :filter-options="statusOptions"
          @reset="resetFilters"
        />

        <div class="rounded-md border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>姓名</TableHead>
                <TableHead class="hidden md:table-cell">邮箱</TableHead>
                <TableHead>状态</TableHead>
                <TableHead class="hidden md:table-cell">日容量</TableHead>
                <TableHead class="hidden lg:table-cell">可用周期</TableHead>
                <TableHead class="hidden xl:table-cell">技能 / 标签</TableHead>
                <TableHead class="w-[60px]" />
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow
                v-for="r in filteredResources"
                :key="r.id"
                @mouseenter="loadDisplay(r)"
              >
                <TableCell class="font-medium">{{ r.name }}</TableCell>
                <TableCell class="hidden md:table-cell text-muted-foreground">
                  {{ r.email || "-" }}
                </TableCell>
                <TableCell>
                  <Badge variant="outline">{{ r.status }}</Badge>
                </TableCell>
                <TableCell class="hidden md:table-cell">
                  {{ r.daily_capacity_pd != null ? `${r.daily_capacity_pd} PD/天` : "-" }}
                </TableCell>
                <TableCell class="hidden lg:table-cell text-muted-foreground whitespace-nowrap">
                  <span v-if="r.available_from && r.available_to">
                    {{ r.available_from }} ~ {{ r.available_to }}
                  </span>
                  <span v-else>-</span>
                </TableCell>
                <TableCell class="hidden xl:table-cell">
                  <div class="flex flex-wrap gap-1">
                    <Badge
                      v-for="s in (skillCache[r.id] || []).slice(0, 3)"
                      :key="'sk' + s.skill_id"
                      variant="secondary"
                      class="text-xs"
                    >
                      {{ s.skill_name }} {{ s.proficiency }}
                    </Badge>
                    <Badge
                      v-for="t in (tagCache[r.id] || []).slice(0, 3)"
                      :key="'tg' + t.tag_id"
                      class="text-xs"
                      :style="t.color ? { backgroundColor: t.color, color: '#fff' } : undefined"
                    >
                      {{ t.tag_name }}
                    </Badge>
                    <span
                      v-if="!(skillCache[r.id]?.length || tagCache[r.id]?.length)"
                      class="text-xs text-muted-foreground"
                    >-</span>
                  </div>
                </TableCell>
                <TableCell class="text-right">
                  <ListRowActions
                    @edit="openEdit(r)"
                    @delete="openDelete(r)"
                  />
                </TableCell>
              </TableRow>
              <TableRow v-if="!filteredResources.length">
                <TableCell colspan="7" class="text-center text-muted-foreground py-6">
                  无匹配资源
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>

    <!-- Create resource dialog -->
    <Dialog v-model:open="createVisible">
      <DialogContent class="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>新建资源</DialogTitle>
        </DialogHeader>
        <CardContent class="pt-4">
          <ResourceForm />
        </CardContent>
      </DialogContent>
    </Dialog>

    <!-- Edit resource dialog -->
    <Dialog v-model:open="editVisible">
      <DialogContent class="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>编辑资源</DialogTitle>
        </DialogHeader>
        <div class="grid gap-4 py-4">
          <div class="grid gap-2">
            <Label for="edit-name">姓名</Label>
            <Input id="edit-name" v-model="editName" />
          </div>
          <div class="grid gap-2">
            <Label for="edit-email">邮箱</Label>
            <Input id="edit-email" v-model="editEmail" placeholder="email (可选)" />
          </div>
          <div class="grid gap-2">
            <Label>可用起始日</Label>
            <Popover>
              <PopoverTrigger as-child>
                <Button variant="outline" class="w-full justify-start text-left font-normal">
                  <CalendarIcon class="mr-2 size-4" />
                  {{ editAvailFrom != null ? fmtDate(editAvailFrom) : '选择日期' }}
                </Button>
              </PopoverTrigger>
              <PopoverContent class="w-auto p-2">
                <Calendar v-model="editAvailFromDate" initial-focus />
                <div class="mt-2 flex justify-end">
                  <Button variant="ghost" size="sm" @click="editAvailFrom = null">清除</Button>
                </div>
              </PopoverContent>
            </Popover>
          </div>
          <div class="grid gap-2">
            <Label>可用截止日</Label>
            <Popover>
              <PopoverTrigger as-child>
                <Button variant="outline" class="w-full justify-start text-left font-normal">
                  <CalendarIcon class="mr-2 size-4" />
                  {{ editAvailTo != null ? fmtDate(editAvailTo) : '选择日期' }}
                </Button>
              </PopoverTrigger>
              <PopoverContent class="w-auto p-2">
                <Calendar v-model="editAvailToDate" initial-focus />
                <div class="mt-2 flex justify-end">
                  <Button variant="ghost" size="sm" @click="editAvailTo = null">清除</Button>
                </div>
              </PopoverContent>
            </Popover>
          </div>
          <div class="grid gap-2">
            <Label for="edit-capacity">日容量 (PD)</Label>
            <NumberField v-model="editCapacityModel" :min="0" :step="0.5">
              <NumberFieldContent>
                <NumberFieldDecrement />
                <NumberFieldInput id="edit-capacity" placeholder="如 1.0" />
                <NumberFieldIncrement />
              </NumberFieldContent>
            </NumberField>
          </div>
          <div class="grid gap-2">
            <Label for="edit-rate">日费率</Label>
            <NumberField v-model="editRateModel" :min="0" :step="100">
              <NumberFieldContent>
                <NumberFieldDecrement />
                <NumberFieldInput id="edit-rate" placeholder="如 800" />
                <NumberFieldIncrement />
              </NumberFieldContent>
            </NumberField>
          </div>
          <div class="grid gap-2">
            <Label>技能</Label>
            <div class="grid gap-2">
              <Select multiple :model-value="editSkills.map((s) => s.skillId)" @update:model-value="onSkillSelect">
                <SelectTrigger>
                  <SelectValue placeholder="选择技能" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem v-for="opt in skillOptions()" :key="opt.value" :value="opt.value">{{ opt.label }}</SelectItem>
                </SelectContent>
              </Select>
              <div v-if="editSkills.length" class="flex flex-col gap-2">
                <div v-for="s in editSkills" :key="s.skillId" class="flex items-center gap-2">
                  <span class="text-xs w-24 truncate">
                    {{ catalog.skills.find((sk) => sk.id === s.skillId)?.name ?? s.skillId }}
                  </span>
                  <NumberField v-model="s.proficiency" :min="1" :max="5" :step="1" class="w-28">
                    <NumberFieldContent>
                      <NumberFieldDecrement />
                      <NumberFieldInput />
                      <NumberFieldIncrement />
                    </NumberFieldContent>
                  </NumberField>
                  <span class="text-xs text-muted-foreground">熟练度 1-5</span>
                </div>
              </div>
            </div>
          </div>
          <div class="grid gap-2">
            <Label>标签</Label>
            <Select multiple :model-value="editTags" @update:model-value="onTagSelect">
              <SelectTrigger>
                <SelectValue placeholder="选择标签" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="opt in tagOptions()" :key="opt.value" :value="opt.value">{{ opt.label }}</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" @click="editVisible = false">取消</Button>
          <Button @click="saveEdit">保存</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <Dialog v-model:open="deleteDialogOpen">
      <DialogContent class="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>确认删除</DialogTitle>
          <DialogDescription>确定删除资源 "{{ deletingName }}" 吗？</DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" @click="deleteDialogOpen = false">取消</Button>
          <Button variant="destructive" @click="confirmDelete">删除</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </ListPage>
</template>
