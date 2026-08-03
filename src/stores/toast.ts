import { defineStore } from "pinia";
import { ref } from "vue";

export type ToastTone = "success" | "error" | "info";

export interface ToastItem {
  id: number;
  message: string;
  tone: ToastTone;
}

interface ToastTimer {
  handle: number;
  remaining: number;
  startedAt: number;
}

const MAX_TOASTS = 3;
export const useToastStore = defineStore("toast", () => {
  const items = ref<ToastItem[]>([]);
  const timers = new Map<number, ToastTimer>();
  let nextId = 0;

  function removeItem(id: number) {
    const index = items.value.findIndex((item) => item.id === id);
    if (index >= 0) items.value.splice(index, 1);
  }

  function schedule(id: number, duration: number) {
    const startedAt = Date.now();
    const handle = window.setTimeout(() => dismiss(id), duration);
    timers.set(id, { handle, remaining: duration, startedAt });
  }

  function show(message: string, tone: ToastTone, duration: number) {
    if (items.value.length >= MAX_TOASTS) dismiss(items.value[0].id);

    const id = ++nextId;
    items.value.push({ id, message, tone });
    schedule(id, duration);
    return id;
  }

  function dismiss(id: number) {
    const timer = timers.get(id);
    if (timer) window.clearTimeout(timer.handle);
    timers.delete(id);
    removeItem(id);
  }

  function pause(id: number) {
    const timer = timers.get(id);
    if (!timer) return;

    window.clearTimeout(timer.handle);
    timer.remaining = Math.max(0, timer.remaining - (Date.now() - timer.startedAt));
  }

  function resume(id: number) {
    const timer = timers.get(id);
    if (!timer) return;
    if (timer.remaining <= 0) {
      dismiss(id);
      return;
    }
    schedule(id, timer.remaining);
  }

  return {
    items,
    show,
    dismiss,
    pause,
    resume,
    success: (message: string) => show(message, "success", 2000),
    error: (message: string) => show(message, "error", 4000),
    info: (message: string) => show(message, "info", 3000),
  };
});
