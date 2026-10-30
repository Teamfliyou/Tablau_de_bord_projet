import React from "react";
import { AbsoluteFill, Audio, Sequence, staticFile } from "remotion";
import { SCENES } from "../animations/timing";
import { FilmFinish } from "../components/Backdrop";
import { PanelWipe } from "../components/PanelWipe";
import { BRAND } from "../data/brand";
import { ArgumentsScene } from "../scenes/ArgumentsScene";
import { CtaScene } from "../scenes/CtaScene";
import { DiscoveryScene } from "../scenes/DiscoveryScene";
import { HookScene } from "../scenes/HookScene";
import { ProductsScene } from "../scenes/ProductsScene";
import { COLORS } from "../styles/theme";

export const HavenaAd: React.FC = () => (
  <AbsoluteFill style={{ background: COLORS.ink }}>
    <Sequence {...SCENES.hook} name="1 Hook"><HookScene /></Sequence>
    <Sequence {...SCENES.discovery} name="2 Découverte"><DiscoveryScene /></Sequence>
    <Sequence {...SCENES.products} name="3 Produits"><ProductsScene /></Sequence>
    <Sequence {...SCENES.arguments} name="4 Arguments"><ArgumentsScene /></Sequence>
    <Sequence {...SCENES.cta} name="5 CTA"><CtaScene /></Sequence>
    <PanelWipe at={SCENES.discovery.from} dir="ltr" />
    <PanelWipe at={SCENES.products.from} dir="rtl" />
    <PanelWipe at={SCENES.arguments.from} dir="ltr" />
    <PanelWipe at={SCENES.cta.from} dir="rtl" />
    <FilmFinish />
    {BRAND.music ? <Audio src={staticFile(BRAND.music)} volume={0.8} /> : null}
  </AbsoluteFill>
);
