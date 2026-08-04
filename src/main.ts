import { mount } from "svelte";
import Bootstrap from "./Bootstrap.svelte";
import "./styles.css";
import "./svelte.css";

const target = document.getElementById("app");

if (!target) throw new Error("app mount target is missing");

mount(Bootstrap, { target });
