import { defineStore } from "pinia";
import { computed, ref } from "vue";

export const useShellStore = defineStore("shell", () => {
  const barHeight = ref(40);
  const workspaceCount = ref(4);
  const title = ref("Halo-1");
  const isLockedToTop = ref(true);

  const workspaceLabels = computed(() =>
    Array.from({ length: workspaceCount.value }, (_, index) => `${index + 1}`),
  );

  return {
    barHeight,
    isLockedToTop,
    title,
    workspaceCount,
    workspaceLabels,
  };
});
