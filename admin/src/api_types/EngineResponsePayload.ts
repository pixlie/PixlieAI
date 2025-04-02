// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { APIEdges } from "./APIEdges";
import type { APINodeItem } from "./APINodeItem";

export type EngineResponsePayload =
  | { type: "NodeCreatedSuccessfully"; data: number }
  | { type: "EdgeCreatedSuccessfully" }
  | { type: "Nodes"; data: Array<APINodeItem> }
  | { type: "Labels"; data: Array<string> }
  | { type: "Edges"; data: APIEdges }
  | { type: "Error"; data: string };
