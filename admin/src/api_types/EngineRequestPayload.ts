// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { EdgeWrite } from "./EdgeWrite";
import type { NodeWrite } from "./NodeWrite";

export type EngineRequestPayload =
  | { Explore: number | null }
  | "GetLabels"
  | "GetEntities"
  | "GetClassifications"
  | { GetNodesWithLabel: string }
  | { GetNodesWithIds: Array<number> }
  | { GetAllNodes: bigint }
  | { GetAllEdges: bigint }
  | { CreateNode: NodeWrite }
  | { CreateEdge: EdgeWrite }
  | { Query: number };
