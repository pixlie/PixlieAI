import { Component, createMemo, For } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses";
import { ThemableItem } from "../../utils/uIClasses/types";
import { JSX } from "solid-js/jsx-runtime";

interface IHighlightTerms {
  content: string;
  terms: string[];
  color?: ThemableItem;
  maxWords?: number;
}

export const HighlightTerms: Component<IHighlightTerms> = (props) => {
  const [_, { getColors }] = useUIClasses();

  const getSortedTerms = createMemo<Array<string>>(() => {
    return props.terms
      .sort()
      .sort((a, b) => b.split(/[\s\n]+/).length - a.split(/[\s\n]+/).length);
  });

  const getFinalContent = createMemo<Array<JSX.Element>>(() => {
    if (props.maxWords === undefined) {
      props.maxWords = 50;
    }
    const firstMentionAt = props.content.search(
      new RegExp(getSortedTerms().join("|"), "gim"),
    );
    const wordsBefore = props.content.slice(0, firstMentionAt).split(/[\s\n]+/);
    const words = props.content.split(/[\s\n]+/);
    let startAt = 0;
    if (wordsBefore.length > props.maxWords) {
      startAt = Math.round(wordsBefore.length * 0.3);
    }
    let finalContent: Array<JSX.Element> = [
      words.slice(startAt, startAt + props.maxWords).join(" "),
    ];
    for (const term of getSortedTerms()) {
      let termContent = [];
      const regex = new RegExp(term, "gim");
      for (const content of finalContent) {
        if (typeof content === "string") {
          termContent.push(
            ...content.split(regex).flatMap((part, idx) => {
              return (
                <>
                  {idx > 0 ? (
                    <span class={getColors()[props.color || "highlight"]}>
                      {term}
                    </span>
                  ) : (
                    ""
                  )}
                  {part}
                </>
              );
            }),
          );
        } else {
          termContent.push(content);
        }
      }
      finalContent = termContent;
    }
    if (startAt > 0) {
      finalContent.slice().unshift("…" as JSX.Element);
    }
    if (startAt + props.maxWords < words.length) {
      finalContent.push("…" as JSX.Element);
    }
    return finalContent;
  });

  return (
    <For each={getFinalContent()}>
      {(partial) => {
        return <>{partial}</>;
      }}
    </For>
  );
};

export default HighlightTerms;
