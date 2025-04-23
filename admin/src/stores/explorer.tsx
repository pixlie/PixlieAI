import { batch, Component, createContext, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { EngineResponsePayload } from "../api_types/EngineResponsePayload.ts";
import { getPixlieAIAPIRoot } from "../utils/api";
import { IExplorerStore, IPosition, IProviderPropTypes } from "../utils/types";

const makeStore = () => {
  const [store, setStore] = createStore<IExplorerStore>({
    projects: {},
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
      x1: Math.round(x),
      y1: Math.round(y),
      x2: Math.round(x) + Math.round(width),
      y2: Math.round(y) + Math.round(height),
    });
  };

  const placeNodeOnCanvas = (
    projectId: string,
    width: number,
    height: number,
  ) => {
    // We try to place each node on the canvas, starting from the top left corner
    // Nodes should not overlap

    width = Math.round(width);
    height = Math.round(height);

    // Loop through all existing node positions and find an empty slot for this node
    let x1 = 0;
    let y1 = 0;
    let newPosition: IPosition | undefined = undefined;
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
          x1: x1,
          y1: y1,
          x2: x1 + Math.round(width),
          y2: y1 + Math.round(height),
        };
      } else {
        // Try to find a new position by incrementing x and y
        x1 = overlap.x1 + 50;
        y1 = overlap.y2 + 50;
      }
    }

    setStore(
      "projects",
      projectId,
      "nodePositions",
      store.projects[projectId].nodePositions.length,
      newPosition,
    );
    console.log(newPosition);
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
