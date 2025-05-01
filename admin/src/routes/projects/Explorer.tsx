import { createElementSize } from "@solid-primitives/resize-observer";
import { useParams } from "@solidjs/router";
import {
  Component,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import LoaderIcon from "../../assets/icons/tabler-loader.svg";
import { ExplorerProvider, useExplorer } from "../../stores/explorer.tsx";
import Workflow from "../../widgets/explorer/Workflow.tsx";
import WorkflowDataJSON from "../../widgets/explorer/WorkflowDataJSON.tsx";
import WorkflowElementDataJSON from "../../widgets/explorer/WorkflowElementsDataJSON.tsx";

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
          explorer.projects[params.projectId].ready &&
          !!explorer.projects[params.projectId].workflow ? (
            <>
              <div
                class="absolute top-0 left-0 w-full h-full transform-gpu origin-top-left"
                style={getDisplayStyle()}
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="w-full h-full">
                  <g fill="none" stroke="gray" stroke-width="0.5">
                    {/* <For each={getPaths()}>{(path) => <path d={path} />}</For> */}
                  </g>
                </svg>
                {explorer.projects[params.projectId].workflow && (
                  <Workflow
                    workflow={explorer.projects[params.projectId].workflow}
                  />
                )}
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
            <div class="flex flex-col gap-2 items-center justify-center w-full h-full text-gray-500">
              <LoaderIcon /> Preparing to explore Pixlie's workflow...
            </div>
          )
        ) : (
          <div class="flex flex-col gap-2 items-center justify-center w-full h-full text-gray-500">
            <LoaderIcon /> Loading data...
          </div>
        )}
      </div>
    </ExplorerProvider>
  );
};

export default Explorer;
