import { NodeWrite } from "../api_types/NodeWrite.ts";

export const getPixlieAIAPIRoot = () => {
  let api = import.meta.env.VITE_PIXLIE_AI_API;

  if (!api) {
    throw new Error("Pixie AI API URL is not set");
  }

  return api;
};

// The API sends and receives objects where the keys use snake_case names
// Our frontend expects keys to be camelCase
const snakeCase = (str: string) => {
  return str.replace(/([A-Z])/g, (match) => `_${match.toLowerCase()}`);
};

// This function converts keys in objects to snake case
export const snakeCasedKeys = (obj: any): any => {
  if (typeof obj !== "object" || obj === null) {
    return obj;
  }

  if (Array.isArray(obj)) {
    return obj.map((item) => snakeCasedKeys(item));
  }

  const newObj: any = {};
  Object.keys(obj).forEach((key) => {
    newObj[snakeCase(key)] = snakeCasedKeys(obj[key]);
  });

  return newObj;
};

const camelCase = (str: string) => {
  return str.replace(/_([a-z])/g, (match) => match[1].toUpperCase());
};

// This function converts keys in objects to camel case
export const camelCasedKeys = (obj: any): any => {
  if (typeof obj !== "object" || obj === null) {
    return obj;
  }

  if (Array.isArray(obj)) {
    return obj.map((item) => camelCasedKeys(item));
  } else {
    return Object.keys(obj).reduce(
      (newObj, key) => ({
        ...newObj,
        [camelCase(key)]: camelCasedKeys(obj[key]),
      }),
      {},
    );
  }
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
