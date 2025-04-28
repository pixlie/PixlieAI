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
  IExplorerWorkflowElement,
  IExplorerWorkflowElements,
  IExplorerWorkflowNode,
  IProviderPropTypes,
} from "../utils/types";
import { polynomial_rolling_hash } from "../utils/utils.ts";

const makeStore = () => {
  const [store, setStore] = createStore<IExplorerStore>({
    projects: {},
    nodeLabelsOfInterest: ["Objective", "CrawlerSettings", "WebSearch", "Link"],
    configurableNodeLabels: ["CrawlerSettings"],
    edgeLabelsOfInterest: ["Suggests"],
  });

  const setProjectId = (projectId: string) => {
    if (!Object.keys(store.projects).includes(projectId)) {
      const newProject: IExplorerProject = {
        nodes: {},
        edges: {},
        siblingNodes: [],
        workflow: [],
        workflowElements: {},
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
          const startTime = new Date().getTime();
          batch(() => {
            setStore(
              "projects",
              projectId,
              "nodes",
              Object.fromEntries(
                response.data.nodes.map((node) => [node.id, node]),
              ),
            );
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
              "Time taken to refresh workflow elements:",
              wf_time,
              "ms",
            );
            if (!!store.projects[projectId].rootElement.domState) {
              // Update/refresh the explorer render parameters, viz
              // the positions of workflowElements
            }
            setStore("projects", projectId, "loaded", true);
          });
          const endTime = new Date().getTime();
          const timeTaken = endTime - startTime;
          console.log("Total time taken to update store:", timeTaken, "ms");
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
    setStore("projects", projectId, "rootElement", "domState", domState);
  };

  const updateWorkflowElement = (
    projectId: string,
    elementId: string,
    domState: DOMRect | undefined,
  ) => {
    if (!Object.keys(store.projects).includes(projectId)) {
      console.error("Project ID not found. Cant updateWorkflowElement.");
      return;
    }
    setStore(
      "projects",
      projectId,
      "workflowElements",
      elementId,
      "state",
      "dom",
      domState,
    );
  };
  const refreshWorkflowElements = (projectId: string) => {
    if (!Object.keys(store.projects).includes(projectId)) {
      console.error("Project ID not found. Cant refreshWorkflowElements.");
      return;
    }
    const project = store.projects[projectId];
    const totalSiblingNodeIds = project.siblingNodes.flat();
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
            store.nodeLabelsOfInterest.some((label) =>
              node.labels.includes(label),
            ) &&
            !totalSiblingNodeIds.includes(node.id) &&
            !existingWorkflowElementIds.includes(node.id.toString())
          );
        })
        .map((node) => node.id.toString());
    console.info("Node label report:", nodeLabels);
    const siblingGroupsOfInterest = project.siblingNodes
      .map((siblingGroup) =>
        siblingGroup.filter((nodeId) =>
          store.nodeLabelsOfInterest.some((label) =>
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
              state: { dom: undefined, relative: undefined },
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
                  state: { dom: undefined, relative: undefined },
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
              if (store.edgeLabelsOfInterest.includes(edgeLabel)) {
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

  const buildWorkflowTree = (projectId: string) => {
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

      const children: ReturnType<typeof buildTree>[] = [];

      const suggestEdges = element.edges?.Suggests || [];
      for (const elem in workflowElements) {
        if (suggestEdges.includes(workflowElements[elem].id)) {
          if (!visited.has(workflowElements[elem].id)) {
            const childTree = buildTree(workflowElements[elem].id, depth + 1);
            if (childTree) {
              children.push(childTree);
            }
          }
        }
      }

      return {
        id,
        children,
      };
    };

    const roots: ReturnType<typeof buildTree>[] = [];
    const allTargets = new Set<string>();

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
