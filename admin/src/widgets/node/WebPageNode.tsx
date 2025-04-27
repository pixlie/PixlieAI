import { Component, createMemo, createSignal } from "solid-js";
import { useEngine } from "../../stores/engine.tsx";
import { useParams } from "@solidjs/router";

import ShareOptions from "../interactable/ShareOptions.tsx";
import LoaderIcon from "../../assets/icons/tabler-loader.svg";
import { WebMetadata } from "../../api_types/WebMetadata.ts";

interface WebPageNodeProps {
  nodeId: number;
}

interface WebPagePreviewContainerProps {
  metadata: WebMetadata;
}

interface WebPagePreviewProps extends WebPagePreviewContainerProps {
  showShareOptions: boolean;
}

const WebPagePreview: Component<WebPagePreviewProps> = ({
  metadata,
  showShareOptions,
}) => {
  const [imageVisible, setImageVisible] = createSignal(true);
  const [faviconVisible, setFaviconVisible] = createSignal(true);

  return (
    <div class="flex flex-col w-full bg-white border border-slate-200 rounded-xl shadow-sm group hover:shadow-lg transition-all duration-50 ease-in-out overflow-hidden">
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
          {metadata.site_name && (
            <div class="flex items-center gap-2">
              {metadata.favicon && faviconVisible() && (
                <img
                  src={metadata.favicon}
                  class="w-6 h-6 object-contain"
                  alt="logo"
                  onError={() => setFaviconVisible(false)}
                />
              )}

              <p class="text-xs text-slate-500 font-semibold rounded-full px-2 py-1 bg-slate-100 group-hover:bg-blue-200/50 group-hover:text-blue-600 transition-all duration-50 ease-in-out">
                {metadata.site_name}
              </p>
            </div>
          )}

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
                  // <p
                  //   class="text-xs text-slate-500 font-semibold rounded px-2 py-1 bg-slate-100 group-hover:bg-yellow-200/50 group-hover:text-yellow-600 transition-all duration-50 ease-in-out"
                  //   id={`tag-${i}-${tag}`}
                  // >
                  <p
                    class="text-xs text-slate-400 underline"
                    id={`tag-${i}-${tag}`}
                  >
                    {`#${tag}`}
                  </p>
                ))}
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

  const getWebMetadata = createMemo<WebMetadata | null>(() => {
    return getRelatedNodes(params.projectId, props.nodeId, "ParentOf", (n) =>
      n.labels.includes("WebMetadata")
    )?.[0]?.payload.data as WebMetadata | null;
  });

  return (
    <>
      {!!getWebMetadata() ? (
        <WebPagePreviewContainer metadata={getWebMetadata()!} />
      ) : (
        <div class="h-10 w-full flex items-center justify-center border border-slate-200 rounded-xl shadow-sm bg-white text-slate-500">
          <LoaderIcon />
        </div>
      )}
    </>
  );
};

export default WebPageNode;
