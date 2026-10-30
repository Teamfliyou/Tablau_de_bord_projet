import type { DataPoint, IconName, ParsedScript, SceneType, ThemeName } from "./types";

const ICONS: IconName[] = [
  "rocket", "bulb", "chart", "clock", "target", "shield", "heart", "star",
  "bolt", "check", "users", "globe", "code", "play", "arrow", "money",
];
const SCENE_TYPES: SceneType[] = ["hook", "stat", "bars", "line", "list", "question", "outro"];
const THEME_NAMES: ThemeName[] = ["neon", "sunset", "ocean"];

/** Mots-clés → icône, pour choisir automatiquement un pictogramme. */
const ICON_KEYWORDS: [RegExp, IconName][] = [
  [/argent|euro|€|\$|prix|revenu|money|gagn/i, "money"],
  [/temps|heure|minute|seconde|rapide|vite|clock/i, "clock"],
  [/idée|idee|astuce|conseil|créati|creati|bulb/i, "bulb"],
  [/lanc|démarr|demarr|boost|croiss|rocket/i, "rocket"],
  [/objectif|cible|but\b|target/i, "target"],
  [/sécur|secur|protég|proteg|shield/i, "shield"],
  [/amour|coeur|cœur|passion|heart/i, "heart"],
  [/communaut|équipe|equipe|client|users|abonn/i, "users"],
  [/monde|global|international|globe/i, "globe"],
  [/code|programm|développ|develop/i, "code"],
  [/énergie|energie|puissan|éclair|eclair/i, "bolt"],
  [/résultat|resultat|donnée|donnee|statist|chart|%/i, "chart"],
  [/étoile|etoile|meilleur|qualité|qualite/i, "star"],
];

const TYPE_DEFAULT_ICON: Record<SceneType, IconName> = {
  hook: "bolt", stat: "chart", bars: "chart", line: "rocket",
  list: "check", question: "bulb", outro: "play",
};

export function wordsOf(text: string): string[] {
  return text.trim().split(/\s+/).filter(Boolean);
}

/** Parse `key=value key2="valeur avec espaces"`. */
export function parseAttrs(src: string): Record<string, string> {
  const out: Record<string, string> = {};
  const re = /([\w-]+)=(?:"([^"]*)"|(\S+))/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(src))) out[m[1]] = m[2] ?? m[3];
  return out;
}

/** "2021:12,2022:25" → points de données. */
export function parseData(src: string | undefined): DataPoint[] | undefined {
  if (!src) return undefined;
  const pts = src
    .split(",")
    .map((p) => p.split(":"))
    .filter((p) => p.length === 2 && Number.isFinite(Number(p[1])))
    .map(([label, value]) => ({ label: label.trim(), value: Number(value) }));
  return pts.length >= 2 ? pts : undefined;
}

function pickIcon(text: string, type: SceneType, explicit?: string): IconName {
  if (explicit && (ICONS as string[]).includes(explicit)) return explicit as IconName;
  for (const [re, icon] of ICON_KEYWORDS) if (re.test(text)) return icon;
  return TYPE_DEFAULT_ICON[type];
}

/** Extrait "87 %", "3x", "1,5 M€"… du texte. */
export function extractNumber(text: string) {
  const m = text.match(/(\d[\d\s]*(?:[.,]\d+)?)\s*(%|x|×|k|M|Md|€|\$)?/);
  if (!m) return undefined;
  const value = Number(m[1].replace(/\s/g, "").replace(",", "."));
  if (!Number.isFinite(value)) return undefined;
  const unit = m[2];
  const isPrefix = unit === "$";
  return { value, prefix: isPrefix ? unit : "", suffix: unit && !isPrefix ? (unit === "×" ? "x" : unit) : "" };
}

function firstSentence(text: string): string {
  const m = text.match(/^[^.!?…]+[.!?…]?/);
  return (m ? m[0] : text).trim();
}

