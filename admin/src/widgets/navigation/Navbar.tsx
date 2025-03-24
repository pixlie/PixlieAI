import { Component } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses";
import NavbarMenu from "./NavbarMenu.tsx";
import ToolTip from "./ToolTip";
import { A } from "@solidjs/router";
import Icon from "../interactable/Icon.tsx";

const Navbar: Component = () => {
  const [_, { getColors }] = useUIClasses();

  return (
    <div
      class={
        "fixed w-full h-20 inset-x-0 inset-y-0 z-50 border-b grid grid-cols-3 items-center px-6 gap-6 " +
        getColors()["navBar"]
      }
    >
      <div class="flex items-center gap-6">
        <A
          href="/"
          class={
            "text-3xl font-bold flex items-center gap-6 no-underline " +
            getColors()["navBar.logo"]
          }
        >
          <img
            class="h-auto w-8"
            src="https://pixlie.com/images/logo.png"
            alt="Pixlie"
          />
          Pixlie
        </A>
      </div>

      <nav class="flex items-center justify-center"></nav>

      <nav class="flex items-center justify-end gap-6">
        <ToolTip text="Help">
          <button
            onClick={() => {
              window.location.hash = "#help";
            }}
            aria-label="Help"
          >
            <Icon name="help" />
          </button>
        </ToolTip>
        <ToolTip text="Settings">
          <button
            onClick={() => {
              window.location.hash = "#settings";
            }}
            aria-label="Settings"
          >
            <Icon name="settings" />
          </button>
        </ToolTip>
        <NavbarMenu />
      </nav>
    </div>
  );
};

export default Navbar;
