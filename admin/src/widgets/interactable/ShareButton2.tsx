import { Component, createSignal, onCleanup } from "solid-js";
import ShareIcon from "../../assets/icons/tabler-share-2.svg";
import IconButton from "./IconButton";

interface IPropTypes {
  title?: string;
  url?: string;
}

const ShareButton: Component<IPropTypes> = (props) => {
  const [copied, setCopied] = createSignal(false);
  const [menuOpen, setMenuOpen] = createSignal(false);
  const shareUrl = props.url ?? "https://pixlie.com";
  const shareTitle = (props.title ? `${props.title} | ` : "") + "Powered by Pixlie";

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(shareUrl);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
      setMenuOpen(false);
    } catch (err) {
      console.error("Copy failed:", err);
    }
  };

  const handleEmail = () => {
    const subject = encodeURIComponent(shareTitle);
    const body = encodeURIComponent(`${shareTitle}\n\n${shareUrl}`);
    window.open(`mailto:?subject=${subject}&body=${body}`, "_blank");
    setMenuOpen(false);
  };

  const handleTwitter = () => {
    const text = encodeURIComponent(`${shareTitle} ${shareUrl}`);
    window.open(`https://twitter.com/intent/tweet?text=${text}`, "_blank");
    setMenuOpen(false);
  };

  const handleLinkedIn = () => {
    const url = encodeURIComponent(shareUrl);
    const title = encodeURIComponent(props.title ?? "Check this out");
    window.open(`https://www.linkedin.com/sharing/share-offsite/?url=${url}&title=${title}`, "_blank");
    setMenuOpen(false);
  };

  const handleNativeShare = async () => {
    if (navigator.share) {
      try {
        await navigator.share({ title: shareTitle, url: shareUrl });
        setMenuOpen(false);
      } catch (err) {
        console.error("Sharing failed:", err);
      }
    }
  };

  const handleClickOutside = (e: MouseEvent) => {
    if (!(e.target as HTMLElement).closest(".share-menu")) {
      setMenuOpen(false);
    }
  };

  document.addEventListener("click", handleClickOutside);
  onCleanup(() => document.removeEventListener("click", handleClickOutside));

  return (
    <div class="relative share-menu -m-2">
      <IconButton
        name="Share"
        icon={
          <div class="flex h-6 w-6 items-center justify-center text-slate-400">
            <ShareIcon />
          </div>
        }
        onClick={() => setMenuOpen(!menuOpen())}
      />
      {menuOpen() && (
        <div class="absolute right-0 mt-2 w-40 bg-white border rounded-lg shadow-lg z-10">
          <button
            onClick={handleCopy}
            class="w-full text-left px-4 py-2 text-sm hover:bg-slate-100"
          >
            {copied() ? "Copied!" : "Copy Link"}
          </button>
          <button
            onClick={handleEmail}
            class="w-full text-left px-4 py-2 text-sm hover:bg-slate-100"
          >
            Email
          </button>
          <button
            onClick={handleTwitter}
            class="w-full text-left px-4 py-2 text-sm hover:bg-slate-100"
          >
            Twitter/X
          </button>
          <button
            onClick={handleLinkedIn}
            class="w-full text-left px-4 py-2 text-sm hover:bg-slate-100"
          >
            LinkedIn
          </button>
          {/* {navigator.share && ( */}
            <button
              onClick={handleNativeShare}
              class="w-full text-left px-4 py-2 text-sm hover:bg-slate-100"
            >
              Share via...
            </button>
      
        </div>
      )}
    </div>
  );
};

export default ShareButton;
