import { Component, createSignal } from "solid-js";

import { APIMatch } from "../../api_types/APIMatch.ts";
import SparkleIcon from "../../assets/icons/tabler-sparkles.svg";
import BrainIcon from "../../assets/icons/tabler-brain.svg";
import ShareOptions from "../interactable/ShareOptions.tsx";

interface MatchProps {
  match: APIMatch;
  nodeType?: string;
}

interface MatchPreviewProps extends MatchProps {
  showShareOptions?: boolean;
}

const WebPageMatchPreview: Component<MatchPreviewProps> = ({
  match,
  showShareOptions,
}) => {
  const { full_url, metadata, insight, reason } = match;

  const [imageVisible, setImageVisible] = createSignal(true);
  const [faviconVisible, setFaviconVisible] = createSignal(true);

  const cleanUrl = (url: string | null): string => {
    if (!url) return "";
    try {
      const { hostname } = new URL(url);
      return hostname.replace(/^www\./, "");
    } catch (error) {
      console.error("Invalid URL:", url);
      return url; // fallback
    }
  };

  return (
    <div class="flex flex-col w-full bg-white border-2 border-green-500 rounded-xl group hover:shadow-xl transition-all duration-50 ease-in-out overflow-hidden">
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
          href={metadata.url || full_url}
          target="_blank"
          rel="noreferrer"
          class="w-full flex flex-col gap-3"
        >
          <div class="flex items-center gap-2">
            {metadata.favicon && faviconVisible() && (
              <img
                src={metadata.favicon}
                class="w-6 h-6 object-contain"
                alt="logo"
                onError={() => setFaviconVisible(false)}
              />
            )}
            <div>
              <p class="text-xs text-slate-500 font-semibold rounded-full px-2 py-1 bg-slate-100 group-hover:bg-blue-200/50 group-hover:text-blue-600 transition-all duration-50 ease-in-out line-clamp-1">
                {metadata.site_name ||
                  cleanUrl(metadata.url) ||
                  cleanUrl(full_url)}
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
                <p class="text-md text-slate-800 leading-snug line-clamp-5">
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
                  <p
                    class="text-xs text-slate-500 font-semibold rounded-full px-2 py-1 bg-slate-100 group-hover:bg-yellow-200/50 group-hover:text-yellow-600 transition-all duration-50 ease-in-out"
                    id={`tag-${i}-${tag}`}
                  >
                    {`#${tag}`}
                  </p>
                ))}
            </div>
          )}

          {reason && (
            <div class="flex flex-col gap-1.5">
              <div class="flex items-center gap-2 text-lg font-medium text-fuchsia-600">
                <BrainIcon />
                <p>Reasoning</p>
              </div>
              <p class="text-md text-slate-800 leading-snug">{reason}</p>
            </div>
          )}

          {insight && (
            <div class="flex flex-col gap-1.5">
              <div class="flex items-center gap-2 text-lg font-medium text-purple-600">
                <SparkleIcon />
                <p>Insights</p>
              </div>
              <p class="text-md text-slate-800 leading-snug">{insight}</p>
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
              url={metadata.url || full_url}
              title={
                metadata.title || cleanUrl(metadata.url) || cleanUrl(full_url)
              }
            />
          </div>
        )}
      </div>
    </div>
  );
};

const URLMatchPreview: Component<MatchPreviewProps> = (props) => {
  const url = props.match.full_url;

  return (
    <div class="flex items-center h-10 gap-6 group transition-all duration-50 ease-in-out">
      <a
        href={url}
        target="_blank"
        rel="noreferrer"
        class="text-md flex-1 text-blue-500 group-hover:text-blue-600 underline font-semibold  "
      >
        {url}
      </a>
      <div
        class="flex justify-end -m-2
            w-0 group-hover:w-auto
                opacity-0 group-hover:opacity-100
                translate-x-8 group-hover:translate-x-0
                pointer-events-none group-hover:pointer-events-auto
                transition-all duration-100 delay-50 ease-in-out"
      >
        <ShareOptions url={url} title={`Match found from ${url}`} />
      </div>
      <span class="flex items-center gap-2 text-md font-semibold cursor-default text-green-600 bg-green-100 rounded-full px-4 py-2">
        {/* TODO: hover over info to show insights and reasoning */}
        Match
      </span>
    </div>
  );
};

const MatchResult: Component<MatchProps> = (props) => (
  <div class="group relative">
    {props.nodeType === "WebPage" && (
      <>
        <div class="absolute inset-0 z-10 flex items-center justify-center opacity-0 group-hover:opacity-100 pointer-events-none group-hover:pointer-events-auto cursor-pointer">
          <WebPageMatchPreview match={props.match} showShareOptions={true} />
        </div>
        <WebPageMatchPreview match={props.match} showShareOptions={false} />
      </>
    )}
    {props.nodeType === "URL" && <URLMatchPreview match={props.match} />}
  </div>
);

export default MatchResult;
