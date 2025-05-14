import { Component, createMemo, createSignal } from "solid-js";
import { useEngine } from "../../stores/engine.tsx";
import { useParams } from "@solidjs/router";

import ShareOptions from "../interactable/ShareOptions.tsx";
import { WebMetadata } from "../../api_types/WebMetadata.ts";
import SparkleIcon from "../../assets/icons/custom-gradient-sparkle.svg";
import { APINodeItem } from "../../api_types/APINodeItem.ts";
import { Link } from "../../api_types/Link.ts";

interface WebPageNodeProps {
  nodeId: number;
}

interface WebPagePreviewContainerProps {
  metadata: WebMetadata;
  insight?: string;
  reason?: string;
}

interface WebPagePreviewProps extends WebPagePreviewContainerProps {
  showShareOptions: boolean;
}

function cleanUrl(url: string | null): string {
  if (!url) return "";
  try {
    const { hostname } = new URL(url);
    return hostname.replace(/^www\./, "");
  } catch (error) {
    console.error("Invalid URL:", url);
    return url; // fallback
  }
}

const WebPagePreview: Component<WebPagePreviewProps> = ({
  metadata,
  insight,
  reason,
  showShareOptions,
}) => {
  const [imageVisible, setImageVisible] = createSignal(true);
  const [faviconVisible, setFaviconVisible] = createSignal(true);

  return (
    <div
      class={`flex flex-col w-full bg-white ${!!insight ? "border-2 border-green-600" : "border border-slate-200"} rounded-xl shadow-sm group hover:shadow-lg transition-all duration-50 ease-in-out overflow-hidden`}
    >
      {metadata.image && imageVisible() && (
        <img
          src={metadata.image}
          class="w-full object-cover"
          alt={"image"}
          onError={() => setImageVisible(false)}
        />
      )}

      <div class="w-full flex flex-col p-4 gap-3">
        <a
          href={metadata.url || ""}
          target="_blank"
          rel="noreferrer"
          class="w-full flex flex-col gap-3"
        >
          <div class="flex items-center gap-2">
            {metadata.favicon && faviconVisible() && (
              <img
                src={metadata.favicon}
                class="w-5 h-5 object-contain"
                alt="logo"
                onError={() => setFaviconVisible(false)}
              />
            )}
            <div class="flex shrink overflow-hidden rounded-full px-2 py-1 bg-slate-100 text-slate-500 group-hover:bg-blue-200/50 group-hover:text-blue-600 transition-all duration-50 ease-in-out">
              <p class="text-xs font-semibold truncate">
                {metadata.site_name
                  ? metadata.site_name
                  : cleanUrl(metadata.url)}
              </p>
            </div>
            {insight && (
              <span class="text-xs font-semibold text-green-600 bg-green-100 rounded-full px-2 py-1">
                Match
              </span>
            )}
          </div>

          {(metadata.title || metadata.description) && (
            <div class="flex flex-col gap-1.5">
              {metadata.title && (
                <p class="text-lg font-medium text-slate-800 leading-snug">
                  {metadata.title}
                </p>
              )}

              {metadata.description && (
                <p class="text-md text-slate-700 leading-snug line-clamp-5">
                  {metadata.description}
                </p>
              )}
            </div>
          )}

          {metadata.tags?.length && (
            <div class="w-full flex-wrap flex items-center gap-2 pb-1">
              {metadata.tags
                .flatMap((tag) => tag.split(","))
                ?.map((tag) => tag.trim())
                ?.filter((tag) => tag.length > 0)
                ?.map((tag, i) => (
                  <div class="flex shrink overflow-hidden rounded-full px-2 py-1 bg-slate-100 text-slate-500 group-hover:bg-yellow-200/50 group-hover:text-yellow-600 transition-all duration-50 ease-in-out">
                    <p
                      class="text-xs font-semibold truncate"
                      id={`tag-${i}-${tag}`}
                    >
                      {`#${tag}`}
                    </p>
                  </div>
                ))}
            </div>
          )}

          {reason && (
            <div class="flex flex-col gap-0.5 bg-slate-100 rounded-lg p-2 mb-1 text-slate-500 group-hover:text-violet-600 group-hover:bg-violet-200/50">
              <div class="flex items-center gap-1.5 text-xs font-semibold">
                <SparkleIcon />
                <p>REASONING</p>
              </div>
              <p class="text-md text-slate-700 leading-snug">{reason}</p>
            </div>
          )}

          {insight && (
            <div class="flex flex-col gap-0.5 bg-slate-100 rounded-lg p-2 mb-0.5 text-slate-500 group-hover:text-fuchsia-600 group-hover:bg-fuchsia-200/50">
              <div class="flex items-center gap-1.5 text-xs  font-semibold">
                <SparkleIcon />
                <p>INSIGHTS</p>
              </div>
              <p class="text-md text-slate-700 leading-snug">{insight}</p>
            </div>
          )}
        </a>

        {showShareOptions && (
          <div
            class="w-full flex justify-end -m-2
                opacity-0 group-hover:opacity-100
                translate-x-2 group-hover:translate-x-0
                pointer-events-none group-hover:pointer-events-auto
                transition-all duration-50 delay-50 ease-in-out"
          >
            <ShareOptions
              url={metadata.url || ""}
              title={metadata.title || ""}
            />
          </div>
        )}
      </div>
    </div>
  );
};

