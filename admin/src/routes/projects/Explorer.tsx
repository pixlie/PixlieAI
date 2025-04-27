import { createElementSize } from "@solid-primitives/resize-observer";
import { useParams } from "@solidjs/router";
import {
  Component,
  createEffect,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import LoaderIcon from "../../assets/icons/tabler-loader.svg";
import { ExplorerProvider, useExplorer } from "../../stores/explorer.tsx";
import { IExplorerWorkflowElements } from "../../utils/types";

interface IInnerProps {
  elements: IExplorerWorkflowElements;
  workflow: string[];
}

const Inner: Component<IInnerProps> = (props: IInnerProps) => {
  return (
    <>
      <details>
        <summary>
          <strong>Workflow</strong>
        </summary>
        <pre>
          <code class="text-wrap break-all">
            {JSON.stringify(props.workflow)}
          </code>
        </pre>
      </details>
      <details>
        <summary>
          <strong>Workflow Elements</strong>
        </summary>
        <pre>
          <code class="text-wrap break-all">
            {JSON.stringify(props.elements, null, 1)}
          </code>
        </pre>
      </details>
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
          explorer.projects[params.projectId].rootElement ? (
            <div class="absolute top-0 left-0 bottom-0 right-0 border">
              <Inner
                elements={explorer.projects[params.projectId].workflowElements}
                workflow={explorer.projects[params.projectId].workflow}
              />
            </div>
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
