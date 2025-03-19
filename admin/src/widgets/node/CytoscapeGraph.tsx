import { useParams } from "@solidjs/router";
import cytoscape from "cytoscape";
import {
  createMemo,
  createSignal,
  onMount,
  onCleanup,
  createEffect,
} from "solid-js";
import { useEngine } from "../../stores/engine";
import { APINodeItem } from "../../api_types/APINodeItem";
import { NodeLabel } from "../../api_types/NodeLabel";

type TElement = {
  data: {
    id: string;
    label?: string;
    color?: string;
    target?: string;
    source?: string;
  };
};

const getNodeLabel = (node: APINodeItem): string => {
  if (node.labels.some((label) => label === "Objective"))
    return `Objective: ${node.payload.data}`;
  // if (node.labels.some((label) => label === "CrawlCondition"))
  //     return `Crawl Condition: ${node.payload.data}`;
  if (node.labels.some((label) => label === "WebSearch"))
    return `Web Search: ${node.payload.data}`;
  if (node.payload.type === "Link") return `Link: ${node.payload.data.path}`;
  return node.labels.join(", ");
};

const getNodeLabelColor = (labels: NodeLabel[]): string => {
  if (labels.some((label) => label === "AddedByUser")) return "#05df72"; // green
  if (labels.some((label) => label === "AddedByAI")) return "#b28aff"; // purple
  if (labels.some((label) => label === "AddedByWebSearch")) return "#ff9e66"; // orange
  return "#99a1af"; // gray
};

const CytoscapeGraph = () => {
  const [engine] = useEngine();
  const params = useParams();

  const [elements, setElements] = createSignal<TElement[]>([]);
  const [depth, setDepth] = createSignal<number>(0);

  const legend = [
    {
      color: "#05df72",
      label: "Added by User",
    },
    {
      color: "#b28aff",
      label: "Added by AI",
    },
    {
      color: "#ff9e66",
      label: "Added by Web Search",
    },
  ];

  const getProject = createMemo(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  const getNodes = createMemo(() => {
    const project = getProject();
    return project ? Object.values(project.nodes) : [];
  });

  const getEdges = createMemo(() => {
    const project = getProject();
    return project ? project.edges : {};
  });

  const getRootId = createMemo(() => {
    const nodes: APINodeItem[] = getNodes();
    return nodes
      ?.find((node: APINodeItem) =>
        node.labels?.some((label: NodeLabel) => label === "Objective")
      )
      ?.id.toString();
  });

  const getElements = createMemo(() => {
    const nodes: APINodeItem[] = getNodes();
    const edges: { [node_id: number]: [number, string][] } = getEdges();
    const rootId: string = getRootId();

    if (!rootId) return [];

    const formattedNodes: TElement[] = nodes
      ?.filter((node: APINodeItem) =>
        node.labels?.every(
          (label) => label !== "Domain" && label !== "CrawlCondition"
        )
      )
      ?.map((node: APINodeItem) => ({
        data: {
          id: node.id.toString(),
          label: getNodeLabel(node),
          color: getNodeLabelColor(node.labels),
        },
      }));

    let visibleIds = new Set([rootId]);
    let currentLevelIds = new Set([rootId]);

    for (let level = 0; level < depth(); level++) {
      const nextLevelIds = new Set(
        [...currentLevelIds]
          .flatMap((parentId) =>
            edges[Number(parentId)]
              ?.filter(([target]) =>
                formattedNodes?.some(
                  (node) => node.data.id === target.toString()
                )
              )
              ?.map(([target]) => target.toString())
          )
          .filter(Boolean) // Remove undefined values
      );

      if (nextLevelIds.size === 0) break; // Stop if no more levels exist

      visibleIds = new Set([...visibleIds, ...nextLevelIds]);
      currentLevelIds = nextLevelIds;
    }

    const visibleNodes = formattedNodes?.filter((node) =>
      visibleIds?.has(node.data.id)
    );
    const visibleEdges: TElement[] = Object.entries(edges)?.flatMap(
      ([source, targets]) =>
        targets
          ?.filter(
            ([target]) =>
              visibleIds?.has(source) && visibleIds?.has(target.toString())
          )
          ?.map(([target]) => ({
            data: {
              id: `${source}-${target}`,
              source: source.toString(),
              target: target.toString(),
            },
          }))
    );
    return [...visibleNodes, ...visibleEdges];
  });

  createEffect(() => {
    setElements(getElements());
  });

  let containerRef: HTMLDivElement | null = null;
  let cy: cytoscape.Core | undefined;

  onMount(() => {
    if (!containerRef) return;

    cy = cytoscape({
      container: containerRef,
      elements: elements(),
      style: [
        {
          selector: "node",
          style: {
            width: "label",
            "background-color": "data(color)",
            label: "data(label)",
            shape: "roundrectangle",
            padding: "10px",
            "text-valign": "center",
            "text-halign": "center",
            "text-max-width": "none",
          },
        },
        {
          selector: "edge",
          style: {
            width: 3,
            "line-color": "#000000",
          },
        },
      ],
      layout: {
        name: "breadthfirst",
        directed: true,
        fit: true,
        roots: [getRootId()],
      },
    });

    onCleanup(() => {
      cy?.destroy();
      cy = undefined;
    });
  });

  createEffect(() => {
    if (!cy) return;

    const existingIds = new Set(cy.elements().map((ele) => ele.id()));
    const newElements = elements();
    const newIds = new Set(newElements.map((el: TElement) => el.data.id));

    cy.elements().forEach((ele) => {
      if (!newIds.has(ele.id())) ele.remove();
    });

    newElements.forEach((el: TElement) => {
      const existing = cy!.getElementById(el.data.id);
      if (existing.length === 0) {
        cy!.add(el);
      } else {
        existing.data(el.data);
      }
    });

    if (existingIds.size !== newIds.size) {
      cy.layout({
        name: "breadthfirst",
        directed: true,
        fit: true,
        roots: [getRootId()],
      }).run();
    }
  });

  return (
    <div class="relative flex-1">
      <div class="absolute top-0 left-0 z-10 bg-white p-2 rounded shadow-md flex flex-col gap-2">
        <div class="flex items-center gap-2">
          <label for="depth">Depth: </label>
          <input
            id="depth"
            min="0"
            max="10"
            value={depth()}
            onInput={(e) => {
              if (e.currentTarget.value === "") {
                setDepth(0);
                return;
              }
              if (parseInt(e.currentTarget.value) < 0) {
                setDepth(0);
                return;
              }
              if (parseInt(e.currentTarget.value) > 10) {
                setDepth(10);
                return;
              }
              setDepth(parseInt(e.currentTarget.value));
            }}
            class="border px-2 py-1 rounded w-12"
          />
        </div>
          {legend.map(({ color, label }) => (
            <div class="flex items-center gap-2">
              <span
                class="w-4 h-4 rounded"
                style={{ "background-color": color }}
              />
              <span class="text-xs">{label}</span>
            </div>
          ))}
      </div>

      <div ref={(el) => (containerRef = el)} class="absolute w-full h-full" />
    </div>
  );
};

export default CytoscapeGraph;
