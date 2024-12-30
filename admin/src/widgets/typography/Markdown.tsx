import { Component } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses";
import { onMount } from "solid-js";
import { marked } from "marked";

interface IPropTypes {
  size?: "sm" | "base" | "lg";
  text: string;
}

const Markdown: Component<IPropTypes> = (props) => {
  const [_, { getColors }] = useUIClasses();
  let paragraphRef: HTMLParagraphElement | undefined;

  const getSizeClass = (size?: string) => {
    switch (size) {
      case "sm":
        return "text-sm";
      case "base":
        return "text-base";
      case "lg":
        return "text-lg";
      default:
        return "text-base";
    }
  };
  const paragraphClasses = `${getSizeClass(
    props.size,
  )} text-gray-300 select-none cursor-default ${getColors().text}`;

  onMount(() => {
    const parsed = marked.parse(props.text);
    if (!!parsed && typeof parsed === "string" && !!paragraphRef) {
      paragraphRef.innerHTML = parsed;
    }
  });

  return <p class={paragraphClasses} ref={paragraphRef}></p>;
};

export default Markdown;
