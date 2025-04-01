import { Component, JSX } from "solid-js";
import { IFormField } from "../../utils/types";

const PromptInput: Component<IFormField> = (props) => {

  const handleChange: JSX.ChangeEventHandler<HTMLTextAreaElement, Event> = (
    event
  ) => {
    if (!!props.onChange) {
      props.onChange(props.name, event.currentTarget.value);
    }
  };

  return (
    <textarea
      id={props.id}
      name={props.name}
      required={props.isRequired !== null ? props.isRequired : false}
      class={`block w-full rounded-lg px-2 py-2 outline-none font-content m-0 bg-transparent`}
      placeholder={props.placeholder !== null ? props.placeholder : ""}
      value={props.value || ""}
      onChange={handleChange}
      disabled={!props.isEditable}
      style={{ height: "100px", resize: "none" }}
    />
  );
};

export default PromptInput;
