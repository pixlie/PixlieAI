import { Component, createMemo } from "solid-js";
import { useEngine } from "../../stores/engine.tsx";
import { useParams } from "@solidjs/router";
import { IEngine } from "../../utils/types";
import { ProjectSettings } from "../../api_types/ProjectSettings.ts";
import { NodeLabel } from "../../api_types/NodeLabel.ts";
import FormLabel from "../interactable/FormLabel.tsx";

interface INodeProps {
  nodeId: number;
}

const ProjectSettingsNode: Component<INodeProps> = (props) => {
  const [engine] = useEngine();
  const params = useParams();

  const getProject = createMemo<IEngine | undefined>(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  const getProjectSettings = createMemo<ProjectSettings | undefined>(() => {
    if (getProject() && props.nodeId in getProject()!.nodes) {
      let node = getProject()!.nodes[props.nodeId];
      if (
        node.labels.includes("ProjectSettings" as NodeLabel) &&
        node.payload.type === "ProjectSettings"
      ) {
        return node.payload.data as ProjectSettings;
      }
    }
    return undefined;
  });

  return (
    <>
      {!!getProjectSettings() && (
        <div class="">
          <FormLabel label="Only extract data from specified links" />
        </div>
      )}
    </>
  );
};

export default ProjectSettingsNode;
