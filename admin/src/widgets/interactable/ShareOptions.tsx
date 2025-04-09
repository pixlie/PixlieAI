import { Component, createSignal } from "solid-js";

import IconButton from "./IconButton";
import CopyIcon from "../../assets/icons/tabler-clipboard.svg";
import CopiedIcon from "../../assets/icons/tabler-clipboard-check.svg";
import MailIcon from "../../assets/icons/tabler-mail.svg";
import XIcon from "../../assets/icons/tabler-brand-x.svg";
import LinkedInIcon from "../../assets/icons/tabler-brand-linkedin.svg";
import ShareIcon from "../../assets/icons/tabler-share-2.svg";


interface IPropTypes {
  title?: string;
  url?: string;
}

const ShareOptions: Component<IPropTypes> = (props) => {
  const [copied, setCopied] = createSignal(false);
  const shareUrl = props.url ?? "https://pixlie.com";
  const shareTitle =
    (props.title ? `${props.title} | ` : "") + "Powered by Pixlie";

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(shareUrl);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Copy failed:", err);
    }
  };

  const handleEmail = () => {
    const subject = encodeURIComponent(shareTitle);
    const body = encodeURIComponent(`${shareTitle}\n\n${shareUrl}`);
    window.open(`mailto:?subject=${subject}&body=${body}`, "_blank");
  };

  const handleTwitter = () => {
    const text = encodeURIComponent(`${shareTitle} ${shareUrl}`);
    window.open(`https://twitter.com/intent/tweet?text=${text}`, "_blank");
  };

  const handleLinkedIn = () => {
    const url = encodeURIComponent(shareUrl);
    const title = encodeURIComponent(props.title ?? "Check this out");
    window.open(
      `https://www.linkedin.com/sharing/share-offsite/?url=${url}&title=${title}`,
      "_blank"
    );
  };

  const handleNativeShare = async () => {
    if (navigator.share) {
      try {
        await navigator.share({ title: shareTitle, url: shareUrl });
      } catch (err) {
        console.error("Sharing failed:", err);
      }
    }
  };

  return (
    <div class="flex items-center gap-2 -m-2">
      <IconButton
        name={copied() ? "Copied!" : "Copy"}
        icon={copied() ? <CopiedIcon /> : <CopyIcon />}
        onClick={handleCopy}
      />
      <IconButton name="Email" icon={<MailIcon />} onClick={handleEmail} />
      <IconButton name="X" icon={<XIcon />} onClick={handleTwitter} />
      <IconButton
        name="LinkedIn"
        icon={<LinkedInIcon />}
        onClick={handleLinkedIn}
      />
      <IconButton
        name="Share"
        icon={<ShareIcon />}
        onClick={handleNativeShare}
      />
    </div>
  );
};

export default ShareOptions;
