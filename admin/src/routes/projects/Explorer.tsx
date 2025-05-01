import { createElementSize } from "@solid-primitives/resize-observer";
import { useParams } from "@solidjs/router";
import {
  Component,
  createEffect,
  createMemo,
  createSignal,
  For,
  onCleanup,
  onMount,
} from "solid-js";
import LoaderIcon from "../../assets/icons/tabler-loader.svg";
import { ExplorerProvider, useExplorer } from "../../stores/explorer.tsx";
import {
  IExplorerWorkflowElements,
  IExplorerWorkflowNode,
} from "../../utils/types";
import { identifierToTitle } from "../../utils/utils.ts";
import Label from "../../widgets/generic/Label.tsx";
import Heading from "../../widgets/typography/Heading.tsx";

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

interface IWorkflowNodeProps {
  workflowElementId: string;
  layer: number;
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

interface IWorkflowProps {
  workflow: IExplorerWorkflowNode[] | undefined;
  layer?: number;
}

const Workflow: Component<IWorkflowProps> = (props: IWorkflowProps) => {
  if (!props.workflow) {
    return "";
  }
  onMount(() => {
    console.log("Layer", props.layer || 1, "mounted");
  });
  return (
    <>
      <For each={props.workflow}>
        {(workflowNode) => {
          return (
            <WorkflowNode
              workflowElementId={workflowNode.id}
              layer={props.layer || 1}
            />
          );
        }}
      </For>
      <For each={props.workflow}>
        {(workflowNode) =>
          workflowNode.children.length > 0 ? (
            <>
              {workflowNode.children.length > 0 && (
                <Workflow
                  workflow={workflowNode.children}
                  layer={(props.layer || 1) + 1}
                />
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

interface IInnerProps {
  elements: IExplorerWorkflowElements;
  workflow: IExplorerWorkflowNode[] | undefined;
}

const Inner: Component<IInnerProps> = (props: IInnerProps) => {
  return (
    <>
      <svg xmlns="http://www.w3.org/2000/svg" class="w-full h-full">
        <g fill="none" stroke="gray" stroke-width="0.5">
          {/* <For each={getPaths()}>{(path) => <path d={path} />}</For> */}
        </g>
      </svg>
      {props.workflow && <Workflow workflow={props.workflow} />}
      {/* <For each={getSiblingNodeIds()}>
        {(nodeIds) => <NodeGroupDisplay nodeIds={nodeIds} />}
      {/* <For each={getNonSiblingNodes()}>
        {(node) => <NodeDisplay {...node} />}
      </For>
      <For each={getSiblingNodeIds()}>
        {(nodeIds) => <NodeGroupDisplay nodeIds={nodeIds} />}
      </For> */}
    </>
  );
};

const Explorer: Component = () => {
  // The node editor is created with free positioned divs that can be dragged and dropped.
  // Each node's position can be saved (later). Nodes are connected with edges which are SVG paths
  let [explorerRef, setExplorerRef] = createSignal<HTMLDivElement>();
  let [showWorkflowData, setShowWorkflowData] = createSignal(false);
  let [showWorkflowElementsData, setShowWorkflowElementsData] =
    createSignal(false);
  const params = useParams();
  const [explorer, { setProjectId, explore, updateRootElement }] =
    useExplorer();
  const explorerSize = createElementSize(explorerRef);

  onMount(() => {
    if (
      !!params.projectId &&
      !Object.keys(explorer.projects).includes(params.projectId)
    ) {
      setProjectId(params.projectId);
      explore(params.projectId);
      if (explorerRef() && !!explorerSize.width && !!explorerSize.height) {
        updateRootElement(
          params.projectId as string,
          explorerRef()!.getBoundingClientRect(),
        );
      }
    }
  });
  createEffect(() => {
    if (
      !!params.projectId &&
      Object.keys(explorer.projects).includes(params.projectId) &&
      explorerRef() &&
      explorerSize.width &&
      explorerSize.height
    ) {
      updateRootElement(
        params.projectId as string,
        explorerRef()!.getBoundingClientRect(),
      );
    }
  });
  onCleanup(() => {
    if (!!params.projectId && explorer.projects[params.projectId]) {
      updateRootElement(params.projectId as string, undefined);
    }
  });
  const getDisplayStyle = createMemo(() => {
    if (
      !!params.projectId &&
      explorer.projects[params.projectId] &&
      explorer.projects[params.projectId].displayState
    ) {
      const displayState = explorer.projects[params.projectId].displayState;
      return {
        // transform: `scale(${displayState.scale})`,
        width: displayState.size.width
          ? `${displayState.size.width}px`
          : "100%",
        height: displayState.size.height
          ? `${displayState.size.height}px`
          : "100%",
        // translate: `translate(${displayState.translate.x}px, ${displayState.translate.y}px)`,
      };
    }
    return {};
  });
  return (
    <ExplorerProvider>
      <div
        class="relative w-full h-full border overflow-auto"
        ref={setExplorerRef}
      >
        {params.projectId &&
        !!explorer.projects[params.projectId] &&
        explorer.projects[params.projectId].loaded ? (
          explorer.projects[params.projectId].rootElement &&
          !!explorer.projects[params.projectId].workflow ? (
            <>
              <div
                class="absolute top-0 left-0 w-full h-full transform-gpu origin-top-left"
                style={getDisplayStyle()}
              >
                <Inner
                  elements={
                    explorer.projects[params.projectId].workflowElements
                  }
                  workflow={explorer.projects[params.projectId].workflow}
                />
              </div>
              <div class="absolute top-1 right-1">
                <a
                  href="javascript:void(0)"
                  onClick={() => setShowWorkflowData((state) => !state)}
                >
                  Workflow Data
                </a>
                {" | "}
                <a
                  href="javascript:void(0)"
                  onClick={() => setShowWorkflowElementsData((state) => !state)}
                >
                  Workflow Elements Data
                </a>
              </div>
              {showWorkflowData() && (
                <WorkflowDataJSON
                  workflow={explorer.projects[params.projectId].workflow}
                  onClose={() => setShowWorkflowData(false)}
                />
              )}
              {showWorkflowElementsData() && (
                <WorkflowElementDataJSON
                  workflowElements={
                    explorer.projects[params.projectId].workflowElements
                  }
                  onClose={() => setShowWorkflowElementsData(false)}
                />
              )}
            </>
          ) : (
            <div class="flex items-center justify-center w-full h-full text-gray-500">
              <LoaderIcon /> Preparing to explore the project...
            </div>
          )
        ) : (
          <div class="flex items-center justify-center w-full h-full text-gray-500">
            <LoaderIcon /> Loading...
          </div>
        )}
      </div>
    </ExplorerProvider>
  );
};

export default Explorer;
