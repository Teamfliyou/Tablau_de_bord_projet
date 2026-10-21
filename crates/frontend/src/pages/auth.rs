//! Écran de connexion / création de compte (version Web uniquement).

use crate::api;
use crate::icons::{Icone, Spinner};
use crate::state::Etat;
use crate::ui::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use tdb_shared::{Connexion, Inscription};

#[component]
pub fn Authentification() -> impl IntoView {
    let etat = expect_context::<Etat>();
    let creation = RwSignal::new(false);
    let email = RwSignal::new(String::new());
    let mot_de_passe = RwSignal::new(String::new());
    let pseudo = RwSignal::new(String::new());
    let erreur = RwSignal::new(String::new());
    let en_cours = RwSignal::new(false);

    let valider = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        spawn_local(async move {
            en_cours.set(true);
            erreur.set(String::new());
            let e = email.get_untracked().trim().to_string();
            let mdp = mot_de_passe.get_untracked();
            let reponse = if creation.get_untracked() {
                api::inscription(&Inscription {
                    email: e,
                    password: mdp,
                    username: pseudo.get_untracked(),
                })
                .await
            } else {
                api::connexion(&Connexion {
                    email: e,
                    password: mdp,
                })
                .await
            };
            match reponse {
                Ok(_) => {
                    mot_de_passe.set(String::new());
                    etat.authentifie.set(true);
                }
                Err(err) => erreur.set(err.message),
            }
            en_cours.set(false);
        });
    };

    view! {
        <main class="min-h-screen flex items-center justify-center px-4 py-10">
            <div class=format!("w-full max-w-md {CARTE} p-8")>
                <div class="flex items-center gap-3">
                    <div class="w-11 h-11 rounded-2xl bg-gradient-to-br from-indigo-600 to-violet-600 flex items-center justify-center shadow-lg shadow-indigo-500/30">
                        <Icone nom="mortarboard" class="w-6 h-6 text-white" />
                    </div>
                    <div>
                        <h1 class="text-xl font-bold text-slate-900 leading-tight">"Tableau de bord étudiant"</h1>
                        <p class="text-sm text-slate-500">{move || if creation.get() { "Crée ton compte" } else { "Connecte-toi pour continuer" }}</p>
                    </div>
                </div>

                <form class="mt-8 space-y-4" on:submit=valider>
                    {move || creation.get().then(|| view! {
                        <div>
                            <label class="block text-sm font-medium text-slate-700 mb-1.5">"Pseudo " <span class="text-slate-400">"(optionnel)"</span></label>
                            <input class=CHAMP_PLEIN type="text" maxlength="80" autocomplete="nickname"
                                prop:value=move || pseudo.get() on:input=move |ev| pseudo.set(event_target_value(&ev)) />
                        </div>
                    })}
                    <div>
                        <label class="block text-sm font-medium text-slate-700 mb-1.5">"Adresse e-mail"</label>
                        <input class=CHAMP_PLEIN type="email" required autocomplete="email" placeholder="prenom.nom@exemple.fr"
                            prop:value=move || email.get() on:input=move |ev| email.set(event_target_value(&ev)) />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-slate-700 mb-1.5">"Mot de passe"</label>
                        <input class=CHAMP_PLEIN type="password" required maxlength="72" minlength="8"
                            autocomplete=move || if creation.get() { "new-password" } else { "current-password" }
                            prop:value=move || mot_de_passe.get() on:input=move |ev| mot_de_passe.set(event_target_value(&ev)) />
                        {move || creation.get().then(|| view! { <p class="text-xs text-slate-400 mt-1.5">"8 caractères minimum."</p> })}
                    </div>

                    {move || { let m = erreur.get(); (!m.is_empty()).then(|| view! { <p class="rounded-xl bg-rose-50 border border-rose-200 text-rose-600 text-sm p-3.5">{m}</p> }) }}

                    <button type="submit" class=BOUTON_LARGE prop:disabled=move || en_cours.get()>
                        {move || en_cours.get().then(|| view! { <Spinner class="w-4 h-4" /> })}
                        {move || if creation.get() { "Créer mon compte" } else { "Se connecter" }}
                    </button>
                </form>

                <p class="mt-6 text-center text-sm text-slate-500">
                    {move || if creation.get() { "Déjà un compte ? " } else { "Pas encore de compte ? " }}
                    <button type="button" class="font-medium text-indigo-600 hover:text-indigo-500"
                        on:click=move |_| { creation.update(|c| *c = !*c); erreur.set(String::new()); }>
                        {move || if creation.get() { "Se connecter" } else { "Créer un compte" }}
                    </button>
                </p>
            </div>
        </main>
    }
}
