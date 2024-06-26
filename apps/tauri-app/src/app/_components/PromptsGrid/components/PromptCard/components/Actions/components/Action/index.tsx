import type { IconType } from "react-icons/lib";
import React from "react";

import { cn } from "@acme/ui";
import { Dialog, DialogContent, DialogTrigger } from "@acme/ui/dialog";

import { ActionButton } from "~/app/_components/PromptCard";

type ActionProps = React.PropsWithChildren<{
  Icon: IconType;
  className?: {
    trigger?: string;
    content?: string;
  };
  onOpenChange?: (open: boolean) => void;
  open?: boolean;
  disabled?: boolean;
}>;

const Action: React.FC<ActionProps> = ({
  Icon,
  children,
  className,
  onOpenChange,
  open,
  disabled,
}) => {
  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogTrigger asChild>
        <ActionButton
          className={className?.trigger}
          Icon={Icon}
          disabled={disabled}
        />
      </DialogTrigger>
      <DialogContent
        className={cn("max-w-[35rem]", className?.content)}
        onOpenAutoFocus={(e) => e.preventDefault()}
        data-no-dnd
      >
        {children}
      </DialogContent>
    </Dialog>
  );
};

export default Action;
