import { Component } from "solid-js";
import Heading from "../../widgets/typography/Heading";
import Markdown from "../typography/Markdown";

const help = `We can use Ollama or Anthropic for AI models.
You can install Ollama on your computer. It runs locally and is private.
Otherwise, you can use Anthropic, but your queries will be sent to the Anthropic.`;

const Ollama: Component = () => {
  return (
    <>
      <Heading size={3}>Ollama</Heading>
      <Markdown text={help} />
    </>
  );
};

export default Ollama;
