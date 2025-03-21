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
      for={props.for}
      class={`block text-sm font-medium ${getColors()["label"]}`}
    >
      {props.label}
    </label>
  );
};

export default Label;
