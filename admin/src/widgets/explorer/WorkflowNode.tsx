import { createElementSize } from "@solid-primitives/resize-observer";
import { useParams } from "@solidjs/router";
import {
  Component,
  createEffect,
  createMemo,
  createSignal,
  For,
  onMount,
} from "solid-js";
import { useExplorer } from "../../stores/explorer.tsx";
import { identifierToTitle } from "../../utils/utils.ts";
import Label from "../../widgets/generic/Label.tsx";
import Heading from "../../widgets/typography/Heading.tsx";

interface IWorkflowNodeProps {
  workflowElementId: string;
}

const WorkflowNode: Component<IWorkflowNodeProps> = (
  props: IWorkflowNodeProps,
) => {
  const params = useParams();
  const [explorer, { updateWorkflowElement }] = useExplorer();
  const [workflowNodeRef, setWorkflowNodeRef] = createSignal<HTMLDivElement>();
  const [expanded, setExpanded] = createSignal(true);
  const getWorkflowElement = createMemo(() => {
    if (
      !!params.projectId &&
      explorer.projects[params.projectId] &&
      explorer.projects[params.projectId].workflowElements
    ) {
      return explorer.projects[params.projectId].workflowElements[
        props.workflowElementId
      ];
    }
    return undefined;
  });
  const elemSize = createElementSize(workflowNodeRef);
  onMount(() => {
    getWorkflowElement();
    if (isConfigurable()) {
      setExpanded(false);
    }
    if (workflowNodeRef() && !!elemSize.width && !!elemSize.height) {
      updateWorkflowElement(
        params.projectId as string,
        props.workflowElementId,
        workflowNodeRef()!.getBoundingClientRect(),
      );
    }
  });
  createEffect(() => {
    if (workflowNodeRef() && !!elemSize.width && !!elemSize.height) {
      updateWorkflowElement(
        params.projectId as string,
        props.workflowElementId,
        workflowNodeRef()!.getBoundingClientRect(),
      );
    }
  });
  const getNode = createMemo(() => {
    const workflowElement = getWorkflowElement();
    if (!workflowElement) return undefined;
    const project = explorer.projects[params.projectId];
    return project.nodes[workflowElement.nodeIds[0]];
  });
  const isConfigurable = createMemo(() => {
    const node = getNode();
    if (!node) return false;
    return explorer.settings.configurableNodeLabels.some((label) =>
      node.labels.includes(label),
    );
  });
  const getWorkflowElementId = createMemo(() => {
    const workflowElement = getWorkflowElement();
    if (!workflowElement) return undefined;
    return workflowElement.id;
  });
  const getNodeStyle = createMemo(() => {
    const workflowElement = getWorkflowElement();
    // const displayState = explorer.projects[params.projectId].displayState;
    if (!workflowElement || !workflowElement.state.relative?.position)
      return {
        // "transform": `scale(${displayState.scale})`,
      };
    return {
      // "transform": `scale(${displayState.scale})`,
      left: `${workflowElement.state.relative.position.x}px`,
      top: `${workflowElement.state.relative.position.y}px`,
    };
  });
  const getLabel = createMemo(() => {
    const node = getNode();
    if (!node) return "Label not found";
    const labels = node.labels.filter((label) =>
      explorer.settings.nodeLabelsOfInterest.includes(label),
    );
    if (labels.length > 0) {
      return identifierToTitle(labels[0]);
    }
    return "Label not found";
  });
  const getPayload = createMemo(() => {
    const node = getNode();
    if (!node) return "Payload not found";
    let data = undefined;
    if (node.payload.type === "Text") {
      data = node.payload.data;
    } else if (node.payload.type === "Link") {
      data = (
        <div class="w-full text-wrap break-all">
          {node.payload.data.path +
            (node.payload.data.query ? `?${node.payload.data.query}` : "")}
        </div>
      );
    } else if (node.payload.type == "CrawlerSettings") {
      data = (
        <For each={Object.entries(node.payload.data)}>
          {([key, value]) => {
            return (
              <>
                <div class="font-bold mt-2 mb-1">{identifierToTitle(key)}:</div>
                <div class="flex flex-wrap gap-2 px-1 w-full items-end">
                  {Array.isArray(value) ? (
                    <For each={value}>
                      {(item) => <Label color="button.muted">{item}</Label>}
                    </For>
                  ) : (
                    ""
                  )}
                </div>
              </>
            );
          }}
        </For>
      );
    }
    if (data) {
      return (
        <div
          class="text-sm"
          classList={{
            "max-w-[350px]": ["Text", "Link"].includes(node.payload.type),
            "max-w-[400px]": ["CrawlerSettings"].includes(node.payload.type),
          }}
        >
          {data}
        </div>
      );
    }
    return "Payload not supported yet";
  });
  return !!getWorkflowElement() ? (
    <div
      class="absolute p-2 shadow-lg ring-1 ring-black/5 rounded-xl bg-stone-100 -left-full transition-all duration-1000"
      ref={setWorkflowNodeRef}
      style={getNodeStyle()}
      data-id={getWorkflowElementId()}
    >
      {isConfigurable() && (
        <a
          href="javascript:void(0)"
          class="float-right text-xl font-bold cursor"
          onClick={() => setExpanded((state) => !state)}
        >
          {expanded() ? "-" : "+"}
        </a>
      )}
      <Heading size={6}>{getLabel()}</Heading>
      <div
        class="overflow-clip transition-all duration-500"
        classList={{
          "h-0": !expanded(),
          "h-auto": expanded(),
        }}
      >
        {getPayload()}
      </div>
    </div>
  ) : (
    ""
  );
};

export default WorkflowNode;
