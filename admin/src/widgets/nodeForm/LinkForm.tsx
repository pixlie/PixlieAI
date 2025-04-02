import { Component, createSignal } from "solid-js";
import TextInput from "../interactable/TextInput";
import Button from "../interactable/Button";

interface IPropTypes {
  name?: string;
  onChange?: (name: string, value: string) => void;
}

interface IFormData {
  url: string;
}

const LinkForm: Component<IPropTypes> = (props) => {
  const [formData, setFormData] = createSignal<IFormData>({
    url: "",
  });
  const [error, setError] = createSignal<string>("");

  const handleChange = (_: any, value: string | number) => {
    let url = value as string;
    if (!url.startsWith("https")) {
      setError("Link must start with https://");
    } else {
      setFormData({
        url,
      });
    }
  };

  const handleSubmit = async () => {
    if (!!props.onChange && !!props.name) {
      props.onChange(props.name, formData().url);
      setFormData({
        url: "",
      });
    }
  };

  return (
    <>
      <div class="grid grid-cols-[1fr_auto] gap-x-2">
        <TextInput
          name="url"
          placeholder="https://"
          isEditable
          onChange={handleChange}
          value={formData().url}
        />

        <Button
          label="Add a link"
          colorTheme="secondary"
          onClick={handleSubmit}
        />
      </div>

      {error() && <p class="text-red-600 text-sm">{error()}</p>}
    </>
  );
};

export default LinkForm;
