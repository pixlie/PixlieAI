import { Component, createMemo } from "solid-js";
import { useEngine } from "../../stores/engine.tsx";
import { useParams } from "@solidjs/router";

import { Link } from "../../api_types/Link.ts";
import { APINodeItem } from "../../api_types/APINodeItem.ts";
import ShareOptions from "../interactable/ShareOptions.tsx";

interface IPropTypes {
  nodeId: number;
}

const WebPagePreview: Component<IPropTypes> = (props) => {
  const [_, { getRelatedNodes, getNodeById }] = useEngine();
  const params = useParams();

  const getLinkNodeId = createMemo<number | undefined>(() => {
    return getRelatedNodes(params.projectId, props.nodeId, "ContentOf")?.[0]
      ?.id;
  });

  const getNode = createMemo(() => {
    const nodeByNodeId = getNodeById(params.projectId, getLinkNodeId()!);
    if (!!nodeByNodeId && nodeByNodeId.payload.type === "Link") {
      return nodeByNodeId as APINodeItem;
    }
    return undefined;
  });

  const getDomain = createMemo<string | undefined>(
    () =>
      getRelatedNodes(params.projectId, getLinkNodeId()!, "BelongsTo", (n) =>
        n.labels.includes("Domain")
      )[0]?.payload.data as string | undefined
  );

  const getPayload = createMemo<Link | undefined>(() => {
    if (!getNode() || getNode()?.payload?.type !== "Link") return undefined;
    return getNode()!.payload.data as Link;
  });

  const getURL = createMemo<string | undefined>(() => {
    const domain = getDomain();
    const payload = getPayload();
    if (!domain || !payload) return;
    return `https://${domain}${payload.path}${payload.query ? "?" + payload.query : ""}`;
  });

  return (
    <>
      {!!getNode() && (
        <div class="group relative rounded-xl bg-white border shadow-md transition-transform duration-300 hover:scale-[1.02] hover:z-10">
          {getPayload()?.thumbnail && (
            <img
              src={getPayload()!.thumbnail || ""}
              alt="thumbnail"
              class="w-full object-cover rounded-t-xl"
            />
          )}
          <div class="flex flex-col p-4 pb-2 gap-2 relative">
            <div class="flex items-center gap-2">
              <div class="flex items-center gap-2 flex-1">
                {getPayload()?.favicon && (
                  <img
                    src={getPayload()!.favicon || ""}
                    alt="favicon"
                    class="w-5 h-5 object-contain"
                  />
                )}
                {getURL() && (
                  <a
                    href={getURL()}
                    class={`text-blue-600 underline text-sm truncate`}
                    target="_blank"
                    rel="noopener noreferrer"
                    title={getURL()}
                  >
                    {getDomain()?.replace(/^(?:https?:\/\/)?(?:www\.)?/, "")}
                  </a>
                )}
              </div>
            </div>
            {getPayload()?.title && (
              <p class="text-md font-medium text-slate-800">
                {getPayload()!.title}
              </p>
            )}
            {getPayload()?.description && (
              <p class="text-sm text-slate-700">{getPayload()!.description}</p>
            )}
            {getPayload()?.published_date && (
              <p class="text-xs text-slate-500">
                {getPayload()!.published_date}
              </p>
            )}
          </div>
          <div class="relative mx-4 mt-2 mb-0 group-hover:mb-4 h-0 group-hover:h-6 opacity-0 group-hover:opacity-100">
            <div class="absolute bottom-0 right-0 translate-y-2 group-hover:translate-y-0 transition-all duration-300 z-10 pointer-events-none group-hover:pointer-events-auto text-slate-500">
              <ShareOptions title={getPayload()!.title || ""} url={getURL()} />
            </div>
          </div>
        </div>
      )}
    </>
  );
};

export default WebPagePreview;
