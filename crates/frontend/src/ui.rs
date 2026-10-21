//! Classes Tailwind partagées par les pages (jeu de styles « Liquid Glass »).

pub const CARTE: &str =
    "bg-white/60 backdrop-blur-2xl rounded-3xl border border-white/40 shadow-lg shadow-indigo-500/5";
pub const CHAMP: &str = "rounded-xl border border-white/50 bg-white/60 backdrop-blur px-3.5 py-2.5 text-sm placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent";
pub const CHAMP_PLEIN: &str = "w-full rounded-xl border border-white/50 bg-white/60 backdrop-blur px-3.5 py-2.5 text-sm placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent";
pub const CHAMP_MONO: &str = "w-full rounded-xl border border-white/50 bg-white/60 backdrop-blur px-3.5 py-2.5 text-sm font-mono placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent";
pub const BOUTON_PRIMAIRE: &str = "inline-flex items-center gap-2 px-3.5 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-60 text-white text-sm font-medium shadow-sm transition";
pub const BOUTON_LARGE: &str = "w-full inline-flex items-center justify-center gap-2 px-5 py-3 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-60 text-white text-sm font-semibold shadow-sm transition";
pub const BOUTON_PILULE: &str = "inline-flex items-center gap-1.5 px-3.5 py-2 rounded-xl text-sm font-medium border transition bg-white/60 backdrop-blur text-slate-600 border-white/50 hover:border-indigo-300 hover:text-indigo-600";
pub const BOUTON_ICONE: &str = "inline-flex items-center justify-center w-10 h-10 rounded-xl bg-white/60 backdrop-blur-xl border border-white/40 text-slate-500 hover:text-indigo-600 hover:border-indigo-200 shadow-sm transition";
pub const ALERTE_ERREUR: &str =
    "rounded-xl bg-rose-50 border border-rose-200 text-rose-600 text-sm p-4";
pub const ALERTE_OK: &str =
    "rounded-xl bg-emerald-50 border border-emerald-200 text-emerald-700 text-sm p-3.5";
pub const ALERTE_HORS_LIGNE: &str =
    "rounded-xl bg-amber-50 border border-amber-200 text-amber-700 text-sm p-4";
pub const VIDE: &str =
    "rounded-2xl bg-white/40 backdrop-blur-xl border border-dashed border-white/60 text-center";
