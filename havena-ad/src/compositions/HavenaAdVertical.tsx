import React from "react";
import { AbsoluteFill, Audio, Sequence, staticFile } from "remotion";
import { SCENES } from "../animations/timing";
import { FilmFinish } from "../components/Backdrop";
import { PanelWipe } from "../components/PanelWipe";
import { BRAND } from "../data/brand";
import { ArgumentsV } from "../scenes/vertical/ArgumentsV";
import { CtaV } from "../scenes/vertical/CtaV";
import { DiscoveryV } from "../scenes/vertical/DiscoveryV";
import { HookV } from "../scenes/vertical/HookV";
import { ProductsV } from "../scenes/vertical/ProductsV";
import { COLORS } from "../styles/theme";

/** Version 9:16 (TikTok / Reels / Shorts) : mêmes données et même minutage que la version 16:9. */
export const HavenaAdVertical: React.FC = () => (
  <AbsoluteFill style={{ background: COLORS.ink }}>
    <Sequence {...SCENES.hook} name="1 Hook"><HookV /></Sequence>
    <Sequence {...SCENES.discovery} name="2 Découverte"><DiscoveryV /></Sequence>
    <Sequence {...SCENES.products} name="3 Produits"><ProductsV /></Sequence>
    <Sequence {...SCENES.arguments} name="4 Arguments"><ArgumentsV /></Sequence>
    <Sequence {...SCENES.cta} name="5 CTA"><CtaV /></Sequence>
    <PanelWipe at={SCENES.discovery.from} dir="ltr" />
    <PanelWipe at={SCENES.products.from} dir="rtl" />
    <PanelWipe at={SCENES.arguments.from} dir="ltr" />
    <PanelWipe at={SCENES.cta.from} dir="rtl" />
    <FilmFinish />
    {BRAND.music ? <Audio src={staticFile(BRAND.music)} volume={0.8} /> : null}
  </AbsoluteFill>
);
