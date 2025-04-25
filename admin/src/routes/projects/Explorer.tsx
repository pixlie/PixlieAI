import { useParams } from "@solidjs/router";
import { Component, createEffect, createMemo, For, onMount } from "solid-js";
import { APINodeItem } from "../../api_types/APINodeItem.ts";
import { CrawlerSettings } from "../../api_types/CrawlerSettings.ts";
import { EdgeLabel } from "../../api_types/EdgeLabel.ts";
import { Link } from "../../api_types/Link.ts";
import { NodeLabel } from "../../api_types/NodeLabel.ts";
import { ExplorerProvider, useExplorer } from "../../stores/explorer.tsx";
import Heading from "../../widgets/typography/Heading.tsx";
import Paragraph from "../../widgets/typography/Paragraph.tsx";

// const edgeLabelsOfInterest: EdgeLabel[] = [
//   "SuggestedFor" as EdgeLabel,
//   "BelongsTo" as EdgeLabel,
// ];

interface IListOfCollapsibleTextsProps {
  texts: string[];
}

const ListOfCollapsibleTexts: Component<IListOfCollapsibleTextsProps> = (
  props,
) => {
  // Show 3 items by default and add a button to show more

  return (
    <div class="flex flex-row flex-wrap gap-2">
      <For each={props.texts.slice(0, 3)}>
        {(text) => <span class="bg-gray-300 rounded px-2">{text}</span>}
      </For>
    </div>
  );
};

const NodeDisplay: Component<APINodeItem> = (props: APINodeItem) => {
  const [explorer, { placeNodeOnCanvas }] = useExplorer();
  const params = useParams();
  let elementRef: HTMLDivElement | undefined;

  onMount(() => {
    if (params.projectId && elementRef) {
      let nearNodeId: number | undefined;
      if (explorer.projects[params.projectId].edges[props.id]) {
        let nearEdge = explorer.projects[params.projectId].edges[
          props.id
        ].edges.find((nodeIdAndEdgeLabel) =>
          explorer.edgeLabelsOfInterest.includes(
            nodeIdAndEdgeLabel[1] as EdgeLabel,
          ),
        );

        if (nearEdge) {
          nearNodeId = nearEdge[0];
        }
      }

      let position = placeNodeOnCanvas(
        params.projectId as string,
        [props.id],
        elementRef.getBoundingClientRect().width,
        elementRef.getBoundingClientRect().height,
        nearNodeId,
      );

      // Place the node at the given position
      elementRef.style.left = `${position.x1}px`;
      elementRef.style.top = `${position.y1}px`;
    }
  });

  // const nodeLabelsOfInterest = ["Objective", "CrawlerSettings", "WebSearch"];
  return (
    <>
      {explorer.nodeLabelsOfInterest.some((label) =>
        props.labels.includes(label as NodeLabel),
      ) ? (
        <div
          class="absolute px-4 py-6 shadow-lg ring-1 ring-black/5 rounded bg-stone-100 max-w-[50%]"
          ref={elementRef}
          data-id={props.id}
        >
          {props.labels.includes("Objective" as NodeLabel) && (
            <>
              <Heading size={4}>Objective</Heading>
              <Paragraph>{props.payload.data as string}</Paragraph>
            </>
          )}
          {props.labels.includes("CrawlerSettings" as NodeLabel) && (
            <>
              <Heading size={4}>Crawler Settings</Heading>
              <Paragraph>
                Crawl link if anchor text has any of these keywords:
              </Paragraph>
              <ListOfCollapsibleTexts
                texts={
                  (props.payload.data as CrawlerSettings)
                    .crawl_link_if_anchor_text_has_any_of_these_keywords || []
                }
              />

              <Paragraph>Web search with following keywords:</Paragraph>
              <ListOfCollapsibleTexts
                texts={
                  (props.payload.data as CrawlerSettings)
                    .keywords_to_search_the_web_to_get_starting_urls || []
                }
              />
            </>
          )}
          {props.labels.includes("WebSearch" as NodeLabel) && (
            <>
              <Paragraph size={"sm"}>
                Web Search keywords: {props.payload.data as string}
              </Paragraph>
            </>
          )}
        </div>
      ) : (
        <></>
      )}
    </>
  );
};

