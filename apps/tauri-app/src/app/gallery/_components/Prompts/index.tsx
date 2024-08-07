"use client";

import React, { useEffect, useState } from "react";
import Link from "next/link";
import { getVersion } from "@tauri-apps/api/app";
import { fetch } from "@tauri-apps/plugin-http";
import { error } from "@tauri-apps/plugin-log";
import { compare } from "compare-versions";

import type { GalleryPromptData, GalleryPromptsData } from "./types";
import { PromptsGrid } from "~/app/_components/PromptCard";
import Card from "./components/Card";

const Prompts: React.FC = () => {
  const [status, setStatus] = useState<
    "loading" | "deprecated" | "success" | "error"
  >("loading");
  const [prompts, setPrompts] = useState<GalleryPromptData[]>([]);

  useEffect(() => {
    const handler = async () => {
      const resp = await fetch(
        "https://raw.githubusercontent.com/auto-wallpaper/auto-wallpaper/gallery/data.json",
      );

      if (!resp.ok) {
        void error(
          `Error during fetching gallery data.json: ${await resp.text()}`,
        );
        setStatus("error");
        return;
      }

      const data = (await resp.json()) as GalleryPromptsData;

      setStatus(
        compare(await getVersion(), data.minVersion, ">=")
          ? "success"
          : "deprecated",
      );

      setPrompts(data.prompts);
    };

    void handler();
  }, []);

  if (status === "loading") {
    return null;
  }

  if (status === "error") {
    return (
      <p className="text-center text-sm">
        An unexpected error during getting the data. please report the issue in
        our{" "}
        <Link
          href="https://github.com/auto-wallpaper/auto-wallpaper/issues"
          className="text-zinc-400 underline underline-offset-4 transition hover:text-zinc-500"
        >
          github repository
        </Link>
      </p>
    );
  }

  if (status === "deprecated") {
    return (
      <p className="text-center text-sm">
        Your current version is deprecated. To see the gallery you must update
        your Auto Wallpaper app through the{" "}
        <Link
          href="/settings"
          className="text-zinc-400 underline underline-offset-4 transition hover:text-zinc-500"
        >
          settings
        </Link>
      </p>
    );
  }

  return (
    <PromptsGrid sortable={false}>
      {prompts.map((prompt) => (
        <Card key={prompt.id} prompt={prompt} />
      ))}
    </PromptsGrid>
  );
};

export default Prompts;
