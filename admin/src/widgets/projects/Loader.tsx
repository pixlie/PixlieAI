import { Component, onMount } from "solid-js";
import { useEngine } from "../../stores/engine";
import { useParams } from "@solidjs/router";
import { useWorkspace } from "../../stores/workspace";

const ProjectLoader: Component = () => {
  const [_, { setCurrentProject }] = useEngine();
  const [_w, { fetchSettings }] = useWorkspace();
  const params = useParams();

  onMount(() => {
    if (params.projectId) {
      setCurrentProject(params.projectId).then(() => {
        fetchSettings().then(() => {});
      });
    }
  });

  return null;
};

export default ProjectLoader;
