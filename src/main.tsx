import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import DevConsole from "@/components/DevConsole";
import "./styles/globals.css";
import "./lib/i18n";
import { devInfo } from "@/lib/devlog";

devInfo("ui", "BricarOBD v2.0 starting...");

const isDevConsole = window.location.hash === "#/devconsole";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    {isDevConsole ? <DevConsole isStandalone={true} onClose={() => window.close()} /> : <App />}
  </React.StrictMode>
);
