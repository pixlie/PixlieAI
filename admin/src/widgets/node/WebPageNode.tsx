import { Component, createMemo } from "solid-js";
import { useEngine } from "../../stores/engine.tsx";
import { useParams } from "@solidjs/router";

import { Link } from "../../api_types/Link.ts";
import { APINodeItem } from "../../api_types/APINodeItem.ts";
import ShareOptions from "../interactable/ShareOptions.tsx";

interface WebPageNodeProps {
  nodeId: number;
}

interface WebPagePreviewContainerProps {
  url: string;
  data: Link;
}

interface WebPagePreviewProps extends WebPagePreviewContainerProps {
  showShareOptions?: boolean;
}

const WebPagePreview: Component<WebPagePreviewProps> = (props) => (
  <div class="w-full bg-white border rounded-xl shadow-sm">
    {props.data.thumbnail && (
      <img
        src={props.data.thumbnail}
        alt="thumbnail"
        class="w-full object-cover rounded-t-xl"
      />
    )}

    <div class="flex flex-col p-4 gap-3">
      <div class="flex items-center gap-2">
        {props.data.favicon && (
          <img
            src={props.data.favicon}
            alt="favicon"
            class="w-6 h-6 object-contain"
          />
        )}

        <a
          href={props.url}
          title={props.data.title || props.url}
          target="_blank"
          rel="noopener noreferrer"
          class="h-6 flex items-center px-2 font-medium text-sm text-slate-500 hover:text-blue-600 bg-slate-100 hover:bg-slate-200 rounded-full"
        >
          {props.url
            .replace(/^(?:https?:\/\/)?(?:www\.)?/, "")
            .replace(/\/.*$/, "")}
        </a>
      </div>

      {props.data.title && (
        <p class="text-md font-medium text-slate-800 leading-tight">
          {props.data.title}
        </p>
      )}

      {props.data.description && (
        <p class="text-md text-slate-700 leading-tight">
          {props.data.description}
        </p>
      )}

      {props.data.published_date && (
        <p class="text-xs text-slate-400 leading-tight">
          {props.data.published_date}
        </p>
      )}

      {props.showShareOptions && (
        <div
          class="flex justify-end -m-2
                opacity-0 group-hover:opacity-100
                translate-x-2 group-hover:translate-x-0
                pointer-events-none group-hover:pointer-events-auto
                transition-all duration-50 delay-50 ease-in-out"
        >
          <ShareOptions url={props.url} title={props.data.title || ""} />
        </div>
      )}
    </div>
  </div>
);

const WebPagePreviewContainer: Component<WebPagePreviewContainerProps> = (
  props
) => (
  <div class="group relative">
    <div class="absolute inset-0 z-10 flex items-center justify-center opacity-0 group-hover:opacity-100 pointer-events-none group-hover:pointer-events-auto">
      <WebPagePreview
        url={props.url}
        data={props.data}
        showShareOptions={true}
      />
    </div>
    <WebPagePreview
      url={props.url}
      data={props.data}
      showShareOptions={false}
    />
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
    const nodeId = getLinkNode()?.id;
    if (!nodeId) return null;
    return getRelatedNodes(params.projectId, nodeId, "BelongsTo", (n) =>
      n.labels.includes("Domain")
    )[0];
  });

  const getURL = createMemo<string | null>(() => {
    const domainData = getDomainNode()?.payload.data as string | null;
    const linkData = getLinkNode()?.payload.data as Link | null;
    if (!domainData || !linkData) return null;
    return `https://${domainData.replace(/^(?:https?:\/\/)?(?:www\.)?/, "")}${linkData.path}${linkData.query ? "?" + linkData.query : ""}`;
  });

  const getData = createMemo<Link | null>(() => {
    return getLinkNode()?.payload.data as Link | null;
  });

  return (
    <>
      {!!getURL() && !!getData() && (
        <WebPagePreviewContainer url={getURL()!} data={getData()!} />
      )}
    </>
  );
};

export default WebPageNode;
