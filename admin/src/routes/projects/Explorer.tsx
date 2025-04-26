import { createElementSize } from "@solid-primitives/resize-observer";
import { useParams } from "@solidjs/router";
import {
  Component,
  createEffect,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import { ExplorerProvider, useExplorer } from "../../stores/explorer.tsx";
import { IExplorerWorkflowElements } from "../../utils/types";

interface IInnerProps {
  elements: IExplorerWorkflowElements;
  workflow: string[];
}

const Inner: Component<IInnerProps> = (props: IInnerProps) => {
  return (
    <>
      <strong>Workflow</strong>
      <pre>
        <code class="text-wrap break-all">
          {JSON.stringify(props.workflow)}
        </code>
      </pre>
      <strong>Workflow Elements</strong>
      <pre>
        <code>{JSON.stringify(props.elements, null, 2)}</code>
      </pre>
      <svg xmlns="http://www.w3.org/2000/svg" class="w-full h-full">
        <g fill="none" stroke="gray" stroke-width="0.5">
          {/* <For each={getPaths()}>{(path) => <path d={path} />}</For> */}
        </g>
      </svg>
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
      if (!!explorerSize.width && !!explorerSize.height) {
        updateRootElement(
          params.projectId as string,
          explorerSize.width,
          explorerSize.height,
        );
      }
      explore(params.projectId);
    }
  });
  createEffect(() => {
    if (
      !!params.projectId &&
      Object.keys(explorer.projects).includes(params.projectId) &&
      explorerSize.width &&
      explorerSize.height
    ) {
      updateRootElement(
        params.projectId as string,
        explorerSize.width,
        explorerSize.height,
      );
    }
  });
  onCleanup(() => {
    if (!!params.projectId && explorer.projects[params.projectId]) {
      updateRootElement(params.projectId as string, 0, 0);
    }
  });
  return (
    <ExplorerProvider>
      {params.projectId &&
      !!explorer.projects[params.projectId] &&
      explorer.projects[params.projectId].loaded ? (
        <div class="relative w-full h-full" ref={setExplorerRef}>
          {explorer.projects[params.projectId].rootElement ? (
            <Inner
              elements={explorer.projects[params.projectId].workflowElements}
              workflow={explorer.projects[params.projectId].workflow}
            />
          ) : (
            <div class="flex items-center justify-center w-full h-full">
              <p class="text-gray-500">Preparing to explore the project...</p>
            </div>
          )}
        </div>
      ) : (
        <div class="flex items-center justify-center w-full h-full">
          <p class="text-gray-500">Loading...</p>
        </div>
      )}
    </ExplorerProvider>
  );
};

export default Explorer;
