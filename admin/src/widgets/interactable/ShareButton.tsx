import { Component, createSignal } from "solid-js";
import ShareIcon from "../../assets/icons/tabler-share-2.svg";
import IconButton from "./IconButton";

interface IPropTypes {
  title?: string;
  url?: string;
}

const ShareButton: Component<IPropTypes> = (props) => {
  const [copied, setCopied] = createSignal<boolean>(false);

  const handleShare = async () => {
    const shareTitle =
      (props.title ? `${props.title} | ` : "") + "Powered by Pixlie";
    const shareUrl = props.url ?? "https://pixlie.com";

    if (navigator.share) {
      try {
        await navigator.share({ title: shareTitle, url: shareUrl });
      } catch (err) {
        console.error("Sharing failed:", err);
      }
    } else {
      try {
        await navigator.clipboard.writeText(shareUrl);
        setCopied(true);
      } catch (err) {
        console.error("Copy failed:", err);
        window.open(
          `mailto:?subject=${encodeURIComponent(shareTitle)}&body=${encodeURIComponent(shareUrl)}`,
          "_blank"
        );
      }
    }
  };

  return (
    <div class="-m-2">
      <IconButton
        name={copied() ? "Copied!" : "Share"}
        icon={
          <div class="flex h-6 w-6 items-center justify-center text-slate-400">
            <ShareIcon />
          </div>
        }
        onClick={handleShare}
      />
    </div>
  );
};

export default ShareButton;
