import { Component, For } from "solid-js";
import { useExplorer } from "../../stores/explorer.tsx";
import WorkflowPath from "./WorkflowPath.tsx";

interface IWorkflowPathsProps {
  projectId: string;
}

const WorkflowPaths: Component<IWorkflowPathsProps> = (props) => {
  const [explorer] = useExplorer();
  return (
    <svg xmlns="http://www.w3.org/2000/svg" class="w-full h-full">
      <g fill="none" stroke="gray" stroke-width="0.5">
        <For
          each={Object.values(
            explorer.projects[props.projectId].workflowElements,
          )}
        >
          {(element) => (
            <For each={Object.entries(element.edges)}>
              {([edgeLabel, targets]) => (
                <For each={targets}>
                  {(target) => (
                    <WorkflowPath
                      projectId={props.projectId}
                      label={edgeLabel}
                      source={element.id}
                      target={target}
                    />
                  )}
                </For>
              )}
            </For>
          )}
        </For>
      </g>
    </svg>
  );
};

export default WorkflowPaths;
