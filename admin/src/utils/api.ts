export const getPixlieAIAPIRoot = () => {
  let protocol = import.meta.env.VITE_PIXLIE_AI_API_PROTOCOL;
  let host = import.meta.env.VITE_PIXLIE_AI_API_HOST;
  let port = import.meta.env.VITE_PIXLIE_AI_API_PORT;

  if (!host || !port) {
    throw new Error("Pixie AI host and port not set");
  }

  return `${protocol}://${host}:${port}`;
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
