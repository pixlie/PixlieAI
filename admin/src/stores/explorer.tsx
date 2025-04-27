import { batch, Component, createContext, useContext } from "solid-js";
import { createStore, produce } from "solid-js/store";
import { APINodeItem } from "../api_types/APINodeItem.ts";
import { EdgeLabel } from "../api_types/EdgeLabel.ts";
import { EngineResponsePayload } from "../api_types/EngineResponsePayload.ts";
import { NodeLabel } from "../api_types/NodeLabel.ts";
import { getPixlieAIAPIRoot } from "../utils/api";
import { WorkflowElementType } from "../utils/enums.ts";
import {
  IExplorerStore,
  IExplorerWorkflowElement,
  IExplorerWorkflowElements,
  IProviderPropTypes,
} from "../utils/types";
import { polynomial_rolling_hash } from "../utils/utils.ts";

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
      });
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
            !totalSiblingNodeIds.includes(node.id) &&
            !existingWorkflowElementIds.includes(node.id.toString())
          );
        })
        .map((node) => node.id.toString());
    console.info("Node label report:", nodeLabels);
    const siblingGroupsHashmap = Object.fromEntries(
      project.siblingNodes.map((siblingGroup) => [
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
          idsToRemoveFromWorkflow.forEach((elId) => {
            delete state.projects[projectId].workflowElements[elId];
            if (elId in state.projects[projectId].workflow) {
              state.projects[projectId].workflow.splice(
                state.projects[projectId].workflow.indexOf(elId),
                1,
              );
            }
          });
        }),
      );
      setStore(
        "projects",
        projectId,
        "workflowElements",
        workflowElementsToAdd,
      );
      Object.keys(workflowElementsToAdd).forEach((elId) => {
        setStore(
          "projects",
          projectId,
          "workflow",
          store.projects[projectId].workflow.length,
          elId,
        );
      });
    });
  };

  return [
    store,
    {
      explore,
      setProjectId,
      updateRootElement,
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
