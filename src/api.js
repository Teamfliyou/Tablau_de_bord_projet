// Pont vers le backend Rust : toutes les opérations passent par des commandes Tauri (IPC), jamais par HTTP.
import { invoke } from '@tauri-apps/api/core';

const dansTauri = () => typeof window !== 'undefined' && window.__TAURI_INTERNALS__ !== undefined;

let moquette = null;

export async function appeler(commande, args = {}) {
  try {
    if (!dansTauri()) {
      // Aperçu navigateur (npm run dev) uniquement : jamais inclus dans l'application distribuée.
      if (import.meta.env.DEV) {
        moquette ??= (await import('./mock.js')).default;
        return await moquette(commande, args);
      }
      throw new Error("Cette application doit être lancée via l'exécutable Tableau de bord étudiant.");
    }
    return await invoke(commande, args);
  } catch (e) {
    throw new Error(typeof e === 'string' ? e : (e && e.message) || 'Erreur inattendue.');
  }
}
