import { useParams } from "@solidjs/router";
import cytoscape from "cytoscape";
import fcose from "cytoscape-fcose";
cytoscape.use(fcose);

import {
  createMemo,
  createSignal,
  onMount,
  onCleanup,
  createEffect,
} from "solid-js";
import { useEngine } from "../../stores/engine";
import { APINodeItem } from "../../api_types/APINodeItem";
import { APINodeEdges } from "../../api_types/APINodeEdges";

type TElement = {
  data: {
    id: string;
    label?: string;
    url?: string;
    color?: string;
    target?: string;
    source?: string;
  };
};

const truncateText = (text: string, maxLength: number = 20): string => {
  return text.length > maxLength ? text.substring(0, maxLength) + "..." : text;
};

const getNodeLabel = (node: APINodeItem): string => {
  let label = node.labels.join(", ");
  if (node.labels.some((label) => label === "Domain")) {
    label = `${node.payload.data}`;
  }
  if (node.payload.type === "Link") {
    label = truncateText(`${node.payload.data.path}`);
  }
  return label;
};

const getNodeUrl = (
  node: APINodeItem,
  nodes: APINodeItem[],
  edges: { [nodeId: number]: APINodeEdges }
): string => {
  let url = `https://${node.payload.data}`;
  if (node.payload.type === "Link") {
    const relatedNodes = [];
    for (const edge of edges[node.id].edges) {
      let [nId]: [number, string] = edge;
      if (nId in nodes) {
        relatedNodes.push(nodes[nId]);
      }
    }
    const path = node.payload?.data?.path?.substring(1);
    const domain = relatedNodes[0]?.payload?.data;
    url = domain && path ? `https://${domain}/${path}` : url;
  }
  return url;
};

const Graph = () => {
  const [engine] = useEngine();
  const params = useParams();

  const [containerRef, setContainerRef] = createSignal<HTMLDivElement | null>(
    null
  );

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

  const getElements = createMemo(() => {
    const nodes = getNodes();
    const edges = getEdges();

    const visibleNodes =
      nodes
        .filter((node) =>
          node.labels?.some(
            (label) =>
              label === "Domain" ||
              (label === "Link" && getNodeLabel(node) !== "/")
          )
        )
        .map((node) => ({
          data: {
            id: node.id.toString(),
            label: getNodeLabel(node),
            url: getNodeUrl(node, nodes, edges),
          },
        })) || ([] as TElement[]);

    const visibleIds = new Set(visibleNodes.map((node) => node.data.id));
    const visibleEdges: TElement[] = [];
    const seenEdges = new Set<string>();

    for (const [source, edgeData] of Object.entries(edges)) {
      if (!visibleIds.has(source)) continue;

      for (const [target] of edgeData.edges) {
        const targetId = target.toString();
        if (!visibleIds.has(targetId)) continue;

        const [a, b] =
          Number(source) < Number(targetId)
            ? [source, targetId]
            : [targetId, source];

        const key = `${a}-${b}`;

        if (seenEdges.has(key)) continue;
        seenEdges.add(key);

        visibleEdges.push({
          data: {
            id: `${source}-${targetId}`,
            source,
            target: targetId,
          },
        });
      }
    }
    return [...visibleNodes, ...visibleEdges];
  });

  let cy: cytoscape.Core | undefined;

  onMount(() => {
    const container = containerRef();
    if (!container || cy) return;

    cy = cytoscape({
      container,
      elements: getElements(),
      style: [
        {
          selector: "node",
          style: {
            label: "data(label)",
            "overlay-opacity": 0,
            width: "label",
            color: "#304FFE",
            "background-color": "white",
            "border-color": "white",
            "border-width": "2px",
            // shape: "ellipse",
            shape: "round-rectangle",
            padding: "6px",
            "text-valign": "center",
            "text-halign": "center",
            "text-max-width": "none",
            "font-size": "16px",
            "transition-property": "font-size",
            "transition-duration": 0.3,
          },
        },
        {
          selector: "edge",
          style: {
            width: 2,
            "overlay-opacity": 0,
            "line-color": "#9E9E9E",
            "source-arrow-shape": "triangle",
            "target-arrow-shape": "triangle",
          },
        },
      ],
      layout: {
        name: "fcose",
        animate: true,
        fit: true,
        spacingFactor: 1.2,
        nodeRepulsion: 10000,
        gravity: 0.5,
        componentSpacing: 200,
        nestingFactor: 0.5,
      } as cytoscape.LayoutOptions,
    });

    cy.fit();
    cy.center();

    cy.on("mouseover", "node", (event) => {
      const node = event.target;
      node.style("font-size", "100%");
      node.style("z-index", "1000");
    });

    cy.on("mouseout", "node", (event) => {
      const node = event.target;
      node.style("font-size", "16px");
      node.style("z-index", "0");
    });

    cy.on("tap", "node", (event) => {
      const node = event.target;
      const url = node.data("url");
      if (url) window.open(url, "_blank");
      node.style("background-color", "#D1C4E9");
      node.style("color", "#6200EA");
    });

    onCleanup(() => {
      cy?.destroy();
      cy = undefined;
    });
  });

  createEffect(() => {
    if (!cy) return;

    const existingIds = new Set(cy.elements().map((ele) => ele.id()));
    const newElements = getElements();
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
        name: "fcose",
        animate: true,
        fit: true,
        spacingFactor: 1.2,
        nodeRepulsion: 10000,
        gravity: 0.5,
        componentSpacing: 200,
        nestingFactor: 0.5,
      } as cytoscape.LayoutOptions).run();
      cy.fit();
      cy.center();
    }
  });

  return (
    <div class="relative flex-1">
      <div ref={setContainerRef} class="absolute w-full h-full" />
    </div>
  );
};

export default Graph;
