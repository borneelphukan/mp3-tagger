import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App.tsx";
import icon from "../../assets/icon.png";
import "./index.css";

// Let Vite resolve the shared app icon in both dev and project-site builds.
const favicon = document.querySelector<HTMLLinkElement>("#app-icon");
if (favicon) favicon.href = icon;

const root = document.getElementById("root");
if (!root) {
  throw new Error("missing #root element");
}

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
