import React from "react";
import { Img, staticFile } from "remotion";
import type { Product } from "../data/brand";
import { KitchenSVG } from "./KitchenSVG";

/** Visuel d'une cuisine, ratio 3:2 : photo officielle (recadrée sans déformation) ou, à défaut, illustration SVG. */
export const ProductVisual: React.FC<{ product: Product; uid: string; reveal?: number; width: number; ratio?: number }> = ({ product, uid, reveal, width, ratio = 2 / 3 }) => (
  <div style={{ width, height: width * ratio, overflow: "hidden", background: "#1d1915" }}>
    {product.image ? (
      <Img src={staticFile(product.image)} style={{ width: "100%", height: "100%", objectFit: "cover" }} />
    ) : (
      <KitchenSVG finish={product.finish ?? "blanc"} uid={uid} reveal={reveal} />
    )}
  </div>
);
