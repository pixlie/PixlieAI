/* @refresh reload */
import { render } from "solid-js/web";
import "./index.css";
import App from "./App.tsx";
import { Router } from "@solidjs/router";
import Routes from "./routes/index.tsx";

const root = document.getElementById("root");

render(
  () => (
    <Router root={App}>
      <Routes />
    </Router>
  ),
  root!,
);
