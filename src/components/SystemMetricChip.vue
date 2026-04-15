<script setup lang="ts">
defineProps<{
  icon: string;
  tooltip: string;
  value: string;
  muted?: boolean;
  alert?: boolean;
}>();
</script>

<template>
  <div
    class="metric-chip"
    :class="{
      'metric-chip-muted': muted,
      'metric-chip-alert': alert,
    }"
    :data-tooltip="tooltip"
  >
    <img class="metric-icon" :src="icon" alt="Metric icon" />
    <Transition name="metric-fade" mode="out-in">
      <span :key="value" class="metric-value">{{ value }}</span>
    </Transition>
  </div>
</template>
