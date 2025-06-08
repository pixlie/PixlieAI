import { Component, Show, createMemo, createSignal } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses";
import { useWorkspace } from "../../stores/workspace";
import { useParams } from "@solidjs/router";
import CollapseSidebarIcon from "../../assets/icons/tabler-layout-sidebar-inactive.svg";
import ExpandSidebarIcon from "../../assets/icons/tabler-layout-sidebar.svg";
import IconButton from "../interactable/IconButton.tsx";
import { CrawlerSettings } from "../../api_types/CrawlerSettings.ts";
import { useEngine } from "../../stores/engine.tsx";
import { ClassifierSettings } from "../../api_types/ClassifierSettings.ts";
import { NodeLabel } from "../../api_types/NodeLabel.ts";
import { ToolEnabled } from "../../api_types/ToolEnabled.ts";

const ToolToggle = ({
  name,
  info = "",
  isEnabled = false,
}: {
  name: string;
  info?: string;
  isEnabled?: boolean;
}) => {
  const bgColor = isEnabled ? "bg-green-600" : "bg-slate-200";
  
  return (
    <div class="flex flex-col gap-2 w-full">
      <div class="flex items-center justify-between gap-2">
        <p>{name}</p>
        <div
          class={
            "relative inline-flex h-6 w-10 items-center rounded-full cursor-pointer " +
            bgColor
          }
        >
          <span
            class="absolute left-1 inline-block h-4 w-4 transform rounded-full bg-white shadow-md transition-transform"
            classList={{
              "translate-x-0": !isEnabled,
              "translate-x-4": isEnabled,
            }}
          />
        </div>
      </div>
      {info && <p class="text-sm text-slate-500">{info}</p>}
    </div>
  );
};

const Tools: Component = () => {
  const [_, { getColors }] = useUIClasses();
  const [_e, { getNodes }] = useEngine();
  const [workspace] = useWorkspace();
  const params = useParams();
  const [collapsed, setCollapsed] = createSignal(false);

  const getCrawlerSettings = createMemo<CrawlerSettings | undefined>(() => {
    if (!params.projectId) return undefined;
    const nodes = getNodes(
      params.projectId,
      (node) => node.payload.type === "CrawlerSettings"
    );
    return nodes[0]?.payload.data as CrawlerSettings | undefined;
  });

  const getClassifierSettings = createMemo<ClassifierSettings | undefined>(
    () => {
      if (!params.projectId) return undefined;
      const nodes = getNodes(
        params.projectId,
        (node) => node.payload.type === "ClassifierSettings"
      );
      return nodes[0]?.payload.data as ClassifierSettings | undefined;
    }
  );

  const getStartingLinkIds = createMemo<number[]>(() => {
    if (!params.projectId) return [];
    const nodes = getNodes(
      params.projectId,
      (node) =>
        node.labels.includes("Link" as NodeLabel) &&
        (node.labels.includes("AddedByUser" as NodeLabel) ||
          node.labels.includes("AddedByAI" as NodeLabel) ||
          node.labels.includes("AddedByWebSearch" as NodeLabel))
    );
    return nodes
      .sort((a, b) => a.id - b.id)
      .slice(0, 100)
      .map((x) => x.id);
  });

  const tools = createMemo(() => {
    const crawlerSettings = getCrawlerSettings();
    const classifierSettings = getClassifierSettings();
    const startingLinkIds = getStartingLinkIds();
    
    
    return [
      {
        name: "Search",
        info: "Find URLs",
        isEnabled:
          !!crawlerSettings &&
          !!crawlerSettings.keywords_to_get_accurate_results_from_web_search &&
          startingLinkIds.length === 0,
      },
      {
        name: "Crawl",
        info: "Crawl URLs",
        isEnabled:
          !!crawlerSettings && crawlerSettings.is_enabled === ("Yes" as ToolEnabled),
      },
      {
        name: "Scrape",
        info: "Scrape content from URLs",
        isEnabled:
          !!classifierSettings &&
          classifierSettings.is_enabled === ("Yes" as ToolEnabled),
      },
      {
        name: "Classify",
        info: "Classify relevant content",
        isEnabled:
          !!classifierSettings &&
          classifierSettings.is_enabled === "Yes",
      },
      {
        name: "Extract - New!",
        info: "Extract entities from content",
        isEnabled:
          !!classifierSettings &&
          classifierSettings.is_enabled === "Yes",
      },
      {
        name: "Monitor - Coming Soon",
        info: "Monitor changes over time",
        isEnabled: false, // Not implemented yet
      },
      {
        name: "Notify - Coming Soon",
        info: "Get notified about changes",
        isEnabled: false, // Not implemented yet
      },
    ];
  });

  return (
    <Show
      when={
        workspace.isReady &&
        workspace.settingsStatus?.type === "Complete" &&
        workspace.projects &&
        !!params.projectId
      }
    >
      <div
        class={
          `transition-all duration-300 flex flex-col border max-h-full rounded-lg py-5 ` +
          (collapsed() ? "w-20" : "w-80") +
          " " +
          getColors()["sideBar"]
        }
      >
        <div class="mr-5 pb-8 flex items-center justify-between">
          {!collapsed() && <h1 class="px-4 font-medium ">Tools</h1>}
          {/* TODO: show projects dropdown for per project routes when sidebar expanded */}
          <IconButton
            name={collapsed() ? "Expand" : "Collapse"}
            icon={collapsed() ? <ExpandSidebarIcon /> : <CollapseSidebarIcon />}
            onClick={() => setCollapsed(!collapsed())}
          />
        </div>

        <div class="overflow-y-auto">
          <Show when={!collapsed()}>
            <div class="flex flex-col gap-4 p-4">
              {tools().map((tool, i) => (
                <>
                  <ToolToggle
                    name={tool.name}
                    info={tool.info}
                    isEnabled={tool.isEnabled}
                  />
                  {i < tools().length - 1 && <hr class="border-slate-300" />}
                </>
              ))}
            </div>
          </Show>
        </div>
      </div>
    </Show>
  );
};

export default Tools;
