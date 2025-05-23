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

export const identifierToTitle = (str: string) => {
  return (
    str
      .replace(/_/g, " ") // Replace underscores with spaces
      // Add space between lowercase and uppercase letters
      .replace(/([a-z])([A-Z])/g, "$1 $2")
      .toLowerCase()
      .replace(/^./, (match) => match.toUpperCase()) // Capitalize the first letter
      .trim()
  ); // Remove leading/trailing whitespace
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

export const polynomial_rolling_hash = (arr: number[]): string => {
  const base = 53; // A prime number
  const mod = 1e9 + 9; // A large prime to avoid overflow
  let hash = 0; // Initialize hash
  let power = 1; // Initialize power of base

  for (let i = 0; i < arr.length; i++) {
    // Update hash with the current element
    // Mod twice at each step to avoid integer overflow
    hash = (hash + ((arr[i] * power) % mod)) % mod;
    // Update power
    // Mod to avoid integer overflow
    power = (power * base) % mod;
  }

  return hash.toString();
};
