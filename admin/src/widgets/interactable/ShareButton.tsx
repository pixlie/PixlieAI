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
    const shareUrl = props.url ?? "https://pixlie.com";
    const emailSubject = encodeURIComponent("Check this out!");
    const emailBody = encodeURIComponent(
      `${props.title || ""}\n\nPowered by Pixlie.\n\n${shareUrl}`
    );
    const mailtoLink = `mailto:?subject=${emailSubject}&body=${emailBody}`;

    if (navigator.share) {
      try {
        await navigator.share({
          title: props.title ? `${props.title}\n\nPowered by Pixlie.` : "Powered by Pixlie.",
          url: shareUrl,
        });
      } catch (err) {
        console.error("Sharing failed:", err);
      }
    } else {
      try {
        await navigator.clipboard.writeText(shareUrl);
        setCopied(true);
      } catch (err) {
        console.error("Copy failed:", err);
        window.open(mailtoLink, "_blank");
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
