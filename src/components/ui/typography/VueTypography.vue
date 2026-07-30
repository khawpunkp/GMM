<script setup lang="ts">
import { cva, type VariantProps } from "class-variance-authority";
import type { HTMLAttributes } from "vue";
import { cn } from "@/utils/general";

const typoVariants = cva("", {
  variants: {
    variant: {
      H1B: "text-[24px] font-[600]",
      TitleB: "text-[18px] font-[600]",
      TitleR: "text-[18px] font-[400]",
      BodyB: "text-[16px] font-[600]",
      BodyR: "text-[16px] font-[400]",
      BodyL: "text-[16px] font-[300]",
      CaptionB: "text-[14px] font-[600]",
      CaptionR: "text-[14px] font-[400]",
      CaptionL: "text-[14px] font-[300]",
      SubL: "text-[12px] font-[300]",
      FooterNav: "text-[12px] font-[400]",
    },
  },
  defaultVariants: {
    variant: "BodyL",
  },
});

type TypoVariants = VariantProps<typeof typoVariants>;

interface Props {
  variant?: TypoVariants["variant"];
  as?: keyof HTMLElementTagNameMap;
  class?: HTMLAttributes["class"];
}

const props = withDefaults(defineProps<Props>(), {
  as: "p",
});
</script>

<template>
  <component
    :is="as"
    :class="
      cn(
        'leading-normal whitespace-pre-line text-inherit',
        typoVariants({ variant }),
        props.class,
      )
    "
  >
    <slot />
  </component>
</template>
