import { Component, For } from "solid-js";
import { IExplorerWorkflow } from "../../utils/types";
import WorkflowNode from "../../widgets/explorer/WorkflowNode.tsx";

interface IWorkflowProps {
  workflow: IExplorerWorkflow | undefined;
}

const Workflow: Component<IWorkflowProps> = (props: IWorkflowProps) => {
  if (!props.workflow) {
    return "";
  }
  return (
    <>
      <For each={props.workflow}>
        {(workflowNode) => {
          return <WorkflowNode workflowElementId={workflowNode.id} />;
        }}
      </For>
      <For each={props.workflow}>
        {(workflowNode) =>
          workflowNode.children.length > 0 ? (
            <>
              {workflowNode.children.length > 0 && (
                <Workflow workflow={workflowNode.children} />
              )}
            </>
          ) : (
            <></>
          )
        }
      </For>
    </>
  );
};

export default Workflow;
