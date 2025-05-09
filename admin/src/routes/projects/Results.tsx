import { Component, createMemo, Show } from "solid-js";
import { useEngine } from "../../stores/engine";
import { useParams, useSearchParams } from "@solidjs/router";
import Heading from "../../widgets/typography/Heading.tsx";
import { NodeLabel } from "../../api_types/NodeLabel.ts";
import ResultsCount from "../../widgets/generic/ResultsCount.tsx";
import { Link } from "../../api_types/Link.ts";
import { WebMetadata } from "../../api_types/WebMetadata.ts";
import MatchResult from "../../widgets/node/MatchResult.tsx";
import { APIMatch } from "../../api_types/APIMatch.ts";

const labelTypes: string[] = ["WebPage", "URL"];
type LabelType = (typeof labelTypes)[number];

const Results: Component = () => {
  const [_, { getNodes, getRelatedNodes }] = useEngine();
  const [searchParams] = useSearchParams();
  const params = useParams();

  const getNodeTypeFromSearchParam = createMemo(() => {
    if (!!searchParams.label) {
      return searchParams.label as LabelType;
    }
    return undefined;
  });

  const getWebPages = createMemo(() => {
    return getNodes(params.projectId, (node) =>
      node.labels.includes("WebPage" as NodeLabel)
    );
  });

  const getMatches = createMemo(() =>
    getWebPages()
      .map((node) => {
        const linkNode = getRelatedNodes(
          params.projectId,
          node.id,
          "ParentOf",
          (n) => n.labels.includes("Link")
        )?.[0];

        const domainNode = linkNode
          ? getRelatedNodes(params.projectId, linkNode.id, "BelongsTo", (n) =>
              n.labels.includes("Domain")
            )?.[0]
          : null;

        const fullUrl =
          linkNode && domainNode
            ? `https://${(domainNode.payload.data as string).replace(
                /^(?:https?:\/\/)?(?:www\.)?/,
                ""
              )}${(linkNode.payload.data as Link).path}${
                (linkNode.payload.data as Link).query
                  ? "?" + (linkNode.payload.data as Link).query
                  : ""
              }`
            : null;

        const metadata = getRelatedNodes(
          params.projectId,
          node.id,
          "ParentOf",
          (n) => n.labels.includes("WebMetadata")
        )?.[0]?.payload.data as WebMetadata | null;

        const insight = getRelatedNodes(
          params.projectId,
          node.id,
          "Matches",
          (n) => n.labels.includes("Insight")
        )?.[0]?.payload.data as string | null;

        const reason = getRelatedNodes(
          params.projectId,
          node.id,
          "Matches",
          (n) => n.labels.includes("Reason")
        )?.[0]?.payload.data as string | null;

        if (!fullUrl || !metadata || !insight || !reason) {
          return null;
        }

        return {
          node_id: node.id,
          full_url: fullUrl,
          metadata,
          insight,
          reason,
        };
      })
      .filter(Boolean)
  );

  return (
    <div class="relative flex-1">
      <div class="absolute w-full h-full flex flex-col gap-8">
      {searchParams.label === "WebPage" && (
        <Heading size={3}>Web Pages</Heading>
      )}
      {searchParams.label === "URL" && (
        <Heading size={3}>URLs</Heading>
      )}

      <ResultsCount count={getMatches()?.length} />

      {getMatches()?.length ? (
        <>
          {searchParams.label === "WebPage" && (
            <div class="columns-1 lg:columns-3 space-y-8 gap-8">
              {getMatches().map((match) => (
                <div class="break-inside-avoid overflow-visible will-change-transform">
                  <MatchResult
                    match={match as APIMatch}
                    nodeType={getNodeTypeFromSearchParam()}
                  />
                </div>
              ))}
            </div>
          )}
          {searchParams.label === "URL" && (
            <div class="flex-1 flex flex-col gap-5">
              {getMatches().map((match, i) => (
                <>
                  <Show when={i > 0}>
                    <div class="border border-slate-200" />
                  </Show>
                  <MatchResult
                    match={match as APIMatch}
                    nodeType={getNodeTypeFromSearchParam()}
                  />
                  <Show when={i === getMatches()!.length - 1}>
                    <div />
                  </Show>
                </>
              ))}
            </div>
          )}
        </>
      ) : (
        <div class="text-slate-500 h-full w-full flex justify-center items-center text-center">
          No matches found yet!
        </div>
      )}
    </div>
    </div>
  );
};

export default Results;
