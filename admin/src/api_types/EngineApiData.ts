// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Node } from "./Node";

export type EngineApiData = {
  nodes: Array<Node>;
  labels: Array<string>;
  nodes_by_label: { [key in string]?: Array<Node> };
};
