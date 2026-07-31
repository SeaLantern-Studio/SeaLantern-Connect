<script setup lang="ts">
import { CircleAlert, CircleCheck, Info, X } from "lucide-vue-next";
import { t } from "../i18n";
import { dismissToast, pauseToast, resumeToast, toastItems, type ToastTone } from "../toast";

function iconFor(tone: ToastTone) {
  if (tone === "success") return CircleCheck;
  if (tone === "error") return CircleAlert;
  return Info;
}
</script>

<template>
  <TransitionGroup
    name="app-toast"
    tag="div"
    class="toast-region"
    aria-live="polite"
    aria-relevant="additions"
  >
    <div
      v-for="item in toastItems"
      :key="item.id"
      class="toast-item"
      :class="`toast-item-${item.tone}`"
      :role="item.tone === 'error' ? 'alert' : 'status'"
      @pointerenter="pauseToast(item.id)"
      @pointerleave="resumeToast(item.id)"
    >
      <span class="toast-icon" aria-hidden="true">
        <component :is="iconFor(item.tone)" :size="17" :stroke-width="2.2" />
      </span>
      <span class="toast-message">{{ item.message }}</span>
      <button
        class="toast-close"
        type="button"
        :title="t('common.dismiss')"
        :aria-label="t('common.dismiss')"
        @click="dismissToast(item.id)"
      >
        <X :size="15" />
      </button>
    </div>
  </TransitionGroup>
</template>

<style scoped>
.toast-region {
  position: fixed;
  top: 58px;
  right: 18px;
  z-index: 1200;
  width: min(320px, calc(100vw - 36px));
  display: flex;
  flex-direction: column;
  gap: 8px;
  pointer-events: none;
}

.toast-item {
  min-height: 46px;
  display: grid;
  grid-template-columns: 28px minmax(0, 1fr) 26px;
  align-items: center;
  gap: 9px;
  padding: 9px 9px 9px 10px;
  color: var(--text);
  background: color-mix(in srgb, var(--surface) 94%, transparent);
  border: 1px solid color-mix(in srgb, var(--border) 82%, transparent);
  border-radius: var(--cmz-radius-lg);
  box-shadow:
    0 12px 30px rgba(19, 36, 52, 0.14),
    0 2px 7px rgba(19, 36, 52, 0.07);
  backdrop-filter: blur(18px) saturate(140%);
  -webkit-backdrop-filter: blur(18px) saturate(140%);
  pointer-events: auto;
}

:root[data-theme="dark"] .toast-item {
  box-shadow:
    0 14px 32px rgba(0, 0, 0, 0.34),
    0 2px 8px rgba(0, 0, 0, 0.24);
}

.toast-icon {
  width: 28px;
  height: 28px;
  display: grid;
  place-items: center;
  border-radius: var(--cmz-radius-md);
  color: var(--primary);
  background: color-mix(in srgb, var(--primary) 12%, transparent);
}

.toast-item-success .toast-icon {
  color: var(--success);
  background: color-mix(in srgb, var(--success) 13%, transparent);
}

.toast-item-error .toast-icon {
  color: var(--danger);
  background: color-mix(in srgb, var(--danger) 12%, transparent);
}

.toast-message {
  min-width: 0;
  font-size: 13px;
  font-weight: 600;
  line-height: 1.4;
  overflow-wrap: anywhere;
  letter-spacing: 0;
}

.toast-close {
  width: 26px;
  height: 26px;
  display: grid;
  place-items: center;
  padding: 0;
  color: var(--muted);
  background: transparent;
  border: 0;
  border-radius: var(--cmz-radius-sm);
  cursor: pointer;
  opacity: 0.62;
  transition:
    opacity 0.15s ease,
    color 0.15s ease,
    background-color 0.15s ease;
}

.toast-close:hover {
  color: var(--text);
  background: color-mix(in srgb, var(--text) 7%, transparent);
  opacity: 1;
}

.toast-close:focus-visible {
  outline: 2px solid color-mix(in srgb, var(--primary) 55%, transparent);
  outline-offset: 1px;
  opacity: 1;
}

.app-toast-enter-active,
.app-toast-leave-active,
.app-toast-move {
  transition:
    opacity 0.18s ease,
    transform 0.22s cubic-bezier(0.22, 1, 0.36, 1);
}

.app-toast-enter-from,
.app-toast-leave-to {
  opacity: 0;
  transform: translateX(14px);
}

@media (prefers-reduced-motion: reduce) {
  .app-toast-enter-active,
  .app-toast-leave-active,
  .app-toast-move {
    transition-duration: 0.01ms;
  }
}
</style>
