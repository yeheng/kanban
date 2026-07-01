<script setup lang="ts">
import { computed, ref } from "vue";
import { useListAllocationsQuery, useUpdateAllocationMutation, useDeleteAllocationMutation } from "@/services/api/allocations.api";
import { useProjectsStore } from "@/stores/projects";
import AllocationForm from "@/components/AllocationForm.vue";
import DateRangePicker from "@/components/DateRangePicker.vue";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { NumberField, NumberFieldContent, NumberFieldDecrement, NumberFieldIncrement, NumberFieldInput } from "@/components/ui/number-field";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { fmtDate, parseDateStrict } from "@/utils/date";
import type { AllocationView } from "@/types";

const allocationsQuery = useListAllocationsQuery(computed(() => projects.current));
const updateAllocation = useUpdateAllocationMutation();
const deleteAllocation = useDeleteAllocationMutation();
const projects = useProjectsStore();

// Edit modal state
const editVisible = ref(false);
const editingId = ref<number | null>(null);
const editDateRange = ref<[number, number]>([0, 0]);
const editPercent = ref(0.5);

function openEdit(row: AllocationView) {
  editingId.value = row.id;
  editDateRange.value = [parseDateStrict(row.start_date), parseDateStrict(row.end_date)];
  editPercent.value = row.percent;
  editVisible.value = true;
}

async function saveEdit() {
  if (editingId.value == null || projects.current == null) return;
  await updateAllocation.mutateAsync({
    id: editingId.value,
    start: fmtDate(editDateRange.value[0]),
    end: fmtDate(editDateRange.value[1]),
    percent: editPercent.value,
    projectId: projects.current,
  });
  editVisible.value = false;
}

// Delete confirmation state
const deleteVisible = ref(false);
const deleteTargetId = ref<number | null>(null);

function openDelete(row: AllocationView) {
  deleteTargetId.value = row.id;
  deleteVisible.value = true;
}

async function confirmDelete() {
  if (deleteTargetId.value == null || projects.current == null) return;
  await deleteAllocation.mutateAsync({ id: deleteTargetId.value, projectId: projects.current });
  deleteVisible.value = false;
  deleteTargetId.value = null;
}

const tableData = computed(() => allocationsQuery.data.value ?? []);
</script>

<template>
  <h2 class="text-2xl font-bold">分配 / Allocations</h2>
  <AllocationForm />
  <Table class="mt-3">
    <TableHeader>
      <TableRow>
        <TableHead>资源</TableHead>
        <TableHead>任务</TableHead>
        <TableHead>区间</TableHead>
        <TableHead>投入</TableHead>
        <TableHead>来源</TableHead>
        <TableHead class="w-[120px]">操作</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      <TableRow v-for="row in tableData" :key="row.id">
        <TableCell>{{ row.resource_name }}</TableCell>
        <TableCell>{{ row.task_title }}</TableCell>
        <TableCell>{{ row.start_date }} → {{ row.end_date }}</TableCell>
        <TableCell>{{ Math.round(row.percent * 100) }}%</TableCell>
        <TableCell>{{ row.source }}</TableCell>
        <TableCell>
          <div class="flex items-center gap-2">
            <Button size="sm" variant="outline" @click="openEdit(row)">编辑</Button>
            <Button size="sm" variant="destructive" @click="openDelete(row)">删除</Button>
          </div>
        </TableCell>
      </TableRow>
      <TableRow v-if="tableData.length === 0">
        <TableCell colspan="6" class="text-center text-muted-foreground py-8">
          暂无数据
        </TableCell>
      </TableRow>
    </TableBody>
  </Table>

  <Dialog v-model:open="editVisible">
    <DialogContent class="sm:max-w-[480px]">
      <DialogHeader>
        <DialogTitle>编辑分配</DialogTitle>
      </DialogHeader>
      <div class="grid gap-4 py-4">
        <div class="grid gap-2">
          <Label>区间</Label>
          <DateRangePicker v-model="editDateRange" />
        </div>
        <div class="grid gap-2">
          <Label>投入比例</Label>
          <NumberField v-model="editPercent" :min="0.01" :max="1" :step="0.05">
            <NumberFieldContent>
              <NumberFieldDecrement />
              <NumberFieldInput />
              <NumberFieldIncrement />
            </NumberFieldContent>
          </NumberField>
        </div>
      </div>
      <DialogFooter>
        <Button variant="outline" @click="editVisible = false">取消</Button>
        <Button @click="saveEdit">保存</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>

  <Dialog v-model:open="deleteVisible">
    <DialogContent class="sm:max-w-[360px]">
      <DialogHeader>
        <DialogTitle>确认删除</DialogTitle>
        <DialogDescription>确定删除此分配吗？</DialogDescription>
      </DialogHeader>
      <DialogFooter>
        <Button variant="outline" @click="deleteVisible = false">取消</Button>
        <Button variant="destructive" @click="confirmDelete">确定</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
