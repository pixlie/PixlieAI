import { Component, createSignal, onCleanup } from "solid-js";
import styles from "./buttons.module.css";

function clickOutside(el: any, accessor: any) {
  const onClick = (e: any) => !el.contains(e.target) && accessor()?.();
  document.body.addEventListener("click", onClick);

  onCleanup(() => document.body.removeEventListener("click", onClick));
}

interface ButtonsProps {
  showDelete: boolean;
  onClickAdd: (numberInputs: number, numberOutputs: number) => void;
  onClickDelete: () => void;
}

const ButtonsComponent: Component<ButtonsProps> = (props: ButtonsProps) => {
  // Signals
  const [isOpen, setIsOpen] = createSignal<boolean>(false);
  const [numberInputs, setNumberInputs] = createSignal<number>(0);
  const [numberOutputs, setNumberOutputs] = createSignal<number>(0);

  // Handlers
  function handleOnClickAdd(event: any) {
    event.stopPropagation();
    setIsOpen(true);
  }

  function handleOnClickAddNode(event: any) {
    event.stopPropagation();

    // Validate number of inputs and outputs
    if (
      numberInputs() > 4 ||
      numberInputs() < 0 ||
      numberOutputs() > 4 ||
      numberOutputs() < 0
    )
      return;

    setIsOpen(false);
    props.onClickAdd(numberInputs(), numberOutputs());
    setNumberInputs(0);
    setNumberOutputs(0);
  }

  function handleChangeNumberInputs(event: any) {
    setNumberInputs(event.target.value);
  }

  function handleChangeNumberOutputs(event: any) {
    setNumberOutputs(event.target.value);
  }

  function handleClickOutsideDropwdown() {
    setIsOpen(false);
    setNumberInputs(0);
    setNumberOutputs(0);
  }

  return (
    <div class={styles.wrapper}>
      <button
        class={
          props.showDelete ? styles.buttonDelete : styles.buttonDeleteHidden
        }
        onClick={props.onClickDelete}
      >
        <svg
          fill="currentColor"
          stroke-width="0"
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 448 512"
          height="1em"
          width="1em"
          style="overflow: visible;"
        >
          <path d="m170.5 51.6-19 28.4h145l-19-28.4c-1.5-2.2-4-3.6-6.7-3.6h-93.7c-2.7 0-5.2 1.3-6.7 3.6zm147-26.6 36.7 55H424c13.3 0 24 10.7 24 24s-10.7 24-24 24h-8v304c0 44.2-35.8 80-80 80H112c-44.2 0-80-35.8-80-80V128h-8c-13.3 0-24-10.7-24-24s10.7-24 24-24h69.8l36.7-55.1C140.9 9.4 158.4 0 177.1 0h93.7c18.7 0 36.2 9.4 46.6 24.9zM80 128v304c0 17.7 14.3 32 32 32h224c17.7 0 32-14.3 32-32V128H80zm80 64v208c0 8.8-7.2 16-16 16s-16-7.2-16-16V192c0-8.8 7.2-16 16-16s16 7.2 16 16zm80 0v208c0 8.8-7.2 16-16 16s-16-7.2-16-16V192c0-8.8 7.2-16 16-16s16 7.2 16 16zm80 0v208c0 8.8-7.2 16-16 16s-16-7.2-16-16V192c0-8.8 7.2-16 16-16s16 7.2 16 16z"></path>
        </svg>
      </button>
      <button class={styles.buttonAdd} onClick={handleOnClickAdd}>
        <svg
          fill="currentColor"
          stroke-width="0"
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 448 512"
          height="1em"
          width="1em"
          style="overflow: visible;"
        >
          <path d="M256 80c0-17.7-14.3-32-32-32s-32 14.3-32 32v144H48c-17.7 0-32 14.3-32 32s14.3 32 32 32h144v144c0 17.7 14.3 32 32 32s32-14.3 32-32V288h144c17.7 0 32-14.3 32-32s-14.3-32-32-32H256V80z"></path>
        </svg>
      </button>

      <div
        class={isOpen() ? styles.dropdown : styles.dropdownHidden}
        //@ts-ignore
        use:clickOutside={handleClickOutsideDropwdown}
      >
        <label class={styles.label}>Number of inputs</label>
        <input
          class={styles.input}
          type="number"
          value={numberInputs()}
          onInput={handleChangeNumberInputs}
        ></input>
        <label class={styles.label}>Number of outputs</label>
        <input
          class={styles.input}
          type="number"
          value={numberOutputs()}
          onInput={handleChangeNumberOutputs}
        ></input>
        <button class={styles.buttonRect} onClick={handleOnClickAddNode}>
          Add node
        </button>
      </div>
    </div>
  );
};

export default ButtonsComponent;
