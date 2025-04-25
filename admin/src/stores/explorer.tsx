import { batch, Component, createContext, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { EdgeLabel } from "../api_types/EdgeLabel.ts";
import { EngineResponsePayload } from "../api_types/EngineResponsePayload.ts";
import { getPixlieAIAPIRoot } from "../utils/api";
import {
  IExplorerStore,
  INodePosition,
  IProviderPropTypes,
} from "../utils/types";

const makeStore = () => {
  const [store, setStore] = createStore<IExplorerStore>({
    projects: {},
    nodeLabelsOfInterest: ["Objective", "CrawlerSettings"],
    configurableNodeLabels: ["CrawlerSettings"],
    edgeLabelsOfInterest: [
      "SuggestedFor" as EdgeLabel,
      "BelongsTo" as EdgeLabel,
    ],
  });

  const setProjectId = (projectId: string) => {
    if (!Object.keys(store.projects).includes(projectId)) {
      setStore("projects", projectId, {
        nodes: [],
        edges: {},
        siblingNodes: [],
        canvasPosition: {
          x1: 0,
          y1: 0,
          x2: 0,
          y2: 0,
        },
        nodePositions: [],
      });
    }
  };

  const explore = (projectId: string) => {
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(`${pixlieAIAPIRoot}/api/engine/${projectId}/explore`, {
      headers: {
        "Content-Type": "application/json",
      },
    })
      .then((response) => {
        if (!response.ok) {
          throw new Error("Failed to fetch nodes");
        }
        return response.json();
      })
      .then((response: EngineResponsePayload) => {
        if (response.type === "Explore") {
          batch(() => {
            for (const node of response.data.nodes) {
              setStore(
                "projects",
                projectId,
                "nodes",
                store.projects[projectId].nodes.length,
                node,
              );
            }
            for (const [nodeId, edges] of Object.entries(response.data.edges)) {
              if (!!edges) {
                setStore(
                  "projects",
                  projectId,
                  "edges",
                  parseInt(nodeId),
                  edges,
                );
              }
            }
            for (const siblingNodes of response.data.sibling_nodes) {
              setStore(
                "projects",
                projectId,
                "siblingNodes",
                store.projects[projectId].siblingNodes.length,
                siblingNodes,
              );
            }
          });
        }
      });
  };

  const setCanvasPosition = (
    projectId: string,
    x: number,
    y: number,
    width: number,
    height: number,
  ) => {
    setStore("projects", projectId, "canvasPosition", {
      x1: x,
      y1: y,
      x2: x + width,
      y2: y + height,
    });
  };

  const placeNodeOnCanvas = (
    projectId: string,
    nodeIds: number[],
    width: number,
    height: number,
    nearNodeId?: number,
  ) => {
    // We try to place each node on the canvas, starting from the top left corner
    // Nodes should not overlap

    const canvasPosition = store.projects[projectId].canvasPosition;

    let x1: number = 0;
    let y1: number = (canvasPosition.y2 - canvasPosition.y1 - height) / 2;
    let nearNode: INodePosition | undefined;

    const getPositionOnCircleAroundNode = (
      nodePosition: INodePosition,
      angle: number,
    ) => {
      const length = nodePosition.x2 - nodePosition.x1;
      const breadth = nodePosition.y2 - nodePosition.y1;
      const radius = length > breadth ? length : breadth;
      return {
        x:
          (nodePosition.x1 + nodePosition.x2) / 2 +
          radius * Math.cos((angle * Math.PI) / 180),
        y:
          (nodePosition.y1 + nodePosition.y2) / 2 +
          radius * Math.sin((angle * Math.PI) / 180),
      };
    };

    if (nearNodeId) {
      // Find the position of the mentioned "near node"
      nearNode = store.projects[projectId].nodePositions.find((position) => {
        return position.nodeIds.includes(nearNodeId);
      });

      if (nearNode) {
        const positionOnCircle = getPositionOnCircleAroundNode(nearNode, 0);
        x1 = positionOnCircle.x;
        y1 = positionOnCircle.y;
      }
    }

    let newPosition: INodePosition | undefined;

    let loopCount = 0;
    // Loop through all existing node positions and find an empty slot for this node
    while (!newPosition) {
      let overlap = store.projects[projectId].nodePositions.find((existing) => {
        return (
          existing.x1 < x1 + width &&
          existing.y1 < y1 + height &&
          existing.x2 > x1 &&
          existing.y2 > y1
        );
      });
      if (!overlap) {
        newPosition = {
          nodeIds: nodeIds,
          x1: x1,
          y1: y1,
          x2: x1 + width,
          y2: y1 + height,
        };
      } else {
        // Try to find a new position by incrementing x and y
        if (nearNode) {
          // When we are planning to place this node near another, we try to place nodes in a circle.
          // const positionOnCircle = getPositionOnCircleAroundNode(
          //   nearNode,
          //   100,
          //   loopCount * 30,
          // );
          const positionOnCircle = getPositionOnCircleAroundNode(
            nearNode,
            loopCount * 5,
          );
          [x1, y1] = [positionOnCircle.x, positionOnCircle.y];
        } else {
          x1 = overlap.x1 + 50;
          y1 = overlap.y2 + 50;
        }
      }
      loopCount++;
    }

    setStore(
      "projects",
      projectId,
      "nodePositions",
      store.projects[projectId].nodePositions.length,
      newPosition,
    );
    return newPosition;
  };

  return [
    store,
    {
      setProjectId,
      explore,
      setCanvasPosition,
      placeNodeOnCanvas,
    },
  ] as const; // `as const` forces tuple type inference
};

type TStoreAndFunctions = ReturnType<typeof makeStore>;
export const explorerStore = makeStore();

const ExplorerContext = createContext<TStoreAndFunctions>(explorerStore);

export const ExplorerProvider: Component<IProviderPropTypes> = (props) => {
  return (
    <ExplorerContext.Provider value={explorerStore}>
      {props.children}
    </ExplorerContext.Provider>
  );
};

export const useExplorer = () => useContext(ExplorerContext);
