<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";

const now = ref(new Date());
let timerId: number | undefined;

const dateLabel = computed(() =>
  new Intl.DateTimeFormat("en-US", {
    weekday: "short",
    month: "short",
    day: "numeric",
  }).format(now.value),
);

const timeLabel = computed(() =>
  new Intl.DateTimeFormat("en-US", {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(now.value),
);

onMounted(() => {
  timerId = window.setInterval(() => {
    now.value = new Date();
  }, 1000);
});

onBeforeUnmount(() => {
  window.clearInterval(timerId);
});
</script>

<template>
  <section class="panel panel-center" aria-label="Date and time">
    <div class="center-pill">
      <span class="center-pill-date">{{ dateLabel }}</span>
      <span class="center-pill-divider" aria-hidden="true"></span>
      <span class="center-pill-time">{{ timeLabel }}</span>
    </div>
  </section>
</template>