interface INodeGroupDisplayProps {
  nodeIds: number[];
}

interface InnerProps {
  parent: HTMLDivElement;
}

const NodeGroupDisplay: Component<INodeGroupDisplayProps> = (
  props: INodeGroupDisplayProps,
) => {
  const [explorer, { placeNodeOnCanvas }] = useExplorer();
  const params = useParams();
  let elementRef: HTMLDivElement | undefined;

  // Get the first node in the group
  const firstNode = createMemo(() => {
    if (
      params.projectId &&
      Object.keys(explorer.projects).includes(params.projectId)
    ) {
      return explorer.projects[params.projectId].nodes.find((node) =>
        props.nodeIds.includes(node.id),
      );
    } else {
      return undefined;
    }
  });

  onMount(() => {
    if (params.projectId && elementRef && firstNode()) {
      let nearNodeId: number | undefined;
      if (explorer.projects[params.projectId].edges[firstNode()!.id]) {
        let nearEdge = explorer.projects[params.projectId].edges[
          firstNode()!.id
        ].edges.find((nodeIdAndEdgeLabel) =>
          explorer.edgeLabelsOfInterest.includes(
            nodeIdAndEdgeLabel[1] as EdgeLabel,
          ),
        );

        if (nearEdge) {
          nearNodeId = nearEdge[0];
        }
      }

      let position = placeNodeOnCanvas(
        params.projectId as string,
        props.nodeIds,
        elementRef.getBoundingClientRect().width,
        elementRef.getBoundingClientRect().height,
        nearNodeId,
      );

      // Place the node at the given position
      elementRef.style.left = `${position.x1}px`;
      elementRef.style.top = `${position.y1}px`;
    }
  });

  return (
    <div
      class="absolute px-4 py-6 shadow-lg ring-1 ring-black/5 rounded bg-stone-100 max-w-[50%]"
      ref={elementRef}
      data-ids={props.nodeIds}
    >
      {firstNode()?.labels.includes("WebSearch" as NodeLabel) && (
        <>
          <span class="text-sm">Web search keyword:</span>{" "}
          <span class="bg-gray-300 rounded px-2 text-sm">
            {firstNode()?.payload.data as string}
          </span>
          <div class="text-xs">{props.nodeIds.length - 1} more</div>
        </>
      )}
      {firstNode()?.labels.includes("Link" as NodeLabel) && (
        <>
          <span class="text-sm">
            Link: {(firstNode()?.payload.data as Link).path}
          </span>
          <div class="text-xs">{props.nodeIds.length - 1} more</div>
        </>
      )}
    </div>
  );
};

