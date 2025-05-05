import { batch, Component, createContext, useContext } from "solid-js";
import { createStore, produce } from "solid-js/store";
import { APINodeItem } from "../api_types/APINodeItem.ts";
import { EdgeLabel } from "../api_types/EdgeLabel.ts";
import { EngineResponsePayload } from "../api_types/EngineResponsePayload.ts";
import { NodeLabel } from "../api_types/NodeLabel.ts";
import { getPixlieAIAPIRoot } from "../utils/api";
import { WorkflowElementType } from "../utils/enums.ts";
import {
  IExplorerProject,
  IExplorerStore,
  IExplorerWorkflow,
  IExplorerWorkflowDisplayState,
  IExplorerWorkflowElement,
  IExplorerWorkflowElements,
  IExplorerWorkflowLayer,
  IExplorerWorkflowLayers,
  IExplorerWorkflowNode,
  IProviderPropTypes,
} from "../utils/types";
import { polynomial_rolling_hash } from "../utils/utils.ts";

const makeStore = () => {
  const [store, setStore] = createStore<IExplorerStore>({
    projects: {},
    settings: {
      nodeLabelsOfInterest: [
        "Objective",
        "CrawlerSettings",
        "ProjectSettings",
        "WebSearch",
        "Link",
      ],
      configurableNodeLabels: ["CrawlerSettings"],
      edgeLabelsOfInterest: ["Suggests"],
      horizontalSpacing: 80,
      verticalSpacing: 30,
      horizontalMargin: 30,
      verticalMargin: 20,
    },
  });

  const setProjectId = (projectId: string) => {
    if (!Object.keys(store.projects).includes(projectId)) {
      const newProject: IExplorerProject = {
        nodes: {},
        edges: {},
        siblingNodes: [],
        workflow: [],
        layers: [],
        workflowElements: {},
        displayState: {
          scale: 1,
          size: {
            width: 0,
            height: 0,
          },
          translate: {
            x: 0,
            y: 0,
          },
        },
        rootElement: {
          domState: undefined,
        },
        loaded: false,
        ready: false,
      };
      setStore("projects", projectId, newProject);
    }
  };

  const explore = (projectId: string) => {
    if (!Object.keys(store.projects).includes(projectId)) {
      console.error("Project ID not found in store. Cant explore.");
      return;
    }
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    const fetchStart = new Date().getTime();
    fetch(`${pixlieAIAPIRoot}/api/engine/${projectId}/explore`, {
      headers: { "Content-Type": "application/json" },
    })
      .then((response) => {
        if (!response.ok) {
          throw new Error("Failed to fetch nodes");
        }
        console.info(
          "Time taken to fetch data:",
          new Date().getTime() - fetchStart,
          "ms",
        );
        setStore("projects", projectId, "loaded", true);
        return response.json();
      })
      .then((responsePayload: EngineResponsePayload) => {
        if (responsePayload.type === "Explore") {
          const startTime = new Date().getTime();
          batch(() => {
            setStore(
              produce((state) => {
                const project = state.projects[projectId];
                project.nodes = Object.fromEntries(
                  responsePayload.data.nodes.map((node) => [node.id, node]),
                );
                Object.entries(responsePayload.data.edges).forEach(
                  ([nodeId, edges]) => {
                    if (!!edges) {
                      project.edges[parseInt(nodeId)] = edges;
                    }
                  },
                );
                project.siblingNodes = responsePayload.data.sibling_nodes;
              }),
            );
            console.info(
              "Explore data fetched",
              "Nodes:",
              Object.keys(store.projects[projectId].nodes).length,
              "Edges:",
              Object.keys(store.projects[projectId].edges).length,
              "Sibling Nodes:",
              store.projects[projectId].siblingNodes.length,
            );
            console.info(
              "Workflow Elements before refresh:",
              Object.keys(store.projects[projectId].workflowElements).length,
            );
            const wf_start = new Date().getTime();
            refreshWorkflowElements(projectId);
            buildWorkflowTree(projectId);
            const wf_end = new Date().getTime();
            const wf_time = wf_end - wf_start;
            console.info(
              "Workflow Elements after refresh:",
              Object.keys(store.projects[projectId].workflowElements).length,
              "Time taken to refresh workflow elements and build workflow tree:",
              wf_time,
              "ms",
            );
            setStore("projects", projectId, "ready", true);
            if (!!store.projects[projectId].rootElement.domState) {
              refreshRenderParameters(projectId);
            }
          });
          const endTime = new Date().getTime();
          const timeTaken = endTime - startTime;
          console.info("Total time taken to update store:", timeTaken, "ms");
        }
      });
  };

  const updateRootElement = (
    projectId: string,
    domState: DOMRect | undefined,
  ) => {
    if (!Object.keys(store.projects).includes(projectId)) {
      console.error("Project ID not found. Cant updateRootElement.");
      return;
    }
    batch(() => {
      setStore("projects", projectId, "rootElement", "domState", domState);
      refreshRenderParameters(projectId);
    });
  };

  const ongoingWorkflowElementUpdates = new Map<string, number>();
  const updateWorkflowElement = (
    projectId: string,
    elementId: string,
    domState: DOMRect | undefined,
  ) => {
    if (!Object.keys(store.projects).includes(projectId)) {
      console.error("Project ID not found. Cant updateWorkflowElement.");
      return;
    }
    const key = `${projectId}::${elementId}`;
    const callId = Math.round(performance.now() * 1000);
    ongoingWorkflowElementUpdates.set(key, callId);
    queueMicrotask(() => {
      // Exit early if this is an outdated call
      if (ongoingWorkflowElementUpdates.get(key) !== callId) return;
      batch(() => {
        setStore(
          produce((state) => {
            state.projects[projectId].workflowElements[elementId].state.dom =
              domState;
          }),
        );
        if (ongoingWorkflowElementUpdates.get(key) !== callId) return;
        refreshRenderParameters(projectId);
      });
    });
  };
  const refreshWorkflowElements = (projectId: string) => {
    if (!Object.keys(store.projects).includes(projectId)) {
      console.error("Project ID not found. Cant refreshWorkflowElements.");
      return;
    }
    const project = store.projects[projectId];
    const allSiblingNodeIds = project.siblingNodes.flat();
    const existingWorkflowElementIds = Object.keys(project.workflowElements);
    const nodeLabels: Record<NodeLabel, number> = {} as Record<
      NodeLabel,
      number
    >;
    const nonSiblingNodeIdsWithoutElement: string[] =
      Object.values<APINodeItem>(project.nodes)
        .filter((node) => {
          for (const label of node.labels) {
            if (!nodeLabels[label]) {
              nodeLabels[label] = 1;
            } else nodeLabels[label]++;
          }
          return (
            store.settings.nodeLabelsOfInterest.some((label) =>
              node.labels.includes(label),
            ) &&
            !allSiblingNodeIds.includes(node.id) &&
            !existingWorkflowElementIds.includes(node.id.toString())
          );
        })
        .map((node) => node.id.toString());
    console.info("Node label report:", nodeLabels);
    const siblingGroupsOfInterest = project.siblingNodes
      .map((siblingGroup) =>
        siblingGroup.filter((nodeId) =>
          store.settings.nodeLabelsOfInterest.some((label) =>
            project.nodes[nodeId].labels.includes(label),
          ),
        ),
      )
      .filter((group) => group.length > 0);
    const siblingGroupsHashmap = Object.fromEntries(
      siblingGroupsOfInterest.map((siblingGroup) => [
        polynomial_rolling_hash(siblingGroup),
        siblingGroup,
      ]),
    );
    const siblingGroupIdsWithoutElement: string[] = Object.keys(
      siblingGroupsHashmap,
    ).filter((hash) => !existingWorkflowElementIds.includes(hash));
    const idsToRemoveFromWorkflow = existingWorkflowElementIds.filter(
      (elId) =>
        !nonSiblingNodeIdsWithoutElement.includes(elId) &&
        !siblingGroupIdsWithoutElement.includes(elId),
    );
    const workflowElementsToAdd: IExplorerWorkflowElements = Object.fromEntries(
      nonSiblingNodeIdsWithoutElement
        .map<[string, IExplorerWorkflowElement]>((nodeId) => {
          const node = project.nodes[parseInt(nodeId)];
          const id = polynomial_rolling_hash([node.id]);
          return [
            id,
            {
              id,
              state: { dom: undefined, relative: undefined, layer: 1 },
              labels: node.labels,
              edges: {},
              type: WorkflowElementType.Node,
              nodeIds: [node.id],
            },
          ];
        })
        .concat(
          siblingGroupIdsWithoutElement.map<[string, IExplorerWorkflowElement]>(
            (hash) => {
              return [
                hash,
                {
                  id: hash,
                  state: { dom: undefined, relative: undefined, layer: 1 },
                  labels: Array.from(
                    new Set(
                      siblingGroupsHashmap[hash]
                        .map((nodeId) => project.nodes[nodeId].labels)
                        .flat(),
                    ),
                  ),
                  edges: {},
                  type: WorkflowElementType.NodeSiblingGroup,
                  nodeIds: siblingGroupsHashmap[hash],
                },
              ];
            },
          ),
        ),
    );
    batch(() => {
      setStore(
        produce((state) => {
          // TODO: Remove corresponding node from workflow
          idsToRemoveFromWorkflow.forEach((elId) => {
            delete state.projects[projectId].workflowElements[elId];
          });
        }),
      );
      setStore(
        "projects",
        projectId,
        "workflowElements",
        workflowElementsToAdd,
      );
    });
    const nodeIdToWorkflowElementMap: Record<string, string> = {};
    const workflowElementIds = Object.keys(project.workflowElements);
    workflowElementIds.forEach((elId) => {
      const el = project.workflowElements[elId];
      el.nodeIds.forEach((nodeId) => {
        nodeIdToWorkflowElementMap[nodeId] = elId;
      });
    });
    batch(() => {
      setStore(
        produce((state) => {
          const project = state.projects[projectId];
          const workflowElements = project.workflowElements;
          for (const [sourceNodeId, { edges }] of Object.entries(
            project.edges,
          )) {
            for (const [targetNodeId, edge] of edges) {
              const edgeLabel = edge as EdgeLabel;
              if (store.settings.edgeLabelsOfInterest.includes(edgeLabel)) {
                const sourceWorkflowElement =
                  workflowElements[nodeIdToWorkflowElementMap[sourceNodeId]];
                const targetWorkflowElementId =
                  nodeIdToWorkflowElementMap[targetNodeId];
                if (sourceWorkflowElement && targetWorkflowElementId) {
                  if (!sourceWorkflowElement.edges[edgeLabel]) {
                    sourceWorkflowElement.edges[edgeLabel] = [];
                  }
                  if (
                    !sourceWorkflowElement.edges[edgeLabel].includes(
                      targetWorkflowElementId,
                    )
                  ) {
                    sourceWorkflowElement.edges[edgeLabel].push(
                      targetWorkflowElementId,
                    );
                  }
                }
              }
            }
          }
        }),
      );
    });
  };

  const getTreeBuilder = (projectId: string) => {
    const visited = new Set<string>();

    const workflowElements = store.projects[projectId].workflowElements;

    const treeBuilder = (
      id: string,
      depth = 0,
    ): IExplorerWorkflowNode | undefined => {
      visited.add(id);
      const element = workflowElements[id];
      if (!element) {
        console.warn(`Workflow element with ID ${id} not found.`);
        return undefined;
      }
      setStore(
        produce((state) => {
          const project = state.projects[projectId];
          project.workflowElements[id].state.layer = depth;
        }),
      );
      const children: IExplorerWorkflow = [];
      let treeSize = 0;

      // TODO: Add support for more edge labels
      const suggestEdges = element.edges?.Suggests || [];
      for (const elem in workflowElements) {
        if (suggestEdges.includes(workflowElements[elem].id)) {
          if (!visited.has(workflowElements[elem].id)) {
            const childTree = treeBuilder(workflowElements[elem].id, depth + 1);
            if (childTree) {
              children.push(childTree);
              treeSize += childTree.treeSize;
            }
          }
        }
      }
      treeSize += children.length;

      return { id, treeSize, children: balanceByTreeWeight(children) };
    };
    return treeBuilder;
  };

  const buildWorkflowTree = (
    projectId: string,
  ): IExplorerWorkflow | undefined => {
    if (!Object.keys(store.projects).includes(projectId)) {
      console.error("Project ID not found. Cannot buildWorkflowTree.");
      return undefined;
    }

    const project = store.projects[projectId];
    const workflowElements = project.workflowElements;

    const buildTree = getTreeBuilder(projectId);
    const roots: IExplorerWorkflow = [];
    const allTargets = new Set<string>();

    // TODO: Add support for more edge labels
    // Collect all target node hashes
    Object.values(workflowElements).forEach((el) => {
      const suggests = el.edges?.Suggests || [];
      suggests.forEach((targetId) => {
        allTargets.add(targetId);
      });
    });

    // Find root nodes — nodes NOT targeted by anyone
    Object.keys(workflowElements).forEach((id) => {
      if (!allTargets.has(id)) {
        const rootTree = buildTree(id);
        if (rootTree) {
          roots.push(rootTree);
        }
      }
    });
    batch(() => {
      setStore(
        "projects",
        projectId,
        "workflow",
        balanceByTreeWeight(roots.filter((x) => !!x)),
      );
    });
  };

  const balanceByTreeWeight = (
    arr: IExplorerWorkflow,
    isFirstCall: boolean = true,
    isRight: boolean = false,
  ): IExplorerWorkflow => {
    if (arr.length <= 1) return arr.slice();
    const arrCopy = arr.slice();
    if (arrCopy.every((n) => n.treeSize === arrCopy[0].treeSize))
      return arrCopy;
    const left: IExplorerWorkflow = [];
    const right: IExplorerWorkflow = [];
    if (isFirstCall) {
      arrCopy.sort((a, b) => a.treeSize - b.treeSize);
    }
    const center = arrCopy.pop();
    let toLeft = isRight;
    while (arrCopy.length > 0) {
      if (toLeft) left.unshift(arrCopy.pop()!);
      else right.unshift(arrCopy.pop()!);
      toLeft = !toLeft;
    }
    return [
      ...balanceByTreeWeight(left, false),
      center!,
      ...balanceByTreeWeight(right, false, true),
    ];
  };

  const ongoingLayerParameterUpdates = new Map<string, number>();
  const refreshLayerParameters = (projectId: string) => {
    if (!Object.keys(store.projects).includes(projectId)) {
      console.error("Project ID not found. Cannot refreshLayerParameters.");
      return;
    }
    const key = projectId;
    const callId = Math.round(performance.now() * 1000);
    ongoingLayerParameterUpdates.set(key, callId);
    const project = store.projects[projectId];
    const workflowElements = project.workflowElements;
    const layers: IExplorerWorkflowLayers = [];
    const layerTemplate: IExplorerWorkflowLayer = {
      height: 0,
      width: 0,
    };
    batch(() => {
      Object.values(workflowElements).forEach((element) => {
        if (ongoingLayerParameterUpdates.get(key) !== callId) return;
        const dom = element.state.dom;
        if (dom) {
          if (!layers[element.state.layer]) {
            layers[element.state.layer] = { ...layerTemplate };
          }
          layers[element.state.layer].height +=
            dom.height + store.settings.verticalSpacing;
          if (dom.width > layers[element.state.layer].width) {
            layers[element.state.layer].width = dom.width;
          }
        }
        setStore(
          produce((state) => {
            layers.forEach((layer, index) => {
              if (ongoingLayerParameterUpdates.get(key) !== callId) return;
              if (layer) {
                if (!state.projects[projectId].layers[index]) {
                  state.projects[projectId].layers[index] = layer;
                } else {
                  state.projects[projectId].layers[index].width = layer.width;
                  state.projects[projectId].layers[index].height = layer.height;
                }
              }
            });
          }),
        );
      });

      // TODO: Refactor layer height calculation
      // Need to rethink layer heights for sibling groups instead of full layer
      // Maybe rely on upperBound & lowerBound to calculate height and absolutely
      // shift canvas accordingly = low-hanging fruit and reliable solution.
      // Also, make treeSize comparision to place 2 nodes closer or apart.
      // But, need to reduce the factor as we go deeper into layers.
      setStore(
        produce((state) => {
          state.projects[projectId].layers.forEach((layer) => {
            layer.height = Math.max(
              layer.height +
                2 * store.settings.verticalMargin -
                store.settings.verticalSpacing,
              project.rootElement.domState?.height || 0,
            );
          });
        }),
      );
    });
  };

  const makeElementRenderQueues = (
    tree: IExplorerWorkflow,
    parent: string = "",
  ): Record<string, string[]> => {
    if (tree.length === 0) return {};
    const processQueue = {
      [parent]: tree.map((node) => node.id),
    };
    for (const node of tree) {
      if (node.children.length > 0) {
        const childQueues = makeElementRenderQueues(node.children, node.id);
        for (const queueParent in childQueues) {
          processQueue[queueParent] = childQueues[queueParent];
        }
      }
    }
    return processQueue;
  };

  const ongoingRenderParameterUpdates = new Map<string, number>();
  const refreshRenderParameters = (projectId: string) => {
    if (!Object.keys(store.projects).includes(projectId)) {
      console.error("Project ID not found. Cannot refreshRenderParameters.");
      return;
    }
    const key = projectId;
    const callId = Math.round(performance.now() * 1000);
    ongoingRenderParameterUpdates.set(key, callId);
    const project = store.projects[projectId];
    const workflowElements = project.workflowElements;
    if (ongoingRenderParameterUpdates.get(key) !== callId) return;
    refreshLayerParameters(projectId);
    const renderQueues = makeElementRenderQueues(project.workflow);
    const maxLayerHeight = Math.max(
      ...Object.values(project.layers).map((layer) => layer.height),
    );
    batch(() => {
      for (const queueParent in renderQueues) {
        if (ongoingRenderParameterUpdates.get(key) !== callId) return;
        const renderQueue = renderQueues[queueParent];
        const parent = !!queueParent
          ? workflowElements[queueParent]
          : undefined;
        const parentRel = parent ? parent.state.relative : undefined;
        const parentDom = parent ? parent.state.dom : undefined;
        const orderedRenderIndices: number[] = [];
        const middleIndex = Math.floor(renderQueue.length / 2);
        const isQueueLengthOdd = renderQueue.length % 2 === 1;
        if (isQueueLengthOdd) {
          orderedRenderIndices.push(middleIndex);
        }
        for (let i = 0; i < middleIndex; i++) {
          orderedRenderIndices.push(middleIndex - i - 1);
          orderedRenderIndices.push(middleIndex + i + +isQueueLengthOdd);
        }
        enum Directions {
          up = 1,
          down = -1,
        }
        let direction: Directions = Directions.up;
        interface ILastRenderParameters {
          [Directions.up]?: { height: number; y: number };
          [Directions.down]?: { height: number; y: number };
        }
        let lastRender: ILastRenderParameters = {};
        let renderStarted: boolean = false;
        for (const elementRenderIndex of orderedRenderIndices) {
          const elementId = renderQueue[elementRenderIndex];
          const workflowElement = workflowElements[elementId];
          const dom = workflowElement.state.dom;
          if (!dom) continue;
          const layer = workflowElement.state.layer;
          const { width, height } = dom;

          // TODO: Implement placement based on incoming and outgoing edges ratio
          // More incoming edges tends to left edge of layer
          // More outgoing edges tends to right edge of layer

          // Centered placement
          const x =
            (parent && parentRel && parentDom
              ? parentRel.position.x +
                (project.layers[parent.state.layer].width + parentDom.width) /
                  2 +
                store.settings.horizontalSpacing
              : store.settings.horizontalMargin) +
            (project.layers[layer].width - width) / 2;

          // Right placement
          // const x =
          //   (parentRel && parentDom
          //     ? parentRel.position.x +
          //       parentDom.width +
          //       store.settings.horizontalSpacing
          //     : store.settings.horizontalMargin) +
          //   project.layers[layer].width -
          //   width;

          let y = 0;
          if (!renderStarted) {
            y =
              (parentRel && parentDom
                ? parentRel.position.y + parentDom.height / 2
                : maxLayerHeight / 2) -
              (isQueueLengthOdd
                ? height / 2
                : height + store.settings.verticalMargin / 2);
          } else {
            if (direction == Directions.up) {
              y = lastRender[Directions.up]?.y || 0;
              y -= dom.height + store.settings.verticalSpacing;
            } else if (direction == Directions.down) {
              y =
                (lastRender[Directions.down]?.y ||
                  lastRender[Directions.up]?.y ||
                  0) +
                (lastRender[Directions.down]?.height ||
                  lastRender[Directions.up]?.height ||
                  0);
              y += store.settings.verticalSpacing;
            }
          }
          lastRender[direction] = {
            height,
            y: y,
          };
          direction = -direction;
          renderStarted = true;

          if (
            workflowElements[elementId].state.relative?.position.x !== x ||
            workflowElements[elementId].state.relative?.position.y !== y
          ) {
            setStore(
              produce((state) => {
                const element =
                  state.projects[projectId].workflowElements[elementId];
                element.state.relative = {
                  position: { key: `${x}-${y}`, x, y },
                  size: { w: 0, h: 0 },
                };
              }),
            );
          }
        }
      }
      if (
        project.rootElement.domState &&
        Object.values(workflowElements).every((el) => !!el.state.relative)
      ) {
        if (ongoingRenderParameterUpdates.get(key) !== callId) return;
        const totalWidth =
          2 * store.settings.horizontalMargin +
          Object.values(project.layers).reduce(
            (acc, layer) => acc + layer.width,
            0,
          ) +
          store.settings.horizontalSpacing * (project.layers.length - 1);
        const scale =
          Math.floor(
            Math.min(
              project.rootElement.domState.width / totalWidth || 1,
              project.rootElement.domState.height / maxLayerHeight || 1,
            ) * 100,
          ) / 100;
        const size = {
          width: totalWidth,
          height: maxLayerHeight,
        };
        const translate = {
          x: (project.rootElement.domState.width - size.width * scale) / 2,
          y: (project.rootElement.domState.height - size.height * scale) / 2,
        };
        const style: IExplorerWorkflowDisplayState = {
          scale,
          size,
          translate,
        };
        if (ongoingRenderParameterUpdates.get(key) !== callId) return;
        setStore("projects", projectId, "displayState", style);
      }
      ongoingRenderParameterUpdates.delete(key);
    });
  };

  return [
    store,
    {
      explore,
      setProjectId,
      updateRootElement,
      updateWorkflowElement,
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