function guessTitle(text: string, type: SceneType): string {
  const s = firstSentence(text).replace(/[.!…]+$/, "");
  const limit = type === "list" ? 28 : 70;
  const w = wordsOf(s);
  let out = "";
  for (const word of w) {
    if ((out + " " + word).trim().length > limit) break;
    out = (out + " " + word).trim();
  }
  return out || s;
}

function guessType(block: string, index: number, total: number, hasBullets: boolean): SceneType {
  if (hasBullets) return "list";
  if (index === 0) return "hook";
  if (index === total - 1 && total > 1) return "outro";
  if (/\?\s*$/.test(firstSentence(block))) return "question";
  if (/\d/.test(block) && /%|x\b|×|fois|million|milliard/i.test(block)) return "stat";
  return "question";
}

/** Sans saut de ligne : regroupe les phrases par paires pour former des scènes. */
function splitIntoBlocks(body: string): string[] {
  const paragraphs = body.split(/\n\s*\n/).map((p) => p.trim()).filter(Boolean);
  if (paragraphs.length > 1) return paragraphs;
  const sentences = body.replace(/\s+/g, " ").match(/[^.!?…]+[.!?…]+|\S[^.!?…]*$/g) ?? [body];
  const blocks: string[] = [];
  for (let i = 0; i < sentences.length; i += 2) {
    blocks.push(sentences.slice(i, i + 2).join(" ").trim());
  }
  return blocks;
}

export function parseScript(source: string): ParsedScript {
  const headers: Record<string, string> = {};
  const bodyLines: string[] = [];
  for (const line of source.replace(/\r/g, "").split("\n")) {
    const h = line.match(/^@(\w+)\s+(.+)$/);
    if (h) headers[h[1].toLowerCase()] = h[2].trim();
    else if (!line.trim().startsWith("//")) bodyLines.push(line);
  }
  const theme = (THEME_NAMES as string[]).includes(headers.theme) ? (headers.theme as ThemeName) : "neon";
  const blocks = splitIntoBlocks(bodyLines.join("\n"));

  const scenes = blocks.map((raw, i) => {
    let block = raw;
    let attrs: Record<string, string> = {};
    let explicitType: SceneType | undefined;
    const dir = block.match(/^\[(\w+)([^\]]*)\]\s*/);
    if (dir && (SCENE_TYPES as string[]).includes(dir[1])) {
      explicitType = dir[1] as SceneType;
      attrs = parseAttrs(dir[2]);
      block = block.slice(dir[0].length);
    }
    const lines = block.split("\n").map((l) => l.trim()).filter(Boolean);
    const bullets = lines.filter((l) => /^([-•*]|\d+[.)])\s+/.test(l));
    const items = bullets.map((l) => l.replace(/^([-•*]|\d+[.)])\s+/, ""));
    const prose = lines.filter((l) => !bullets.includes(l)).join(" ");
    const text = [prose, ...items].filter(Boolean).join(". ").replace(/\.\./g, ".").replace(/\s+/g, " ").trim();

    let type = explicitType ?? guessType(text, i, blocks.length, items.length >= 2);
    const data = parseData(attrs.data);
    if ((type === "bars" || type === "line") && !data) type = "stat"; // pas de données inventées
    if (type === "list" && items.length < 2) type = "question";

    const num = extractNumber(text);
    const value = attrs.value !== undefined ? Number(attrs.value) : num?.value;
    if (type === "stat" && value === undefined) type = "question";

    return {
      id: i,
      type,
      text,
      title: attrs.title ?? guessTitle(prose || text, type),
      icon: pickIcon(text, type, attrs.icon),
      value,
      prefix: attrs.prefix ?? (attrs.value !== undefined ? "" : num?.prefix),
      suffix: attrs.suffix ?? (attrs.value !== undefined ? "" : num?.suffix),
      cta: attrs.cta,
      data,
      items: type === "list" ? items : undefined,
    };
  });

  return {
    title: headers.title ?? "Vidéo",
    theme,
    voice: headers.voice,
    targetSeconds: headers.duration ? Number(headers.duration) : undefined,
    scenes,
  };
}
