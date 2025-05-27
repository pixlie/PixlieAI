import { Component, createMemo, createSignal } from "solid-js";
import { useEngine } from "../../stores/engine.tsx";
import { useParams } from "@solidjs/router";

import { APINodeItem } from "../../api_types/APINodeItem.ts";
import { Link } from "../../api_types/Link.ts";
import SparkleIcon from "../../assets/icons/custom-gradient-sparkle.svg";
import InfoIcon from "../../assets/icons/tabler-info-circle.svg";
import { Classification } from "../../api_types/Classification.ts";

interface URLNodeProps {
  nodeId: number;
  showDivider?: boolean;
}

interface URLPreviewProps {
  url: string;
  classification: Classification;
}

const URLPreview: Component<URLPreviewProps> = ({ url, classification }) => {
  const [viewed, setViewed] = createSignal<boolean>(false);

  return (
    <div class="flex items-center py-1">
      <a
        href={url}
        target="_blank"
        rel="noopener noreferrer"
        onClick={() => setViewed(true)}
        class={
          "text-left text-md flex-1 hover:underline truncate " +
          (viewed() ? "text-purple-700" : "text-blue-700")
        }
      >
        {url}
      </a>
      {!!classification?.is_relevant && (
        <span class="text-xs font-semibold text-green-600 bg-green-100 rounded-full px-2 py-1">
          Match
        </span>
      )}
      <div class="rounded-full group p-2 hover:bg-slate-200 hover:text-slate-800 hover:cursor-pointer relative flex items-center gap-1">
        <div
          class="pointer-events-none opacity-0  scale-95 transition-all duration-100 ease-in-out 
            group-hover:opacity-100  group-hover:scale-100 group-hover:pointer-events-auto 
            flex flex-col absolute right-full top-full -translate-y-1/2 -mt-6 w-[30vw]"
        >
          <div
            class="relative bg-white border border-slate-200 mt-2 mr-3.5 p-4 rounded-xl shadow-lg flex flex-col gap-4
            before:content-[''] before:absolute before:top-1/2 before:-translate-y-1/2 before:-right-2 before:w-4 before:h-4
            before:bg-white before:border-r before:border-b before:border-slate-200
            before:-rotate-45 before:shadow-lg before:shadow-slate-200"
          >
            {!!classification?.reason && (
              <>
                <div class="flex flex-col gap-1">
                  <p class="text-xs text-slate-800 font-semibold">REASONING</p>
                  <p class="text-xs text-slate-700 leading-snug">
                    {classification.reason}
                  </p>
                </div>
              </>
            )}
            {!!classification?.insight_if_classified_as_relevant && (
              <>
                <div class="flex flex-col gap-1">
                  <p class="text-xs text-slate-800 font-semibold">INSIGHTS</p>
                  <p class="text-xs text-slate-700 leading-snug">
                    {classification.insight_if_classified_as_relevant}
                  </p>
                </div>
              </>
            )}
          </div>
        </div>
        <SparkleIcon />
        <InfoIcon />
      </div>
    </div>
  );
};

const URLNode: Component<URLNodeProps> = (props) => {
  const [_, { getRelatedNodes }] = useEngine();
  const params = useParams();

  const getLinkNode = createMemo<APINodeItem | null>(() => {
    return getRelatedNodes(
      params.projectId,
      props.nodeId,
      "ContentOf",
      (n) => n.payload.type === "Link"
    )[0];
  });

  const getDomainNode = createMemo<APINodeItem | null>(() => {
    const linkNode = getLinkNode();
    if (!linkNode?.id) return null;
    const domainNode = getRelatedNodes(
      params.projectId,
      linkNode.id,
      "BelongsTo",
      (n) => n.labels.includes("Domain")
    )[0];
    return domainNode;
  });

  const getFullUrl = createMemo<string | null>(() => {
    const domain = getDomainNode()?.payload.data as string | null;
    const link = getLinkNode()?.payload.data as Link | null;
    if (!domain || !link) return null;
    return `https://${domain.replace(/^(?:https?:\/\/)?(?:www\.)?/, "")}${link.path}${link.query ? "?" + link.query : ""}`;
  });

  const getClassification = createMemo<Classification | null>(() => {
    return getRelatedNodes(params.projectId, props.nodeId, "Classifies", (n) =>
      n.labels.includes("Classification")
    )[0]?.payload.data as Classification | null;
  });

  return (
    <>
      {!!getFullUrl() && !!getClassification() && (
        <>
          <URLPreview
            url={getFullUrl()!}
            classification={getClassification()!}
          />
          {props.showDivider && <div class="border-b border-slate-200" />}
        </>
      )}
    </>
  );
};

export default URLNode;
