import { mount } from "svelte";
import "./styles/installer.css";
import App from "./App.svelte";

const target = document.getElementById("app");
if (!target) throw new Error("no #app element");
mount(App, { target });
