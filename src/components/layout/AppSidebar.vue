<script setup lang="ts">
import { computed } from "vue";
import {
  HousePlus,
  LogIn,
  Palette,
  PanelLeftClose,
  PanelLeftOpen,
  Settings,
} from "lucide-vue-next";
import logoUrl from "../../assets/logo.svg";
import { t } from "../../i18n";

interface NavItem {
  id: string;
  label: string;
  icon: typeof HousePlus;
}

defineProps<{
  active: string;
  connectionState: "idle" | "busy" | "active";
  tunnelMode: "host" | "join" | null;
  collapsed: boolean;
}>();

const emit = defineEmits<{
  navigate: [id: string];
  toggleCollapse: [];
}>();

const primaryItems = computed<NavItem[]>(() => [
  { id: "join", label: t("navigation.joinRoom"), icon: LogIn },
  { id: "create", label: t("navigation.createRoom"), icon: HousePlus },
]);

const utilityItems = computed<NavItem[]>(() => [
  { id: "personalize", label: t("navigation.personalization"), icon: Palette },
  { id: "settings", label: t("navigation.settings"), icon: Settings },
]);
</script>

<template>
  <aside class="sidebar" :class="{ collapsed }">
    <div class="sidebar-brand" data-tauri-drag-region title="SeaLantern Connect">
      <img :src="logoUrl" alt="" draggable="false" />
      <div class="sidebar-brand-name" data-tauri-drag-region>
        <strong data-tauri-drag-region>SeaLantern</strong>
        <small data-tauri-drag-region>Connect</small>
      </div>
    </div>

    <nav class="sidebar-nav" :aria-label="t('navigation.main')">
      <div class="nav-group">
        <button
          v-for="item in primaryItems"
          :key="item.id"
          class="nav-item"
          :class="{ active: active === item.id }"
          type="button"
          :title="item.label"
          @click="emit('navigate', item.id)"
        >
          <component :is="item.icon" class="nav-icon" :size="19" />
          <span class="nav-label">{{ item.label }}</span>
          <span
            v-if="
              connectionState !== 'idle' &&
              ((item.id === 'join' && tunnelMode === 'join') ||
                (item.id === 'create' && tunnelMode === 'host'))
            "
            class="connection-dot"
            :class="connectionState"
            :aria-label="t('navigation.connectionRunning')"
          ></span>
        </button>
      </div>

      <div class="nav-group nav-group-bottom">
        <button
          v-for="item in utilityItems"
          :key="item.id"
          class="nav-item"
          :class="{ active: active === item.id }"
          type="button"
          :title="item.label"
          @click="emit('navigate', item.id)"
        >
          <component :is="item.icon" class="nav-icon" :size="19" />
          <span class="nav-label">{{ item.label }}</span>
        </button>
        <div class="nav-separator"></div>
        <button
          class="nav-item collapse-button"
          type="button"
          :title="collapsed ? t('navigation.expandSidebar') : t('navigation.collapseSidebar')"
          :aria-label="collapsed ? t('navigation.expandSidebar') : t('navigation.collapseSidebar')"
          @click="emit('toggleCollapse')"
        >
          <PanelLeftOpen v-if="collapsed" class="nav-icon" :size="19" />
          <PanelLeftClose v-else class="nav-icon" :size="19" />
          <span class="nav-label">{{ t("navigation.collapseSidebar") }}</span>
        </button>
      </div>
    </nav>
  </aside>
</template>
