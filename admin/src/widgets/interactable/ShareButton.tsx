import { Component } from "solid-js";
import ShareIcon from "../../assets/icons/tabler-share-2.svg";
import IconButton from "./IconButton";

const ShareButton: Component = () => {
  const handleShare = () => {
    const shareUrl = window.location.href; // or a custom share link
    const emailSubject = encodeURIComponent("Check this out!");
    const emailBody = encodeURIComponent(`${shareUrl}`);
    const mailtoLink = `mailto:?subject=${emailSubject}&body=${emailBody}`;

    if (navigator.share) {
      navigator
        .share({
          title: "Check this out!",
          text: "",
          url: shareUrl,
        })
        .catch(console.error);
    } else {
      window.open(mailtoLink, "_blank");
    }
  };

  return <IconButton onClick={handleShare} icon={<ShareIcon />} name="Share" />;
};

export default ShareButton;
