import { Component } from "solid-js";
import { ITextFormField } from "../../utils/types";

const HiddenInput: Component<ITextFormField> = (props) => {
  return (
    <input
      type="hidden"
      name={props.name}
      value={!!props.value ? props.value : ""}
    />
  );
};

export default HiddenInput;
