import {Component, onMount} from "solid-js";
import { useWorkspace } from "../../stores/workspace";
import ProjectListItem from "./ProjectListItem";
import Heading from "../typography/Heading";

const ProjectList: Component = () => {
  const [workspace, {fetchProjects}] = useWorkspace();

  onMount(() => {
      fetchProjects().then(_ => {});
  })

  return (
    <>
      <Heading size={1}>Projects</Heading>
      <div class="sm:grid sm:grid-cols-2 sm:gap-4 sm:divide-y-0">
        {workspace.isReady && !!workspace.projects ? (
          <>
            {workspace.projects.map((project) => (
              <ProjectListItem {...project} />
            ))}
          </>
        ) : null}
      </div>
    </>
  );
};

export default ProjectList;
