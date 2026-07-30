import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import "cmzya-modern-ui/style.css";
import "./styles.css";

createApp(App).use(createPinia()).mount("#app");
