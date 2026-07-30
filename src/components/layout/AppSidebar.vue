<script setup lang="ts">
import {
  HousePlus,
  LogIn,
  Palette,
  PanelLeftClose,
  PanelLeftOpen,
  Settings,
} from "lucide-vue-next";
import logoUrl from "../../assets/logo.svg";

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

const primaryItems: NavItem[] = [
  { id: "create", label: "创建房间", icon: HousePlus },
  { id: "join", label: "加入房间", icon: LogIn },
];

const utilityItems: NavItem[] = [
  { id: "personalize", label: "个性化", icon: Palette },
  { id: "settings", label: "设置", icon: Settings },
];
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

    <nav class="sidebar-nav" aria-label="主导航">
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
            aria-label="连接正在运行"
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
          :title="collapsed ? '展开侧栏' : '收起侧栏'"
          :aria-label="collapsed ? '展开侧栏' : '收起侧栏'"
          @click="emit('toggleCollapse')"
        >
          <PanelLeftOpen v-if="collapsed" class="nav-icon" :size="19" />
          <PanelLeftClose v-else class="nav-icon" :size="19" />
          <span class="nav-label">收起侧栏</span>
        </button>
      </div>
    </nav>
  </aside>
</template>
