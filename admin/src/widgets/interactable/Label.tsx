import { Component } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses.tsx";

interface IPropTypes {
  label: string;
  for?: string;
}

const Label: Component<IPropTypes> = (props) => {
  const [_, { getColors }] = useUIClasses();

  return (
    <label
      class={`block text-sm font-medium leading-2 mb-1 ${getColors()["label"]}`}
    >
      {props.label}
    </label>
  );
};

export default Label;
