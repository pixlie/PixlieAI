import { Component, createEffect, createResource, onMount } from "solid-js";
import { useWorkspace } from "../stores/Workspace";
import { useLocation, useNavigate } from "@solidjs/router";

const Loader: Component = () => {
  const [workspace, { fetchSettings, fetchSettingsStatus }] = useWorkspace();
  const [_settings, { refetch }] = createResource(async () => {
    await fetchSettings();
    await fetchSettingsStatus();
  });
  const navigate = useNavigate();
  const location = useLocation();

  onMount(() => {
    refetch();
  });

  createEffect(() => {
    if (location.pathname.startsWith("/settings/setup")) {
      return;
    }
    if (workspace.isReady && workspace.settingsStatus?.type !== "Complete") {
      navigate("/settings/setup");
    }
  });

  return <></>;
};

export default Loader;
