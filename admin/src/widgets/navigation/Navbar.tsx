import { Component } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses";
import NavbarMenu from "./NavbarMenu.tsx";
import ToolTip from "./ToolTip";
import { A } from "@solidjs/router";

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
          <A href={`${location.pathname}#help`}>
            <svg
              class="w-6 h-6"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <circle cx="12" cy="12" r="10" />
              <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3" />
              <path d="M12 17h.01" />
            </svg>
          </A>
        </ToolTip>
        <ToolTip text="Settings">
          <A href="/settings">
            <svg
              class="w-6 h-6"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <circle cx="12" cy="12" r="3" />
              <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
            </svg>
          </A>
        </ToolTip>
        <NavbarMenu />
      </nav>
    </div>
  );
};

export default Navbar;
