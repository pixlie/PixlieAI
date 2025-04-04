import { Component, createSignal, onMount } from "solid-js";
import IconButton from "../interactable/IconButton.tsx";
import GitHubIcon from "../../assets/icons/tabler-github.svg";
import StarIcon from "../../assets/icons/tabler-star.svg";

const GitHubStars: Component = () => {
  const [stars, setStars] = createSignal<number | null>(null);

  onMount(async () => {
    try {
      const res = await fetch("https://api.github.com/repos/pixlie/PixlieAI");
      const data = await res.json();
      setStars(data.stargazers_count);
    } catch (err) {
      console.error("Failed to fetch GitHub stars:", err);
    }
  });

  return (
    <div class="flex items-center">
      <a
        href="https://github.com/pixlie/PixlieAI"
        target="_blank"
        rel="noreferrer"
      >
        <IconButton name="GitHub" icon={GitHubIcon} onClick={() => {}} />
      </a>

      {!!stars() && (
        <div class="flex items-center gap-1 pl-1" style={{ color: "#FFD600" }}>
          <StarIcon />
          <p class="text-sm text-gray-800 font-semibold">{stars()}</p>
        </div>
      )}
    </div>
  );
};

export default GitHubStars;
