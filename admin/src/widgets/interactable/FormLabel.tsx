import { Component } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses.tsx";

interface IPropTypes {
  label: string;
  for?: string;
}

const FormLabel: Component<IPropTypes> = (props) => {
  const [_, { getColors }] = useUIClasses();

  return (
    <label
      for={props.for}
      class={`block text-sm font-medium ${getColors()["form.label"]}`}
    >
      {props.label}
    </label>
  );
};

export default FormLabel;
