import { Component } from "solid-js";
import { A } from "@solidjs/router";
import HelpPopOver from "../../routes/help/HelpPopOver.tsx";
import SettingsPopOver from "../../routes/settings/SettingsPopOver.tsx";
import ProjectsPopOver from "../../routes/projects/ProjectsPopOver.tsx";
import GitHubStars from "./GitHubStars.tsx";

const Navbar: Component = () => {
  return (
    <div class="fixed w-full h-20 my-1 inset-x-0 inset-y-0 z-50 grid grid-cols-3 flex items-center px-8">
      <div class="flex items-center gap-6">
        <A href="/" class="text-2xl flex items-center gap-3">
          <img
            class="h-auto w-8"
            src="https://pixlie.com/images/logo.png"
            alt="Pixlie"
          />
          Pixlie
        </A>
      </div>

      <nav class="flex items-center justify-center">
        {/* todo: search bar here? */}
      </nav>

      <nav class="flex items-center justify-end gap-4">
        <ProjectsPopOver />
        <HelpPopOver />
        <SettingsPopOver />
        <GitHubStars />
      </nav>
    </div>
  );
};

export default Navbar;
