import { Component, JSX } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses";

type ParagraphSize = "sm" | "base" | "lg";

interface IPropTypes {
  size?: ParagraphSize;
  children: JSX.Element | string;
}

const Paragraph: Component<IPropTypes> = (props) => {
  const [_, { getColors }] = useUIClasses();

  const getSizeClass = (size?: ParagraphSize) => {
    switch (size) {
      case "sm":
        return "text-sm";
      case "lg":
        return "text-lg";
      case "base":
      default:
        return "text-base";
    }
  };
  const paragraphClasses = `${getSizeClass(
    props.size,
  )} text-gray-300 select-none cursor-default ${getColors().text}`;

  return <p class={paragraphClasses}>{props.children}</p>;
};

export default Paragraph;
