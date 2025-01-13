import { Component, createEffect, createResource, onMount } from "solid-js";
import { useWorkspace } from "../stores/Workspace";
import { useLocation, useNavigate } from "@solidjs/router";
import { getPixlieAIAPIRoot } from "./api";

const Loader: Component = () => {
  const [workspace, { fetchSettings, fetchSettingsStatus }] = useWorkspace();
  const [_settings, { refetch }] = createResource(async () => {
    await fetchSettings();
    await fetchSettingsStatus();
  });
  const navigate = useNavigate();
  const location = useLocation();
  const [data] = createResource(async () => {
    let pixieAIAPIRoot = getPixlieAIAPIRoot();
    const res = await fetch(`${pixieAIAPIRoot}/api/settings/check_mqtt_broker`);
    return await res.text();
  });

  onMount(() => {
    refetch();
  });

  createEffect(() => {
    if (location.pathname.startsWith("/settings/setup")) {
      if (
        workspace.isReady &&
        workspace.settingsStatus?.type === "Complete" &&
        data() === "OK"
      ) {
        navigate("/");
      }
    }
    if (
      (workspace.isReady && workspace.settingsStatus?.type !== "Complete") ||
      data() !== "OK"
    ) {
      navigate("/settings/setup");
    }
  });

  return <></>;
};

export default Loader;
