<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { CalendarIcon } from "@lucide/vue";
import { CalendarDate, type DateValue } from "@internationalized/date";
import { Button } from "@/components/ui/button";
import { Card, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import { NumberField, NumberFieldContent, NumberFieldDecrement, NumberFieldIncrement, NumberFieldInput } from "@/components/ui/number-field";
import { Calendar } from "@/components/ui/calendar";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import ResourceForm from "@/components/ResourceForm.vue";
import { useResourcesStore } from "@/stores/resources";
import { useCatalogStore } from "@/stores/catalog";
import { fmtDate, parseDate } from "@/utils/date";
import type { Resource, ResourceSkill, ResourceTag } from "@/types";

const resources = useResourcesStore();
const catalog = useCatalogStore();
onMounted(() => { resources.load(); catalog.load(); });

// Edit dialog state
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
  set: (dv) => { editAvailFrom.value = dv ? fromDateValue(dv) : null; }
});

const editAvailToDate = computed<DateValue | undefined>({
  get: () => toDateValue(editAvailTo.value),
  set: (dv) => { editAvailTo.value = dv ? fromDateValue(dv) : null; }
});

const editCapacityModel = computed<number | undefined>({
  get: () => editCapacity.value ?? undefined,
  set: (v) => { editCapacity.value = v ?? null; }
});

const editRateModel = computed<number | undefined>({
  get: () => editRate.value ?? undefined,
  set: (v) => { editRate.value = v ?? null; }
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

// Display: per-resource skills/tags fetched lazily for the list.
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
  <h2 class="text-2xl font-bold">资源 / Resources</h2>
  <ResourceForm />
  <div v-if="resources.items.length" class="flex flex-col gap-2 mt-4">
    <Card
      v-for="r in resources.items"
      :key="r.id"
      class="transition-colors hover:bg-muted/50"
      @mouseenter="loadDisplay(r)"
    >
      <CardHeader>
        <div class="flex items-start justify-between gap-4">
          <div class="min-w-0">
            <CardTitle>{{ r.name }}</CardTitle>
            <div class="flex flex-wrap items-center gap-2 mt-1">
              <span v-if="r.email" class="text-xs text-muted-foreground">{{ r.email }}</span>
              <Badge v-if="r.daily_capacity_pd" variant="secondary">{{ r.daily_capacity_pd }} PD/天</Badge>
              <Badge v-if="r.daily_rate_pd" variant="outline">{{ r.daily_rate_pd }}/天</Badge>
              <span v-if="r.available_from" class="text-xs text-muted-foreground">从 {{ r.available_from }}</span>
              <Badge
                v-for="s in (skillCache[r.id] || [])"
                :key="'sk' + s.skill_id"
                variant="secondary"
              >{{ s.skill_name }} {{ s.proficiency }}</Badge>
              <Badge
                v-for="t in (tagCache[r.id] || [])"
                :key="'tg' + t.tag_id"
                :style="t.color ? { backgroundColor: t.color, color: '#fff' } : undefined"
              >{{ t.tag_name }}</Badge>
            </div>
          </div>
          <div class="flex items-center gap-2 shrink-0">
            <Button size="sm" @click="openEdit(r)">编辑</Button>
            <Button size="sm" variant="destructive" @click="openDelete(r)">删除</Button>
          </div>
        </div>
      </CardHeader>
    </Card>
  </div>
  <div v-else class="text-muted-foreground text-sm mt-4">暂无资源</div>

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
            <Select multiple :model-value="editSkills.map(s => s.skillId)" @update:model-value="onSkillSelect">
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
                  {{ catalog.skills.find(sk => sk.id === s.skillId)?.name ?? s.skillId }}
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
</template>
