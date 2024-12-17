/* @refresh reload */
import { render } from "solid-js/web";
import "./index.css";
import App from "./App.tsx";
import { Route, Router } from "@solidjs/router";
import { PerProjectRoutes, PerProjectWrapper } from "./routes/PerProject.tsx";
import { SettingsRoutes, SettingsWrapper } from "./routes/Settings.tsx";

const root = document.getElementById("root");

render(
  () => (
    <Router root={App}>
      <Route path="/p" component={PerProjectWrapper}>
        <PerProjectRoutes />
      </Route>
      <Route path="/settings" component={SettingsWrapper}>
        <SettingsRoutes />
      </Route>
    </Router>
  ),
  root!,
);
