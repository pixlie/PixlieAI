import { Component, JSX } from "solid-js";
import { IFormField } from "../../utils/types";
import { useUIClasses } from "../../stores/UIClasses";

const TextInput: Component<IFormField> = (props) => {
  const [_, { getColors }] = useUIClasses();

  let inputType: string = "text";

  const handleChange: JSX.ChangeEventHandler<HTMLInputElement, Event> = (
    event,
  ) => {
    if (!!props.onChange) {
      props.onChange(props.name, event.currentTarget.value);
    }
  };

  return (
    <input
      id={props.id}
      name={props.name}
      type={inputType}
      required={props.isRequired || undefined}
      class={`block w-full rounded px-3 py-2.5 border font-content outline-none m-0 ${getColors()["input"]}`}
      placeholder={props.placeholder || undefined}
      value={props.value !== undefined ? props.value : ""}
      onChange={handleChange}
      onFocus={props.onFocus}
      onKeyUp={props.onKeyUp}
      disabled={!props.isEditable}
      autocomplete={props.autocomplete ? "on" : "off"}
    />
  );
};

export default TextInput;
