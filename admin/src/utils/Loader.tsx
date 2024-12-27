import { Component, createResource, onMount } from "solid-js";
import { useWorkspace } from "../stores/Workspace";

const Loader: Component = () => {
  const [_w, { fetchSettings, fetchSettingsStatus }] = useWorkspace();
  const [_settings, { refetch }] = createResource(async () => {
    await fetchSettings();
    await fetchSettingsStatus();
  });

  onMount(() => {
    refetch();
  });

  return <></>;
};

export default Loader;
