export const getPixlieAIAPIRoot = () => {
  let protocol = import.meta.env.VITE_PIXLIE_AI_API_PROTOCOL;
  let host = import.meta.env.VITE_PIXLIE_AI_API_HOST;
  let port = import.meta.env.VITE_PIXLIE_AI_API_PORT;

  if (!host || !port) {
    throw new Error("Pixie AI host and port not set");
  }

  return `${protocol}://${host}:${port}`;
};
