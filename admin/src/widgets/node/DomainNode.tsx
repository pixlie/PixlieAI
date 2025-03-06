import { Component, createMemo } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses.tsx";
import { Domain } from "../../api_types/Domain.ts";
import { useEngine } from "../../stores/engine.tsx";
import { useParams } from "@solidjs/router";
import { IEngine } from "../../utils/types.tsx";

interface IDomainNodeProps {
  nodeId: number;
}

const DomainNode: Component<IDomainNodeProps> = (props) => {
  const [engine, { toggleCrawl }] = useEngine();
  const params = useParams();
  const [_, { getColors }] = useUIClasses();

  const getProject = createMemo<IEngine | undefined>(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  const getPayload = createMemo<Domain | undefined>(() => {
    if (getProject() && props.nodeId in getProject()!.nodes) {
      return getProject()!.nodes[props.nodeId].payload.data as Domain;
    }
    return undefined;
  });

  const isFetching = createMemo<boolean>(() => {
    return !!(getProject() && getProject()!.nodes[props.nodeId]?.isFetching);
  });

  return (
    <>
      {!!getPayload() ? (
        <div class="flex items-center gap-5">
          {isFetching() ? (
            // spinning loader
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="w-6 h-6 animate-spin"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
            >
              <line x1="12" x2="12" y1="2" y2="6" />
              <line x1="12" x2="12" y1="18" y2="22" />
              <line x1="4.93" x2="7.76" y1="4.93" y2="7.76" />
              <line x1="16.24" x2="19.07" y1="16.24" y2="19.07" />
              <line x1="2" x2="6" y1="12" y2="12" />
              <line x1="18" x2="22" y1="12" y2="12" />
              <line x1="4.93" x2="7.76" y1="19.07" y2="16.24" />
              <line x1="16.24" x2="19.07" y1="7.76" y2="4.93" />
            </svg>
          ) : (
            <button onClick={() => toggleCrawl(params.projectId, props.nodeId)}>
              {getPayload()!.is_allowed_to_crawl ? (
                // checked circle
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  class="w-6 h-6"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                >
                  <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
                  <polyline points="22 4 12 14.01 9 11.01" />
                </svg>
              ) : (
                // unchecked circle
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  class="w-6 h-6"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                >
                  <circle cx="12" cy="12" r="10" />
                </svg>
              )}
            </button>
          )}
          <a
            class={getColors().link}
            href={`https://${getPayload()!.name}`}
            target="_blank"
            rel="noreferrer"
          >
            {getPayload()!.name}
          </a>
        </div>
      ) : null}
    </>
  );
};

export default DomainNode;
