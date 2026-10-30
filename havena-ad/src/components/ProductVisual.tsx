import React from "react";
import { Img, staticFile } from "remotion";
import type { Product } from "../data/brand";
import { KitchenSVG } from "./KitchenSVG";

/** Visuel d'une cuisine : vraie photo si `image` est renseignée, sinon illustration SVG. Ratio 3:2, jamais déformé. */
export const ProductVisual: React.FC<{ product: Product; uid: string; reveal?: number; width: number }> = ({ product, uid, reveal, width }) => (
  <div style={{ width, height: (width * 2) / 3, overflow: "hidden" }}>
    {product.image ? (
      <Img src={staticFile(product.image)} style={{ width: "100%", height: "100%", objectFit: "cover" }} />
    ) : (
      <KitchenSVG finish={product.finish} uid={uid} reveal={reveal} />
    )}
  </div>
);
