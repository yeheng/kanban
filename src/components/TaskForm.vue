<script setup lang="ts">
import { computed, ref } from "vue";
import { CalendarDate, type DateValue } from "@internationalized/date";
import { Button } from "@/components/ui/button";
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
import { Switch } from "@/components/ui/switch";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Calendar } from "@/components/ui/calendar";
import { useTasksStore } from "@/stores/tasks";
import { useProjectsStore } from "@/stores/projects";
import { useCatalogStore } from "@/stores/catalog";
import { fmtDate, fmtDateOrNull, parseDate } from "@/utils/date";

const tasks = useTasksStore();
const projects = useProjectsStore();
const catalog = useCatalogStore();
const title = ref("");
const estimate = ref(1);
const selectedSkills = ref<number[]>([]);
const selectedTags = ref<number[]>([]);
const isLongTerm = ref(false);
const segmentKind = ref<string | null>(null);
const parentTaskId = ref<number | null>(null);
const startMs = ref<number | null>(null);
const endMs = ref<number | null>(null);

const skillOptions = computed(() =>
  catalog.skills.map((s) => ({ label: s.name, value: s.id })),
);
const tagOptions = computed(() =>
  catalog.tags.map((t) => ({ label: t.name, value: t.id })),
);
// Parent candidates: long-term tasks in the current project (top-level, not segments).
const parentOptions = computed(() =>
  tasks.tasks
    .filter((t) => t.title)
    .map((t) => ({ label: t.title, value: t.id })),
);
const segmentKindOptions = [
  { label: "阶段 phase", value: "phase" },
  { label: "里程碑 milestone", value: "milestone" },
  { label: "分段 segment", value: "segment" },
];

function toCalendarDate(ms: number): DateValue {
  const [y, m, d] = fmtDate(ms).split("-").map(Number);
  return new CalendarDate(y, m, d);
}

function dateValueToMs(v: DateValue | null | undefined): number | null {
  if (!v) return null;
  return parseDate(`${v.year}-${String(v.month).padStart(2, "0")}-${String(v.day).padStart(2, "0")}`);
}

const startDate = computed<DateValue | undefined>({
  get: () => (startMs.value ? toCalendarDate(startMs.value) : undefined),
  set: (v) => { startMs.value = dateValueToMs(v); },
});

const endDate = computed<DateValue | undefined>({
  get: () => (endMs.value ? toCalendarDate(endMs.value) : undefined),
  set: (v) => { endMs.value = dateValueToMs(v); },
});

function handleSkillUpdate(v: unknown) {
  selectedSkills.value = (v as number[] | undefined) ?? [];
}

function handleTagUpdate(v: unknown) {
  selectedTags.value = (v as number[] | undefined) ?? [];
}

function handleSegmentKindUpdate(v: unknown) {
  segmentKind.value = (v as string | undefined) || null;
  if (!segmentKind.value) parentTaskId.value = null;
}

function handleParentUpdate(v: unknown) {
  parentTaskId.value = (v as number | undefined) ?? null;
}

async function submit() {
  if (!title.value.trim() || !projects.current) return;
  const skillReqs = selectedSkills.value.map((id) => [id, 3, true, 1] as [number, number, boolean, number]);
  await tasks.create({
    projectId: projects.current,
    title: title.value,
    estimatePd: estimate.value,
    start: fmtDateOrNull(startMs.value),
    end: fmtDateOrNull(endMs.value),
    skillReqs,
    tagIds: selectedTags.value,
    isLongTerm: isLongTerm.value,
    parentTaskId: segmentKind.value ? parentTaskId.value : null,
    segmentKind: segmentKind.value,
  });
  title.value = "";
  estimate.value = 1;
  selectedSkills.value = [];
  selectedTags.value = [];
  isLongTerm.value = false;
  segmentKind.value = null;
  parentTaskId.value = null;
  startMs.value = null;
  endMs.value = null;
}
</script>

<template>
  <div class="flex flex-wrap items-end gap-4">
    <div class="grid gap-2">
      <Label for="task-title">标题</Label>
      <Input id="task-title" v-model="title" placeholder="任务标题" @keyup.enter="submit" />
    </div>

    <div class="grid gap-2">
      <Label for="task-estimate">PD</Label>
      <NumberField id="task-estimate" v-model="estimate" :min="0">
        <NumberFieldContent>
          <NumberFieldDecrement />
          <NumberFieldInput />
          <NumberFieldIncrement />
        </NumberFieldContent>
      </NumberField>
    </div>

    <div class="grid gap-2">
      <Label>起始日</Label>
      <Popover>
        <PopoverTrigger as-child>
          <Button variant="outline" class="w-[140px] justify-start text-left font-normal">
            {{ fmtDateOrNull(startMs) ?? "选择起始日" }}
          </Button>
        </PopoverTrigger>
        <PopoverContent class="w-auto p-0">
          <Calendar v-model="startDate" />
        </PopoverContent>
      </Popover>
    </div>

    <div class="grid gap-2">
      <Label>截止日</Label>
      <Popover>
        <PopoverTrigger as-child>
          <Button variant="outline" class="w-[140px] justify-start text-left font-normal">
            {{ fmtDateOrNull(endMs) ?? "选择截止日" }}
          </Button>
        </PopoverTrigger>
        <PopoverContent class="w-auto p-0">
          <Calendar v-model="endDate" />
        </PopoverContent>
      </Popover>
    </div>

    <div class="grid gap-2">
      <Label for="task-skills">技能</Label>
      <Select :model-value="selectedSkills" multiple @update:model-value="handleSkillUpdate">
        <SelectTrigger id="task-skills" class="w-[180px]">
          <SelectValue placeholder="选择技能" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem v-for="opt in skillOptions" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </SelectItem>
        </SelectContent>
      </Select>
    </div>

    <div class="grid gap-2">
      <Label for="task-tags">标签</Label>
      <Select :model-value="selectedTags" multiple @update:model-value="handleTagUpdate">
        <SelectTrigger id="task-tags" class="w-[180px]">
          <SelectValue placeholder="选择标签" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem v-for="opt in tagOptions" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </SelectItem>
        </SelectContent>
      </Select>
    </div>

    <div class="grid gap-2">
      <Label for="task-longterm" class="cursor-pointer">长期任务</Label>
      <Switch id="task-longterm" v-model="isLongTerm" />
    </div>

    <div class="grid gap-2">
      <Label for="task-segment-kind">分段类型</Label>
      <Select :model-value="segmentKind" @update:model-value="handleSegmentKindUpdate">
        <SelectTrigger id="task-segment-kind" class="w-[180px]">
          <SelectValue placeholder="无" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem v-for="opt in segmentKindOptions" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </SelectItem>
        </SelectContent>
      </Select>
    </div>

    <div v-if="segmentKind" class="grid gap-2">
      <Label for="task-parent">父任务</Label>
      <Select :model-value="parentTaskId" @update:model-value="handleParentUpdate">
        <SelectTrigger id="task-parent" class="w-[180px]">
          <SelectValue placeholder="选择父任务" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem v-for="opt in parentOptions" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </SelectItem>
        </SelectContent>
      </Select>
    </div>

    <div class="grid gap-2">
      <Label class="invisible">操作</Label>
      <Button @click="submit">新建任务</Button>
    </div>
  </div>
</template>
