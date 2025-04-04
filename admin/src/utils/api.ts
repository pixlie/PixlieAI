import { NodeWrite } from "../api_types/NodeWrite.ts";

export const getPixlieAIAPIRoot = () => {
  let api = import.meta.env.VITE_PIXLIE_AI_API;

  if (!api) {
    throw new Error("Pixie AI API URL is not set");
  }

  return api;
};

export const insertNode = (projectId: string, node: NodeWrite) => {
  let pixlieAIAPIRoot = getPixlieAIAPIRoot();
  fetch(`${pixlieAIAPIRoot}/api/engine/${projectId}/nodes`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(node),
  }).then((response) => {
    if (!response.ok) {
      throw new Error("Failed to insert node");
    }
  });
};
