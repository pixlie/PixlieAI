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
import Heading from "../../widgets/typography/Heading.tsx";
import Paragraph from "../../widgets/typography/Paragraph.tsx";

interface IWorkflowNodeProps {
  workflowElementId: string;
}

const WorkflowNode: Component<IWorkflowNodeProps> = (
  props: IWorkflowNodeProps,
) => {
  const params = useParams();
  const [explorer, { updateWorkflowElement }] = useExplorer();
  const [workflowNodeRef, setWorkflowNodeRef] = createSignal<HTMLDivElement>();
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
  const getLabel = createMemo(() => {
    const node = getNode();
    if (!node) return "Label not found";
    const labels = node.labels.filter((label) =>
      explorer.nodeLabelsOfInterest.includes(label),
    );
    if (labels.length > 0) {
      return labels[0];
    }
    return "Label not found";
  });
  const getPayload = createMemo(() => {
    const node = getNode();
    if (!node) return "Payload not found";
    if (node.payload.type === "Text") {
      return node.payload.data || "Payload empty";
    }
    return "Payload not supported yet";
  });
  onMount(() => {
    getWorkflowElement();
  });
  return !!getWorkflowElement() ? (
    <div
      class="absolute p-2 shadow-lg ring-1 ring-black/5 rounded-xl bg-stone-100 max-w-[50%] top-[50%]"
      ref={setWorkflowNodeRef}
    >
      <Heading size={6}>{getLabel()}</Heading>
      <Paragraph size="sm">{getPayload()}</Paragraph>
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
  return (
    <>
      <For each={props.workflow}>
        {(workflowNode) => {
          return <WorkflowNode workflowElementId={workflowNode.id} />;
        }}
      </For>
      <div class="p-1">
        Layer {props.layer || 1}, {props.workflow.length} workflow nodes
        <For each={props.workflow}>
          {(workflowNode) => (
            <>
              <div>
                Node ID {workflowNode.id}, children:{" "}
                {workflowNode.children.length}
              </div>
              {workflowNode.children.length > 0 ? (
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
              )}
            </>
          )}
        </For>
      </div>
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
  return (
    <ExplorerProvider>
      <div class="relative w-full h-full" ref={setExplorerRef}>
        {params.projectId &&
        !!explorer.projects[params.projectId] &&
        explorer.projects[params.projectId].loaded ? (
          explorer.projects[params.projectId].rootElement &&
          !!explorer.projects[params.projectId].workflow ? (
            <>
              <div class="absolute top-0 left-0 bottom-0 right-0 border">
                <Inner
                  elements={
                    explorer.projects[params.projectId].workflowElements
                  }
                  workflow={explorer.projects[params.projectId].workflow}
                />
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
                    onClick={() =>
                      setShowWorkflowElementsData((state) => !state)
                    }
                  >
                    Workflow Elements Data
                  </a>
                </div>
              </div>
              <div
                class="absolute bottom-0 left-0 w-full h-full p-2 bg-white overflow-auto"
                classList={{
                  hidden: !showWorkflowData(),
                  block: showWorkflowData(),
                }}
              >
                <pre>
                  <code class="text-wrap break-all">
                    {JSON.stringify(
                      explorer.projects[params.projectId].workflow,
                      null,
                      1,
                    )}
                  </code>
                </pre>
                <a
                  href="javascript:void(0)"
                  onClick={() => setShowWorkflowData(false)}
                  class="absolute top-1 right-1"
                >
                  Close
                </a>
              </div>
              <div
                class="absolute bottom-0 left-0 w-full h-full p-2 bg-white overflow-auto"
                classList={{
                  hidden: !showWorkflowElementsData(),
                  block: showWorkflowElementsData(),
                }}
              >
                <pre>
                  <code class="text-wrap break-all">
                    {JSON.stringify(
                      explorer.projects[params.projectId].workflowElements,
                      null,
                      1,
                    )}
                  </code>
                </pre>
                <a
                  href="javascript:void(0)"
                  onClick={() => setShowWorkflowElementsData(false)}
                  class="absolute top-1 right-1"
                >
                  Close
                </a>
              </div>
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
