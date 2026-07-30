import { reactive } from "vue";

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
const timers = new Map<number, ToastTimer>();
let nextId = 0;

export const toastItems = reactive<ToastItem[]>([]);

function removeItem(id: number) {
  const index = toastItems.findIndex((item) => item.id === id);
  if (index >= 0) toastItems.splice(index, 1);
}

function schedule(id: number, duration: number) {
  const startedAt = Date.now();
  const handle = window.setTimeout(() => dismissToast(id), duration);
  timers.set(id, { handle, remaining: duration, startedAt });
}

function showToast(message: string, tone: ToastTone, duration: number) {
  if (toastItems.length >= MAX_TOASTS) dismissToast(toastItems[0].id);

  const id = ++nextId;
  toastItems.push({ id, message, tone });
  schedule(id, duration);
  return id;
}

export function dismissToast(id: number) {
  const timer = timers.get(id);
  if (timer) window.clearTimeout(timer.handle);
  timers.delete(id);
  removeItem(id);
}

export function pauseToast(id: number) {
  const timer = timers.get(id);
  if (!timer) return;

  window.clearTimeout(timer.handle);
  timer.remaining = Math.max(0, timer.remaining - (Date.now() - timer.startedAt));
}

export function resumeToast(id: number) {
  const timer = timers.get(id);
  if (!timer) return;
  if (timer.remaining <= 0) {
    dismissToast(id);
    return;
  }
  schedule(id, timer.remaining);
}

export const toast = {
  success: (message: string) => showToast(message, "success", 2000),
  error: (message: string) => showToast(message, "error", 4000),
  info: (message: string) => showToast(message, "info", 3000),
};