const Inner: Component<InnerProps> = (props: InnerProps) => {
  const [explorer, { setProjectId, explore, setCanvasPosition }] =
    useExplorer();
  const params = useParams();
  let canvasRef = props.parent;

  onMount(() => {
    if (
      !!params.projectId &&
      !Object.keys(explorer.projects).includes(params.projectId)
    ) {
      setProjectId(params.projectId);
      setTimeout(() => {
        queueMicrotask(() => {
          setCanvasPosition(
            params.projectId as string,
            0,
            0,
            canvasRef.clientWidth,
            canvasRef.clientHeight,
          );
          explore(params.projectId);
        });
      }, 150);
    }
  });

  createEffect((prevProjectId: string | void) => {
    if (!!params.projectId) {
      if (
        prevProjectId !== params.projectId &&
        !Object.keys(explorer.projects).includes(params.projectId)
      ) {
        // When changing to a new project from projects dropdown
        setProjectId(params.projectId);
        explore(params.projectId);
      }
    }
  }, params.projectId);

  const getNonSiblingNodes = createMemo<APINodeItem[]>(() => {
    if (
      !!params.projectId &&
      Object.keys(explorer.projects).includes(params.projectId)
    ) {
      // For each node in the explorer store, filter only the ones that are not siblings of any other node
      // Create a flat array of all sibling nodes
      const siblingNodes =
        explorer.projects[params.projectId].siblingNodes.flat();
      return explorer.projects[params.projectId].nodes.filter(
        (node) => !siblingNodes.includes(node.id),
      );
    } else {
      return [];
    }
  });

  const getSiblingNodeIds = createMemo<number[][]>(() => {
    if (
      !!params.projectId &&
      Object.keys(explorer.projects).includes(params.projectId)
    ) {
      return explorer.projects[params.projectId].siblingNodes;
    } else {
      return [];
    }
  });

  const getPaths = createMemo<string[]>(() => {
    // When nodes are displayed, draw the edges using SVG paths
    if (
      !!params.projectId &&
      Object.keys(explorer.projects).includes(params.projectId)
    ) {
      const allNodeIdsDisplayed = explorer.projects[
        params.projectId
      ].nodePositions.flatMap((nodePosition) => nodePosition.nodeIds);
      const allEdgeKeys = Object.keys(
        explorer.projects[params.projectId].edges,
      );
      let paths: string[] = [];
      for (const nodePosition of explorer.projects[params.projectId]
        .nodePositions) {
        // A node display may represent multiple nodes
        // Far each node display, find all connected (and displayed) nodes except those in this node display itself
        // Then create SVG paths from this node display to each of the other
        const myCenterX = (nodePosition.x1 + nodePosition.x2) / 2;
        const myCenterY = (nodePosition.y1 + nodePosition.y2) / 2;
        for (const nodeId of nodePosition.nodeIds) {
          if (!allEdgeKeys.includes(String(nodeId))) {
            continue;
          }

          for (const otherNodeId of explorer.projects[params.projectId].edges[
            nodeId
          ].edges
            .filter(
              (edge) =>
                explorer.edgeLabelsOfInterest.includes(edge[1] as EdgeLabel) &&
                allNodeIdsDisplayed.includes(edge[0]),
            )
            .map((edge) => edge[0])) {
            const otherNode = explorer.projects[
              params.projectId
            ].nodePositions.find((nodePosition) =>
              nodePosition.nodeIds.includes(otherNodeId),
            );
            if (otherNode) {
              const otherCenterX = (otherNode.x1 + otherNode.x2) / 2;
              const otherCenterY = (otherNode.y1 + otherNode.y2) / 2;

              const midX = (myCenterX + otherCenterX) / 2;
              const midY = (myCenterY + otherCenterY) / 2;
              const theta =
                Math.atan2(otherCenterY - myCenterY, otherCenterX - myCenterX) -
                Math.PI / 2;
              const offset = 120;

              const controlX = midX + offset * Math.cos(theta);
              const controlY = midY + offset * Math.sin(theta);

              paths.push(
                `M ${myCenterX} ${myCenterY} Q ${controlX} ${controlY} ${otherCenterX} ${otherCenterY}`,
              );
            }
          }
        }
      }
      return paths;
    } else {
      return [];
    }
  });

  return (
    <>
      <svg xmlns="http://www.w3.org/2000/svg" class="w-full h-full">
        <g fill="none" stroke="gray" stroke-width="0.5">
          <For each={getPaths()}>{(path) => <path d={path} />}</For>
        </g>
      </svg>
      <For each={getNonSiblingNodes()}>
        {(node) => <NodeDisplay {...node} />}
      </For>
      <For each={getSiblingNodeIds()}>
        {(nodeIds) => <NodeGroupDisplay nodeIds={nodeIds} />}
      </For>
    </>
  );
};

const Explorer: Component = () => {
  // The node editor is created with free positioned divs that can be dragged and dropped.
  // Each node's position can be saved (later). Nodes are connected with edges which are SVG paths
  let canvasRef!: HTMLDivElement;
  return (
    <ExplorerProvider>
      <div class="relative w-full h-full" ref={canvasRef}>
        <Inner parent={canvasRef} />
      </div>
    </ExplorerProvider>
  );
};

export default Explorer;
