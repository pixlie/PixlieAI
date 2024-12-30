import { Component, JSX, createMemo } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses";

interface IPropTypes {
  size?: "sm" | "base" | "lg";
  label: string;
  isBlock?: boolean;
  onClick?: JSX.EventHandlerUnion<HTMLButtonElement, MouseEvent>;
  href?: string;
}

const Button: Component<IPropTypes> = (props) => {
  const [_, { getColors }] = useUIClasses();

  const getSizeClass = createMemo(() => {
    switch (props.size) {
      case "sm":
        return "px-2.5 py-1.5 text-sm font-thin";
      case "lg":
        return "px-6 py-3 text-xl font-bold";
      case "base":
      default:
        return "px-4 py-2 text-base font-normal";
    }
  });

  const buttonClasses =
    getSizeClass() +
    " rounded-md select-none cursor-pointer hover:shadow " +
    `${props.isBlock ? "w-full" : ""}` +
    getColors()["button"];

  if (!!props.href) {
    return (
      <a class={buttonClasses} href={props.href}>
        {props.label}
      </a>
    );
  } else if (!!props.onClick) {
    return (
      <button class={buttonClasses} onClick={props.onClick}>
        {props.label}
      </button>
    );
  } else {
    return <button class={buttonClasses}>{props.label}</button>;
  }
};

export default Button;
