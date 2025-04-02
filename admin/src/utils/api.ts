import { NodeWrite } from "../api_types/NodeWrite.ts";
import { EdgeWrite } from "../api_types/EdgeWrite.ts";
import { EngineResponsePayload } from "../api_types/EngineResponsePayload.ts";

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

export const createNode = (projectId: string, node: NodeWrite) => {
  let pixlieAIAPIRoot = getPixlieAIAPIRoot();
  return fetch(`${pixlieAIAPIRoot}/api/engine/${projectId}/nodes`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(node),
  })
    .then((response) => {
      if (!response.ok) {
        throw new Error("Failed to create node");
      }

      return response.json();
    })
    .then((data: EngineResponsePayload) => {
      if (data.type === "NodeCreatedSuccessfully") {
        return data.data;
      }

      return;
    });
};

export const createEdge = (projectId: string, edge: EdgeWrite) => {
  let pixlieAIAPIRoot = getPixlieAIAPIRoot();
  fetch(`${pixlieAIAPIRoot}/api/engine/${projectId}/edges`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(edge),
  }).then((response) => {
    if (!response.ok) {
      throw new Error("Failed to create edge");
    }
  });
};

export const utcStringToLocaleStringAgo = (
  utcString: string | null | undefined,
) => {
  if (!utcString) {
    return "-";
  }
  let date = new Date(utcString);
  let diff = new Date().getTime() - date.getTime();
  let seconds = Math.floor(diff / 1000);
  if (seconds < 60) {
    return "Just now";
  }
  let minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return `${minutes} mins ago`;
  }
  let hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return `${hours} hours ago`;
  }
  let days = Math.floor(hours / 24);
  if (days < 30) {
    return `${days} days ago`;
  }
  let months = Math.floor(days / 30);
  if (months < 12) {
    return `${months} months ago`;
  }
};
