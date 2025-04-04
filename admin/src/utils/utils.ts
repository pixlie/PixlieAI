const camelCaseToSnakeCase = (str: string) => {
  return str.replace(/([A-Z])/g, (match) => `_${match.toLowerCase()}`);
};

// This function converts keys in objects to snake case
export const camelCasedToSnakeCasedKeys = (obj: any): any => {
  if (typeof obj !== "object" || obj === null) {
    return obj;
  }

  if (Array.isArray(obj)) {
    return obj.map((item) => camelCasedToSnakeCasedKeys(item));
  }

  const newObj: any = {};
  Object.keys(obj).forEach((key) => {
    newObj[camelCaseToSnakeCase(key)] = camelCasedToSnakeCasedKeys(obj[key]);
  });

  return newObj;
};

const snakeCaseToCamelCase = (str: string) => {
  return str.replace(/_([a-z])/g, (match) => match[1].toUpperCase());
};

// This function converts keys in objects to camel case
export const snakeCasedToCamelCasedKeys = (obj: any): any => {
  if (typeof obj !== "object" || obj === null) {
    return obj;
  }

  if (Array.isArray(obj)) {
    return obj.map((item) => snakeCasedToCamelCasedKeys(item));
  } else {
    return Object.keys(obj).reduce(
      (newObj, key) => ({
        ...newObj,
        [snakeCaseToCamelCase(key)]: snakeCasedToCamelCasedKeys(obj[key]),
      }),
      {},
    );
  }
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

export const slugify = (str: string) => {
  return str
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^--+$/g, "-")
    .replace(/^-+|-+$/, "");
};
