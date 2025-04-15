import { Component, createMemo } from "solid-js";
import { useEngine } from "../../stores/engine.tsx";
import { useParams } from "@solidjs/router";

import { Link } from "../../api_types/Link.ts";
import { APINodeItem } from "../../api_types/APINodeItem.ts";
import ShareOptions from "../interactable/ShareOptions.tsx";
import LoaderIcon from "../../assets/icons/tabler-loader.svg";

interface WebPageNodeProps {
  nodeId: number;
}

interface WebPagePreviewContainerProps {
  url?: string;
  name?: string;
  logo?: string;
  image?: string;
  title?: string;
  description?: string;
  created_at?: string;
  modified_at?: string;
  tags?: string[];
}

interface WebPagePreviewProps extends WebPagePreviewContainerProps {
  showShareOptions?: boolean;
}

const WebPagePreview: Component<WebPagePreviewProps> = (props) => (
  <div class="flex flex-col w-full bg-white border border-slate-200 rounded-xl shadow-sm group hover:shadow-lg transition-all duration-50 ease-in-out overflow-hidden">
    {props.image && <img src={props.image} class="w-full object-cover" />}

    <div class="w-full flex flex-col p-4 gap-3">
      <a
        href={props.url}
        target="_blank"
        rel="noreferrer"
        class="w-full flex flex-col gap-3"
      >
        {(props.logo || props.name) && (
          <div class="flex items-center gap-2">
            {props.logo && (
              <img src={props.logo} class="w-6 h-6 object-contain" />
            )}
            {props.name && (
              <p class="text-xs text-slate-500 font-semibold rounded-full px-2 py-1 bg-slate-100 group-hover:bg-blue-200/50 group-hover:text-blue-600 transition-all duration-50 ease-in-out">
                {props.name}
              </p>
            )}
          </div>
        )}

        {(props.title || props.description) && (
          <div class="flex flex-col gap-1.5">
            {props.title && (
              <p class="text-lg font-medium text-slate-800 leading-snug">
                {props.title}
              </p>
            )}

            {props.description && (
              <p class="text-md text-slate-700 leading-snug line-clamp-5">
                {props.description}
              </p>
            )}
          </div>
        )}

        {(props.modified_at || props.created_at) && (
          <p class="text-xs text-slate-400 leading-none">
            {`Modified: ${props.modified_at}` ||
              `Published: ${props.created_at}`}
          </p>
        )}

        {!!props.tags?.length && (
          <div class="w-full flex-wrap flex items-center gap-2 pb-1">
            {props.tags.map((tag, i) => (
              <p
                class="text-xs text-slate-500 font-semibold rounded px-2 py-1 bg-slate-100 group-hover:bg-yellow-200/50 group-hover:text-yellow-600 transition-all duration-50 ease-in-out"
                id={`tag-${i}-${tag}`}
              >
                {tag}
              </p>
            ))}
          </div>
        )}
      </a>

      {props.showShareOptions && (
        <div
          class="w-full flex justify-end -m-2
                opacity-0 group-hover:opacity-100
                translate-x-2 group-hover:translate-x-0
                pointer-events-none group-hover:pointer-events-auto
                transition-all duration-50 delay-50 ease-in-out"
        >
          <ShareOptions url={props.url} title={props.title || ""} />
        </div>
      )}
    </div>
  </div>
);

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

  const getContentNodes = createMemo<APINodeItem[] | null>(() => {
    return getRelatedNodes(
      params.projectId,
      props.nodeId,
      "ParentOf",
      (n) => n.payload.type === "Text"
    );
  });

  const getUrl = createMemo<string | null>(() => {
    return getRelatedNodes(params.projectId, props.nodeId, "ParentOf", (n) =>
      n.labels.includes("Url")
    )[0]?.payload.data as string | null;
  });

  const getFullUrl = createMemo<string | null>(() => {
    const domain = getDomainNode()?.payload.data as string | null;
    const link = getLinkNode()?.payload.data as Link | null;
    if (!domain || !link) return null;
    return `https://${domain.replace(/^(?:https?:\/\/)?(?:www\.)?/, "")}${link.path}${link.query ? "?" + link.query : ""}`;
  });

  const getName = createMemo<string | null>(() => {
    return getRelatedNodes(params.projectId, props.nodeId, "ParentOf", (n) =>
      n.labels.includes("Name")
    )[0]?.payload.data as string | null;
  });

  const getHostName = createMemo<string | null>(() => {
    const domain = getDomainNode()?.payload.data as string | null;
    if (!domain) return null;
    return domain.replace(/^(?:https?:\/\/)?(?:www\.)?/, "");
  });

  const getImage = createMemo<string | null>(() => {
    return getRelatedNodes(params.projectId, props.nodeId, "ParentOf", (n) =>
      n.labels.includes("Image")
    )[0]?.payload.data as string | null;
  });

  const getLogo = createMemo<string | null>(() => {
    return getRelatedNodes(params.projectId, props.nodeId, "ParentOf", (n) =>
      n.labels.includes("Logo")
    )[0]?.payload.data as string | null;
  });

  const getTitle = createMemo<string | null>(() => {
    return getRelatedNodes(params.projectId, props.nodeId, "ParentOf", (n) =>
      n.labels.includes("Title")
    )[0]?.payload.data as string | null;
  });

  const getDescription = createMemo<string | null>(() => {
    return getRelatedNodes(params.projectId, props.nodeId, "ParentOf", (n) =>
      n.labels.includes("Description")
    )[0]?.payload.data as string | null;
  });

  const getCreatedAt = createMemo<string | null>(() => {
    return getRelatedNodes(params.projectId, props.nodeId, "ParentOf", (n) =>
      n.labels.includes("CreatedAt")
    )[0]?.payload.data as string | null;
  });

  const getModifiedAt = createMemo<string | null>(() => {
    return getRelatedNodes(params.projectId, props.nodeId, "ParentOf", (n) =>
      n.labels.includes("ModifiedAt")
    )[0]?.payload.data as string | null;
  });

  const getTags = createMemo<string[] | null>(() => {
    return getRelatedNodes(params.projectId, props.nodeId, "ParentOf", (n) =>
      n.labels.includes("Tag")
    )
      ?.flatMap((node) => (node.payload.data as string).split(","))
      ?.map((tag) => tag.trim())
      ?.filter((tag) => tag.length > 0);
  });

  return (
    <>
      {!!getLinkNode() && !!getContentNodes() ? (
        <WebPagePreviewContainer
          url={getUrl()! || getFullUrl()!}
          name={getName()! || getHostName()!}
          logo={getLogo()!}
          image={getImage()!}
          title={getTitle()!}
          description={getDescription()!}
          created_at={getCreatedAt()!}
          modified_at={getModifiedAt()!}
          tags={getTags()!}
        />
      ) : (
        <div class="h-10 w-full flex items-center justify-center border border-slate-200 rounded-xl shadow-sm bg-white text-slate-500">
          <LoaderIcon />
        </div>
      )}
    </>
  );
};

export default WebPageNode;
