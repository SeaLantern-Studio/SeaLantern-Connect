<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import logoUrl from "../assets/logo.svg";

const props = defineProps<{
  loading: boolean;
}>();

const emit = defineEmits<{
  ready: [];
}>();

const logoScale = ref(0);
const contentVisible = ref(false);
const animationComplete = ref(false);

function finishWhenReady() {
  if (animationComplete.value && !props.loading) {
    emit("ready");
  }
}

onMounted(() => {
  window.setTimeout(() => {
    logoScale.value = 1;
  }, 50);

  window.setTimeout(() => {
    contentVisible.value = true;
  }, 200);

  window.setTimeout(() => {
    animationComplete.value = true;
    finishWhenReady();
  }, 600);
});

watch(() => props.loading, finishWhenReady);
</script>

<template>
  <div class="splash-screen">
    <div class="splash-content">
      <div class="splash-logo" :style="{ transform: `scale(${logoScale})` }">
        <img :src="logoUrl" alt="SeaLantern Connect" width="96" height="96" />
      </div>
      <div class="splash-text" :class="{ visible: contentVisible }">
        <h1>SeaLantern Connect</h1>
        <p>Minecraft Java Edition 联机客户端</p>
      </div>
      <div class="splash-loader" :class="{ visible: contentVisible }" aria-label="正在启动">
        <span></span><span></span><span></span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.splash-screen {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: grid;
  place-items: center;
  background: var(--surface-soft);
}

.splash-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
}

.splash-logo {
  transition: transform 0.5s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.splash-logo img {
  display: block;
  width: 96px;
  height: 96px;
  border-radius: 20px;
  box-shadow: 0 16px 36px color-mix(in srgb, var(--primary) 23%, transparent);
}

.splash-text,
.splash-loader {
  opacity: 0;
  transition: opacity 0.4s ease;
}

.splash-text.visible,
.splash-loader.visible {
  opacity: 1;
}

.splash-text {
  text-align: center;
}
.splash-text h1 {
  margin: 0 0 7px;
  font-size: 28px;
  line-height: 1.2;
}
.splash-text p {
  margin: 0;
  color: var(--muted);
  font-size: 14px;
}

.splash-loader {
  display: flex;
  gap: 8px;
  margin-top: 4px;
}
.splash-loader span {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--primary);
  animation: splash-bounce 1.4s infinite ease-in-out;
}
.splash-loader span:nth-child(1) {
  animation-delay: -0.32s;
}
.splash-loader span:nth-child(2) {
  animation-delay: -0.16s;
}

@keyframes splash-bounce {
  0%,
  80%,
  100% {
    transform: scale(0.8);
    opacity: 0.5;
  }
  40% {
    transform: scale(1.2);
    opacity: 1;
  }
}
</style>
