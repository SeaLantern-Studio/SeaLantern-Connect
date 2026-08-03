<script setup lang="ts">
import { onMounted, ref } from "vue";
import logoUrl from "../assets/logo.png";
import { getAppVersion } from "@api";
import { t } from "@i18n";

const version = ref("--");

onMounted(async () => {
  try {
    version.value = await getAppVersion();
  } catch {
    // Keep the page usable when the Tauri API is unavailable in a browser preview.
  }
});
</script>

<template>
  <section class="about-view">
    <div class="about-product">
      <img :src="logoUrl" alt="" draggable="false" />
      <div>
        <h2>SeaLantern Connect</h2>
        <p>{{ t("about.description") }}</p>
      </div>
    </div>

    <dl class="about-details">
      <div>
        <dt>{{ t("about.version") }}</dt>
        <dd>v{{ version }}</dd>
      </div>
      <div>
        <dt>{{ t("about.license") }}</dt>
        <dd>Apache-2.0</dd>
      </div>
      <div>
        <dt>{{ t("about.developer") }}</dt>
        <dd>SeaLantern-Studio</dd>
      </div>
    </dl>

    <div class="about-frp">
      <h3>{{ t("about.frpTitle") }}</h3>
      <p>{{ t("about.frpDescription") }}</p>
    </div>
  </section>
</template>
