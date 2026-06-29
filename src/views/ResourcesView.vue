<script setup lang="ts">
import { ref } from "vue";
import { NH2, NList, NListItem, NThing, NText, NPopconfirm, NButton, NSpace, NModal, NForm, NFormItem, NInput } from "naive-ui";
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

function openEdit(r: Resource) {
  editingId.value = r.id;
  editName.value = r.name;
  editEmail.value = r.email ?? "";
  editVisible.value = true;
}

async function saveEdit() {
  if (editingId.value == null) return;
  await resources.update(editingId.value, editName.value, editEmail.value || null);
  editVisible.value = false;
}
</script>

<template>
  <n-h2>资源 / Resources</n-h2>
  <ResourceForm />
  <n-list bordered hoverable>
    <n-list-item v-for="r in resources.items" :key="r.id">
      <n-thing :title="r.name">
        <template v-if="r.email" #description>
          {{ r.email }}
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

  <n-modal v-model:show="editVisible" preset="card" title="编辑资源" style="width: 480px">
    <n-form>
      <n-form-item label="姓名">
        <n-input v-model:value="editName" />
      </n-form-item>
      <n-form-item label="邮箱">
        <n-input v-model:value="editEmail" placeholder="email (可选)" />
      </n-form-item>
      <n-space justify="end">
        <n-button @click="editVisible = false">取消</n-button>
        <n-button type="primary" @click="saveEdit">保存</n-button>
      </n-space>
    </n-form>
  </n-modal>
</template>
