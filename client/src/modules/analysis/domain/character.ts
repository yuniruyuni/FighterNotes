export const CHARACTER_CATALOG = [
  { id: "A_K_I", label: "A.K.I." },
  { id: "AKUMA", label: "AKUMA" },
  { id: "ALEX", label: "ALEX" },
  { id: "BLANKA", label: "BLANKA" },
  { id: "C_VIPER", label: "C.VIPER" },
  { id: "CAMMY", label: "CAMMY" },
  { id: "CHUN_LI", label: "CHUN-LI" },
  { id: "DEE_JAY", label: "DEE JAY" },
  { id: "DHALSIM", label: "DHALSIM" },
  { id: "E_HONDA", label: "E.HONDA" },
  { id: "ED", label: "ED" },
  { id: "ELENA", label: "ELENA" },
  { id: "GUILE", label: "GUILE" },
  { id: "INGRID", label: "INGRID" },
  { id: "JAMIE", label: "JAMIE" },
  { id: "JP", label: "JP" },
  { id: "JURI", label: "JURI" },
  { id: "KEN", label: "KEN" },
  { id: "KIMBERLY", label: "KIMBERLY" },
  { id: "LILY", label: "LILY" },
  { id: "LUKE", label: "LUKE" },
  { id: "M_BISON", label: "M.BISON" },
  { id: "MAI", label: "MAI" },
  { id: "MANON", label: "MANON" },
  { id: "MARISA", label: "MARISA" },
  { id: "RASHID", label: "RASHID" },
  { id: "RYU", label: "RYU" },
  { id: "SAGAT", label: "SAGAT" },
  { id: "TERRY", label: "TERRY" },
  { id: "ZANGIEF", label: "ZANGIEF" },
] as const;

export type CharacterId = (typeof CHARACTER_CATALOG)[number]["id"];

export const CHARACTER_IDS: readonly CharacterId[] = CHARACTER_CATALOG.map(
  ({ id }) => id,
);

const characterIds = new Set<string>(CHARACTER_IDS);

export function isCharacterId(value: string): value is CharacterId {
  return characterIds.has(value);
}

export function formatCharacterId(value: string | undefined): string {
  return value?.replaceAll("_", "-") || "未指定";
}
