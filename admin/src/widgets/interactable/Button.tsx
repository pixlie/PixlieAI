import { Component, JSX } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses";

interface IPropTypes {
  size?: "sm" | "base" | "lg";
  label: string;
  isBlock?: boolean;
  onClick?: JSX.EventHandlerUnion<HTMLButtonElement, MouseEvent>;
  href?: string;
  colorTheme?: "cancel" | "secondary" | "success";
}

const Button: Component<IPropTypes> = (props) => {
  const [_, { getColors }] = useUIClasses();

  const getSizeClass = () => {
    switch (props.size) {
      case "sm":
        return "px-2.5 py-1.5 text-sm font-semibold";
      case "lg":
        return "px-6 py-3 text-xl font-semibold";
      case "base":
      default:
        return "px-4 py-2 text-base font-semibold";
    }
  };

  let colorTheme = getColors()["button.default"];
  if (props.colorTheme === "cancel") {
    colorTheme = getColors()["button.cancel"];
  } else if (props.colorTheme === "secondary") {
    colorTheme = getColors()["button.secondary"];
  } else if (props.colorTheme === "success") {
    colorTheme = getColors()["button.success"];
  }

  const buttonClasses =
    getSizeClass() +
    " rounded-md select-none cursor-pointer hover:drop-shadow border-none inline-block hover:box-shadow-md " +
    `${props.isBlock ? "w-full" : ""}` +
    colorTheme;

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
