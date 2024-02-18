"use client";

import type { ImageProps } from "next/image";
import type { IconType } from "react-icons/lib";
import React, { createContext, forwardRef, useContext } from "react";
import Image from "next/image";
import { MdOutlineImageNotSupported } from "react-icons/md";

import type { ClassValue } from "@acme/ui";
import { cn } from "@acme/ui";
import { Button } from "@acme/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@acme/ui/tooltip";

import { VARIABLE_REGEX } from "~/lib/PromptEngine";

export type PromptCardData = {
  id: string;
  prompt: string;
  image: {
    source: ImageProps["src"] | null;
    status: "loading" | "finished" | "error";
  };
};

const PromptContext = createContext<PromptCardData>(null as never);

export const usePromptContext = () => {
  return useContext(PromptContext);
};

const Box: React.FC<React.PropsWithChildren> = ({ children }) => {
  return (
    <div
      className="w-max rounded-md bg-zinc-900/75"
      onClick={(e) => e.stopPropagation()}
    >
      {children}
    </div>
  );
};

type PromptCardProps = PromptCardData & {
  className?: {
    root?: ClassValue;
  };
  actions: React.ReactNode;
  onSelect?: () => void;
};

export const PromptCard = forwardRef<HTMLDivElement, PromptCardProps>(
  ({ id, image, prompt, className, actions, onSelect }, ref) => {
    return (
      <PromptContext.Provider value={{ id, prompt, image }}>
        <div
          ref={ref}
          id={id}
          className={cn(
            "relative col-span-1 flex aspect-video flex-col rounded-md border border-zinc-800 bg-transparent bg-zinc-950 px-2 py-4 transition",
            className?.root,
            onSelect && "cursor-pointer",
          )}
          onClick={() => onSelect?.()}
        >
          <span className="absolute bottom-0 left-0 right-0 top-0 flex h-full w-full flex-col items-center justify-center text-sm">
            {image.source ? (
              <Image
                src={image.source}
                alt=""
                fill
                className="rounded-md object-cover"
              />
            ) : image.status === "loading" ? (
              <div className="text-center">
                <p>Loading the Image</p>
              </div>
            ) : image.status === "finished" ? (
              <div className="text-center">
                <div className="flex items-center justify-center gap-1">
                  <MdOutlineImageNotSupported />
                  <p>No Image Yet.</p>
                </div>
                <p>Generate one to show here</p>
              </div>
            ) : (
              <div className="text-center">
                <p>Unexpected Error</p>
                <p>during loading image</p>
              </div>
            )}
          </span>

          <div className="absolute right-0 top-0 z-50 flex w-full cursor-default justify-end gap-1 p-2">
            <Box>{actions}</Box>
          </div>

          <div
            className={cn(
              "absolute bottom-0 left-0 z-10 mt-auto h-max w-full rounded-b-md px-2 pb-4 pt-32",
              image.source && "bg-gradient-to-t from-black/90 to-transparent",
            )}
          >
            <p className="line-clamp-3 text-center text-xs">
              {prompt.split(VARIABLE_REGEX).map((word, i) =>
                i % 2 === 1 ? (
                  <span key={i} className="font-mono text-zinc-400">
                    ${word}
                  </span>
                ) : (
                  <span key={i}>{word}</span>
                ),
              )}
            </p>
          </div>
        </div>
      </PromptContext.Provider>
    );
  },
);

PromptCard.displayName = "PromptCard"

export const PromptsGrid: React.FC<React.PropsWithChildren> = ({
  children,
}) => {
  return (
    <div className="grid h-max grid-cols-2 gap-2 lg:grid-cols-3 lg:gap-5">
      {children}
    </div>
  );
};

type ButtonProps = {
  Icon: IconType;
  className?: string;
  onClick?: () => void;
  disabled?: boolean;
  tooltip?: React.ReactNode;
};

export const ActionButton = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ Icon, className, onClick, disabled, tooltip }, ref) => {
    return (
      <TooltipProvider>
        <Tooltip open={typeof tooltip === "undefined" ? false : undefined}>
          <TooltipTrigger ref={ref} className="cursor-pointer" asChild>
            <Button
              size="icon"
              className={cn(
                "bg-transparent shadow-none transition-all hover:bg-zinc-950/30 disabled:cursor-default disabled:opacity-25",
                className,
              )}
              onClick={onClick}
              disabled={disabled}
            >
              <Icon size={18} />
            </Button>
          </TooltipTrigger>
          <TooltipContent>{tooltip}</TooltipContent>
        </Tooltip>
      </TooltipProvider>
    );
  },
);

ActionButton.displayName = "ActionButton";
