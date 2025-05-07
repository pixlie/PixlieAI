import { Component } from "solid-js";
import { IExplorerWorkflowElements } from "../../utils/types";

const WorkflowElementDataJSON: Component<{
  workflowElements: IExplorerWorkflowElements | undefined;
  onClose: () => void;
}> = (props) => {
  return props.workflowElements ? (
    <div class="absolute bottom-0 left-0 w-full h-full p-2 bg-white overflow-auto">
      <pre>
        <code class="text-wrap break-all">
          {JSON.stringify(props.workflowElements, null, 1)}
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

export default WorkflowElementDataJSON;