const WebPagePreviewContainer: Component<WebPagePreviewContainerProps> = (
  props
) => (
  <div class="group relative">
    <div class="absolute inset-0 z-10 flex items-center justify-center opacity-0 group-hover:opacity-100 pointer-events-none group-hover:pointer-events-auto cursor-pointer">
      <WebPagePreview {...props} showShareOptions={true} />
    </div>
    <WebPagePreview {...props} showShareOptions={false} />
  </div>
);

const WebPageNode: Component<WebPageNodeProps> = (props) => {
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

  const getInsight = createMemo<string | null>(() => {
    return getRelatedNodes(params.projectId, props.nodeId, "Matches", (n) =>
      n.labels.includes("Insight")
    )[0]?.payload.data as string | null;
  });

  const getReason = createMemo<string | null>(() => {
    return getRelatedNodes(params.projectId, props.nodeId, "Matches", (n) =>
      n.labels.includes("Reason")
    )[0]?.payload.data as string | null;
  });

  const getFullUrl = createMemo<string | null>(() => {
    const domain = getDomainNode()?.payload.data as string | null;
    const link = getLinkNode()?.payload.data as Link | null;
    if (!domain || !link) return null;
    return `https://${domain.replace(/^(?:https?:\/\/)?(?:www\.)?/, "")}${link.path}${link.query ? "?" + link.query : ""}`;
  });

  const getHostName = createMemo<string | null>(() => {
    const domain = getDomainNode()?.payload.data as string | null;
    if (!domain) return null;
    return domain.replace(/^(?:https?:\/\/)?(?:www\.)?/, "");
  });

  const getWebMetadata = createMemo<WebMetadata | null>(() => {
    const metadata = getRelatedNodes(
      params.projectId,
      props.nodeId,
      "ParentOf",
      (n) => n.labels.includes("WebMetadata")
    )?.[0]?.payload.data as WebMetadata | null;
    if (!metadata) return null;
    if (!metadata.url) {
      metadata.url = getFullUrl();
    }
    if (!metadata.site_name) {
      metadata.site_name = getHostName();
    }
    return metadata;
  });

  return (
    <>
      {!!getWebMetadata() && (
        <WebPagePreviewContainer
          metadata={getWebMetadata()!}
          insight={!!getInsight()! ? getInsight()! : undefined}
          reason={!!getReason()! ? getReason()! : undefined}
        />
      )}
    </>
  );
};

export default WebPageNode;
