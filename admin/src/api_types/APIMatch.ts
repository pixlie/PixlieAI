import { WebMetadata } from "./WebMetadata";

export interface APIMatch {
  node_id: number;
  full_url: string;
  metadata: WebMetadata;
  insight: string;
  reason: string;
}
