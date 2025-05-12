import { useParams } from "@solidjs/router";
import { Component, createMemo, For } from "solid-js";
import { NodeLabel } from "../../api_types/NodeLabel.ts";
import { ProjectSettings } from "../../api_types/ProjectSettings.ts";
import { useEngine } from "../../stores/engine.tsx";
import { IEngine } from "../../utils/types";
import { identifierToTitle } from "../../utils/utils.ts";

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
        <div>
          {/* <FormLabel label="Only extract data from specified links" /> */}
          <For each={Object.entries(getProjectSettings()!)}>
            {([settingKey, settingValue]) => (
              <div>
                {identifierToTitle(settingKey)}: {settingValue ? "Yes" : "No"}
              </div>
            )}
          </For>
        </div>
      )}
    </>
  );
};

export default ProjectSettingsNode;
