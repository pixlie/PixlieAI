import { Component } from "solid-js";
import Heading from "../../widgets/typography/Heading";
import Markdown from "../typography/Markdown";

const help = `We can use either Ollama or Anthropic.
You can either install Ollama locally (if you have a GPU) or use Ollama from a cloud server (you will need to set this up yourself).
Otherwise, you can use Anthropic.`;

const Ollama: Component = () => {
  return (
    <>
      <Heading size={3}>Ollama</Heading>
      <Markdown text={help} />
    </>
  );
};

export default Ollama;
