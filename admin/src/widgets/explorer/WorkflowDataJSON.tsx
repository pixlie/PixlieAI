import { Component } from "solid-js";
import { IExplorerWorkflowNode } from "../../utils/types";

const WorkflowDataJSON: Component<{
  workflow: IExplorerWorkflowNode[] | undefined;
  onClose: () => void;
}> = (props) => {
  return props.workflow ? (
    <div class="absolute bottom-0 left-0 w-full h-full p-2 bg-white overflow-auto">
      <pre>
        <code class="text-wrap break-all">
          {JSON.stringify(props.workflow, null, 1)}
        </code>
      </pre>
      <a
        href="javascript:void(0)"
        onClick={props.onClose}
        class="absolute top-1 right-1"
      >
        Close
      </a>
    </div>
  ) : (
    ""
  );
};

export default WorkflowDataJSON;
