import { Component } from "solid-js";
import Heading from "../../widgets/typography/Heading";
import Markdown from "../typography/Markdown";

const help = `If you want to use Anthropic, you will need to set up an account and get your API key.`;

const Anthropic: Component = () => {
  return (
    <>
      <Heading size={3}>Anthropic</Heading>
      <Markdown text={help} />
    </>
  );
};

export default Anthropic;
