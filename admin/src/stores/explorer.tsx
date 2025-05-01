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

  const buildWorkflowTree = (
    projectId: string,
  ): IExplorerWorkflow | undefined => {
    if (!Object.keys(store.projects).includes(projectId)) {
      console.error("Project ID not found. Cannot buildWorkflowTree.");
      return undefined;
    }

    const project = store.projects[projectId];
    const workflowElements = project.workflowElements;

    const visited = new Set<string>();

    const buildTree = (
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
          project.workflowElements[id].state.layer = depth + 1;
        }),
      );
      const children: IExplorerWorkflow = [];
      let treeSize = 0;

      // TODO: Add support for more edge labels
      const suggestEdges = element.edges?.Suggests || [];
      for (const elem in workflowElements) {
        if (suggestEdges.includes(workflowElements[elem].id)) {
          if (!visited.has(workflowElements[elem].id)) {
            const childTree = buildTree(workflowElements[elem].id, depth + 1);
            if (childTree) {
              children.push(childTree);
              treeSize += childTree.treeSize;
            }
          }
        }
      }
      treeSize += children.length;

      return {
        id,
        treeSize,
        children,
      };
    };

    const roots: ReturnType<typeof buildTree>[] = [];
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
        roots.filter((x) => !!x),
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

  const ongoingRenderParametersUpdates = new Map<string, number>();
  const refreshRenderParameters = (projectId: string) => {
    if (!Object.keys(store.projects).includes(projectId)) {
      console.error("Project ID not found. Cannot refreshRenderParameters.");
      return;
    }
    const key = projectId;
    const callId = Math.round(performance.now() * 1000);
    ongoingRenderParametersUpdates.set(key, callId);
    const project = store.projects[projectId];
    const workflowElements = project.workflowElements;

    // Track per-layer heights and max width
    const maxWidthByLayer: Record<number, number> = {};
    const layerHeights: Record<string, number> = {};
    const makeProcessQueues = (
      tree: IExplorerWorkflow,
      parent: string = "",
    ): Record<string, string[]> => {
      if (tree.length === 0) return {};
      const processQueue = {
        [parent]: balanceByTreeWeight(tree).map((node) => node.id),
      };
      for (const node of tree) {
        if (node.children.length > 0) {
          const childQueues = makeProcessQueues(node.children, node.id);
          for (const queueParent in childQueues) {
            processQueue[queueParent] = childQueues[queueParent];
          }
        }
        const element = workflowElements[node.id];
        const dom = element.state.dom;
        if (!(element.state.layer in layerHeights)) {
          layerHeights[element.state.layer] = 0;
        }
        if (!(element.state.layer in maxWidthByLayer)) {
          maxWidthByLayer[element.state.layer] = 0;
        }
        if (dom) {
          if (dom.width > maxWidthByLayer[element.state.layer]) {
            maxWidthByLayer[element.state.layer] = dom.width;
          }
          layerHeights[element.state.layer] +=
            dom.height + store.settings.verticalSpacing;
        }
      }
      return processQueue;
    };
    const queues = makeProcessQueues(project.workflow);
    Object.keys(layerHeights).forEach((l) => {
      layerHeights[l] = Math.max(
        layerHeights[l] +
          2 * store.settings.verticalMargin -
          store.settings.verticalSpacing,
        project.rootElement.domState?.height || 0,
      );
    });
    const maxLayerHeight = Math.max(...Object.values(layerHeights));
    batch(() => {
      for (const queueParent in queues) {
        if (ongoingRenderParametersUpdates.get(key) !== callId) return;
        const queue = queues[queueParent];
        const parent = !!queueParent
          ? workflowElements[queueParent]
          : undefined;
        const parentRel = parent ? parent.state.relative : undefined;
        const parentDom = parent ? parent.state.dom : undefined;
        const processingIndices: number[] = [];
        const middleIndex = Math.floor(queue.length / 2);
        const isOdd = queue.length % 2 === 1;
        if (isOdd) {
          processingIndices.push(middleIndex);
        }
        for (let i = 0; i < middleIndex; i++) {
          processingIndices.push(middleIndex - i - 1);
          processingIndices.push(middleIndex + i + +isOdd);
        }
        enum Directions {
          up = 1,
          down = -1,
        }
        let direction: Directions = Directions.up;
        interface IProcessingParameters {
          [Directions.up]?: { height: number; y: number };
          [Directions.down]?: { height: number; y: number };
        }
        let previous: IProcessingParameters = {};
        let initialized: boolean = false;
        for (const elementIndex of processingIndices) {
          const elementId = queue[elementIndex];
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
            (parentRel && parentDom
              ? parentRel.position.x +
                (maxWidthByLayer[layer - 1] + parentDom.width) / 2 +
                store.settings.horizontalSpacing
              : store.settings.horizontalMargin) +
            (maxWidthByLayer[layer] - width) / 2;

          // Right placement
          // const x =
          //   (parentRel && parentDom
          //     ? parentRel.position.x +
          //       parentDom.width +
          //       store.settings.horizontalSpacing
          //     : store.settings.horizontalMargin) +
          //   maxWidthByLayer[layer] -
          //   width;

          let y = 0;
          if (!initialized) {
            y =
              (parentRel && parentDom
                ? parentRel.position.y + parentDom.height / 2
                : maxLayerHeight / 2) -
              (isOdd ? height / 2 : height + store.settings.verticalMargin / 2);
          } else {
            if (direction == Directions.up) {
              y = previous[Directions.up]?.y || 0;
              y -= dom.height + store.settings.verticalSpacing;
            } else if (direction == Directions.down) {
              y =
                (previous[Directions.down]?.y ||
                  previous[Directions.up]?.y ||
                  0) +
                (previous[Directions.down]?.height ||
                  previous[Directions.up]?.height ||
                  0);
              y += store.settings.verticalSpacing;
            }
          }
          previous[direction] = {
            height,
            y: y,
          };
          direction = -direction;
          initialized = true;

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
        if (ongoingRenderParametersUpdates.get(key) !== callId) return;
        const totalWidth =
          2 * store.settings.horizontalMargin +
          Object.values(maxWidthByLayer).reduce(
            (acc, width) => acc + width,
            0,
          ) +
          store.settings.horizontalSpacing *
            (Object.keys(maxWidthByLayer).length - 1);
        const scale = Math.min(
          project.rootElement.domState.width / totalWidth || 1,
          project.rootElement.domState.height / maxLayerHeight || 1,
        );
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
        if (ongoingRenderParametersUpdates.get(key) !== callId) return;
        setStore("projects", projectId, "displayState", style);
      }
      ongoingRenderParametersUpdates.delete(key);
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
