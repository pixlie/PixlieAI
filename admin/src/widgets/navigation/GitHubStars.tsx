import { Component, createSignal, onMount } from "solid-js";
import ToolTip from "./ToolTip";
import Icon from "../interactable/Icon.tsx";

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

  {
    /* <img alt="GitHub Repo stars" class="ml-2" src="https://img.shields.io/github/stars/pixlie/PixlieAI?style=plastic&logo=github&color=%23eac54f" /> */
  }

  {
    /* <a
          href="https://github.com/pixlie/PixlieAI"
          target="_blank"
          rel="noreferrer"
          class="flex items-center"
        >
    
                  <ToolTip text="GitHub">
                    <div class="p-2 text-gray-800 hover:text-gray-950 hover:bg-slate-200 rounded-full">
       
        <Icon name="brand-github" />
        </div>
       
        </ToolTip>
        <div class="flex items-center gap-0.5">
          <Icon name="star" size={12} color="#eac54f" />
          <p class="text-sm font-semibold">{stars()}</p>
        </div>
      
        </a> */
  }

  return (
    <div class="flex items-center">
      <ToolTip text="GitHub">
        <a
          href="https://github.com/pixlie/PixlieAI"
          target="_blank"
          rel="noreferrer"
          class="flex items-center p-2 text-gray-800 hover:text-gray-950 hover:bg-slate-200 rounded-full"
        >
          <Icon name="github" />
        </a>
      </ToolTip>
      {!!stars() && (
        <div class="flex items-center gap-1 pl-1">
          <Icon name="star" size={14} color="#FFD600" />
          <p class="text-sm font-semibold">{stars()}</p>
        </div>
      )}
    </div>
  );
};

export default GitHubStars;
