<template>
  <div id="app" class="min-h-screen">

    <!-- ==================== SIDEBAR CLASSIQUE (option "Classic Sidebar") ==================== -->
    <aside
      v-if="interfaceStyle === 'classic'"
      class="hidden lg:flex fixed inset-y-0 left-0 w-64 flex-col bg-white/60 backdrop-blur-2xl border-r border-white/40 px-4 py-8">
      <div class="flex items-center gap-3 px-2">
        <div class="w-11 h-11 rounded-2xl bg-gradient-to-br from-indigo-600 to-violet-600 flex items-center justify-center shadow-lg shadow-indigo-500/30">
          <svg class="w-6 h-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 14l9-5-9-5-9 5 9 5z" />
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 14l6.16-3.42a12.08 12.08 0 01.84 6.42M12 14v6M5 10v6a4 4 0 0014 0" />
          </svg>
        </div>
        <div>
          <h1 class="text-lg font-bold text-slate-900 leading-tight">Tableau de bord</h1>
          <p class="text-xs text-slate-500">{{ identifiantAffichable }}</p>
        </div>
      </div>

      <nav class="mt-10 space-y-1.5 flex-1">
        <button
          v-for="onglet in onglets"
          :key="onglet.id"
          @click="ongletActif = onglet.id"
          class="w-full flex items-center gap-3 px-3.5 py-3 rounded-xl text-sm font-medium transition"
          :class="ongletActif === onglet.id ? 'bg-gradient-to-r from-indigo-500 to-violet-500 text-white shadow-md shadow-indigo-500/30' : 'text-slate-500 hover:bg-white/60 hover:text-slate-800'">
          <app-icon :name="onglet.icone" class="w-5 h-5 shrink-0"></app-icon>
          {{ onglet.label }}
        </button>
      </nav>

      <div class="pt-4 border-t border-white/40 px-2">
        <p class="text-xs text-slate-400">{{ dateDuJour }}</p>
      </div>

      <button
        type="button"
        @click="basculerTheme"
        class="mt-4 w-full flex items-center gap-3 px-3.5 py-3 rounded-xl text-sm font-medium text-slate-500 hover:bg-white/60 hover:text-slate-800 transition">
        <svg class="w-5 h-5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" /></svg>
        {{ themeSombre ? 'Mode clair' : 'Mode sombre' }}
      </button>
    </aside>

    <!-- ==================== BARRE MOBILE (option "Classic Sidebar") ==================== -->
    <div
      v-if="interfaceStyle === 'classic'"
      class="lg:hidden sticky top-0 z-30 bg-white/60 backdrop-blur-xl border-b border-white/40 px-3 py-2 flex gap-1.5 overflow-x-auto">
      <button
        v-for="onglet in onglets"
        :key="onglet.id"
        @click="ongletActif = onglet.id"
        class="shrink-0 inline-flex items-center gap-1.5 px-3 py-2 rounded-xl text-sm font-medium transition"
        :class="ongletActif === onglet.id ? 'bg-gradient-to-r from-indigo-500 to-violet-500 text-white shadow-sm' : 'bg-white/60 text-slate-600'">
        <app-icon :name="onglet.icone" class="w-5 h-5 shrink-0"></app-icon>
        <span>{{ onglet.label }}</span>
      </button>
    </div>

    <!-- ==================== DOCK DE NAVIGATION (bas d'écran, style iOS/macOS - par défaut) ==================== -->
    <nav v-if="interfaceStyle === 'dock'" class="fixed bottom-5 left-1/2 -translate-x-1/2 z-50 flex items-center gap-0.5 sm:gap-1 p-1 sm:p-1.5 rounded-full bg-white/30 backdrop-blur-xl border border-white/20 shadow-2xl shadow-indigo-900/10">
      <button
        v-for="onglet in onglets"
        :key="onglet.id"
        :title="onglet.label"
        @click="ongletActif = onglet.id"
        class="relative flex flex-col items-center justify-center gap-1 px-2 sm:px-4 py-2 sm:py-2.5 rounded-full transition-all duration-300"
        :class="ongletActif === onglet.id ? 'text-white' : 'text-slate-500 hover:text-slate-800'">
        <span
          v-if="ongletActif === onglet.id"
          class="absolute inset-0 rounded-full bg-gradient-to-br from-indigo-500 via-violet-500 to-fuchsia-500 dock-active"
          aria-hidden="true"></span>
        <app-icon :name="onglet.icone" class="relative w-6 h-6 shrink-0"></app-icon>
        <span class="relative text-[9px] sm:text-[10px] font-semibold leading-none whitespace-nowrap">{{ onglet.label }}</span>
      </button>
      <span class="w-px h-8 bg-white/40 mx-1 sm:mx-2 hidden sm:block" aria-hidden="true"></span>
      <button
        type="button"
        @click="basculerTheme"
        :title="themeSombre ? 'Passer en mode clair' : 'Passer en mode sombre'"
        class="relative flex flex-col items-center justify-center gap-1 px-2 sm:px-4 py-2 sm:py-2.5 rounded-full transition-all duration-300 text-slate-500 hover:text-slate-800">
        <svg class="relative w-6 h-6 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" /></svg>
        <span class="relative text-[9px] sm:text-[10px] font-semibold leading-none whitespace-nowrap">{{ themeSombre ? 'Clair' : 'Sombre' }}</span>
      </button>
    </nav>

    <!-- ==================== CONTENU ==================== -->
    <main
      class="flex-1 px-4 sm:px-6 lg:px-10 pt-10 sm:pt-14"
      :class="interfaceStyle === 'classic' ? 'lg:ml-64 pb-16' : 'pb-28'">

      <!-- ============ ONGLET : DASHBOARD ============ -->
      <section v-if="ongletActif === 'dashboard'" class="max-w-7xl mx-auto">
        <header class="flex flex-wrap items-center justify-between gap-4">
          <div>
            <h2 class="text-2xl font-bold text-slate-900">Dashboard</h2>
            <p class="text-sm text-slate-500">{{ dateDuJour }}</p>
          </div>
          <span class="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-white/60 backdrop-blur-xl border border-white/40 shadow-sm text-sm text-slate-600">
            <span class="relative flex h-2 w-2">
              <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
              <span class="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
            </span>
            {{ heureActuelle }}
          </span>
        </header>

        <!-- Statistiques -->
        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4 mt-8">
          <div class="bg-white/60 backdrop-blur-2xl rounded-3xl border border-white/40 shadow-lg shadow-indigo-500/5 p-5">
            <p class="text-xs font-medium text-slate-400 uppercase tracking-wide">Cours aujourd'hui</p>
            <p class="text-2xl font-bold text-slate-900 mt-1">{{ cours.length }}</p>
          </div>
          <div class="bg-white/60 backdrop-blur-2xl rounded-3xl border border-white/40 shadow-lg shadow-indigo-500/5 p-5">
            <p class="text-xs font-medium text-slate-400 uppercase tracking-wide">À faire</p>
            <p class="text-2xl font-bold text-rose-600 mt-1">{{ nbStatut('a_faire') }}</p>
          </div>
          <div class="bg-white/60 backdrop-blur-2xl rounded-3xl border border-white/40 shadow-lg shadow-indigo-500/5 p-5">
            <p class="text-xs font-medium text-slate-400 uppercase tracking-wide">En cours</p>
            <p class="text-2xl font-bold text-amber-500 mt-1">{{ nbStatut('en_cours') }}</p>
          </div>
          <div class="bg-white/60 backdrop-blur-2xl rounded-3xl border border-white/40 shadow-lg shadow-indigo-500/5 p-5">
            <p class="text-xs font-medium text-slate-400 uppercase tracking-wide">Terminés</p>
            <p class="text-2xl font-bold text-emerald-600 mt-1">{{ nbStatut('termine') }}</p>
          </div>
        </div>

        <!-- Grille bento -->
        <div class="grid grid-cols-1 lg:grid-cols-5 gap-6 mt-6">

          <!-- Carte : Emploi du temps du jour -->
          <section class="lg:col-span-3 bg-white/60 backdrop-blur-2xl rounded-3xl border border-white/40 shadow-lg shadow-indigo-500/5 p-6">
            <div class="flex items-center justify-between gap-3 flex-wrap">
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-xl bg-indigo-50 flex items-center justify-center">
                  <svg class="w-5 h-5 text-indigo-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                </div>
                <div>
                  <h3 class="font-semibold text-slate-900">Emploi du temps du jour</h3>
                  <p class="text-xs text-slate-400">Actualisation toutes les 60 s</p>
                </div>
              </div>
              <button
                @click="chargerCours()"
                :disabled="chargementCours"
                class="inline-flex items-center gap-2 px-3.5 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-60 text-white text-sm font-medium shadow-sm transition">
                <svg :class="{ 'animate-spin': chargementCours }" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
                {{ chargementCours ? 'Synchro…' : 'Rafraîchir' }}
              </button>
            </div>

            <p v-if="erreurCours" class="mt-5 rounded-xl bg-rose-50 border border-rose-200 text-rose-600 text-sm p-4">{{ erreurCours }}</p>
            <p v-if="horsLigne" class="mt-5 rounded-xl bg-amber-50 border border-amber-200 text-amber-700 text-sm p-4">Mode hors ligne : emploi du temps du dernier chargement réussi{{ majFluxLibelle ? ' (' + majFluxLibelle + ')' : '' }}.</p>

            <ul v-else-if="cours.length" class="mt-5 space-y-3">
              <li
                v-for="c in cours"
                :key="c.debut"
                class="flex items-start gap-4 rounded-xl border p-4 transition"
                :class="estEnCours(c) ? 'border-indigo-300 bg-indigo-50/70 shadow-sm' : 'border-slate-100 hover:border-indigo-200 hover:shadow-sm'">
                <div class="w-16 text-center shrink-0">
                  <p class="font-bold text-slate-900 tabular-nums">{{ c.debut }}</p>
                  <p class="text-slate-300 text-xs leading-none my-1">—</p>
                  <p class="text-sm text-slate-500 tabular-nums">{{ c.fin }}</p>
                </div>
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 flex-wrap">
                    <span v-if="estEnCours(c)" class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-bold text-white bg-emerald-500 animate-pulse">EN COURS</span>
                    <h3 class="font-semibold text-slate-800">{{ c.titre }}</h3>
                  </div>
                  <div class="mt-1 flex flex-wrap items-center gap-x-4 gap-y-1">
                    <p class="text-sm text-slate-500 flex items-center gap-1.5">
                      <svg class="w-3.5 h-3.5 text-slate-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
                        <path stroke-linecap="round" stroke-linejoin="round" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
                      </svg>
                      {{ c.salle || 'Salle non renseignée' }}
                    </p>
                    <p v-if="c.enseignant" class="text-sm text-slate-500 flex items-center gap-1.5">
                      <svg class="w-3.5 h-3.5 text-slate-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                        <circle cx="12" cy="7" r="4" />
                      </svg>
                      {{ c.enseignant }}
                    </p>
                  </div>
                </div>
              </li>
            </ul>

            <div v-else class="mt-6 rounded-2xl bg-white/40 backdrop-blur-xl border border-dashed border-white/60 p-10 text-center">
              <p class="font-medium text-slate-600">Aucun cours prévu aujourd'hui</p>
              <p class="text-sm text-slate-400 mt-1">Profites-en pour avancer sur tes devoirs ou préparer tes révisions.</p>
            </div>
          </section>

          <!-- Carte : Devoirs rapides -->
          <section class="lg:col-span-2 bg-white/60 backdrop-blur-2xl rounded-3xl border border-white/40 shadow-lg shadow-indigo-500/5 p-6">
            <div class="flex items-center justify-between gap-3">
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-xl bg-violet-50 flex items-center justify-center">
                  <svg class="w-5 h-5 text-violet-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
                  </svg>
                </div>
                <div>
                  <h3 class="font-semibold text-slate-900">Devoirs</h3>
                  <p class="text-xs text-slate-400">{{ devoirs.length }} au total</p>
                </div>
              </div>
              <button @click="ongletActif = 'devoirs'" class="text-sm font-medium text-indigo-600 hover:text-indigo-500">Voir le Kanban →</button>
            </div>

            <form @submit.prevent="ajouterDevoirRapide" class="mt-5 flex gap-2">
              <input
                v-model="nouveauDevoirRapide"
                required
                type="text"
                placeholder="Nouveau devoir…"
                class="flex-1 rounded-xl border border-white/50 bg-white/60 backdrop-blur px-3.5 py-2.5 text-sm placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent" />
              <button
                type="submit"
                class="inline-flex items-center justify-center w-11 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white shadow-sm transition">
                <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M12 4v16m8-8H4" />
                </svg>
              </button>
            </form>

            <p v-if="erreurDevoirs" class="mt-4 rounded-xl bg-rose-50 border border-rose-200 text-rose-600 text-sm p-3">{{ erreurDevoirs }}</p>

            <ul v-if="devoirs.length" class="mt-5 divide-y divide-slate-100">
              <li v-for="d in devoirs.slice(0, 6)" :key="d.id" class="flex items-center gap-3 py-3">
                <button
                  type="button"
                  @click="cycleStatut(d)"
                  :title="'Statut : ' + statutLabel(d.statut)"
                  class="w-3 h-3 rounded-full shrink-0 transition"
                  :class="statutDot(d.statut)"></button>
                <div class="flex-1 min-w-0">
                  <p class="truncate font-medium" :class="d.statut === 'termine' ? 'line-through text-slate-400' : 'text-slate-800'">{{ d.titre }}</p>
                  <div class="mt-1 flex flex-wrap items-center gap-1.5">
                    <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs" :class="typeBadge(d.type)">{{ typeLabel(d.type) }}</span>
                    <span v-if="d.matiere" class="text-xs text-slate-400">{{ d.matiere }}</span>
                    <span v-if="d.echeance" class="inline-flex items-center text-xs text-slate-400"><app-icon name="calendar" class="w-3.5 h-3.5 mr-1"></app-icon>{{ formaterEcheance(d.echeance) }}</span>
                    <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs" :class="statutBadge(d.statut)">{{ statutLabel(d.statut) }}</span>
                  </div>
                </div>
                <button
                  type="button"
                  @click="supprimerDevoir(d)"
                  title="Supprimer"
                  class="text-slate-300 hover:text-rose-500 transition">
                  <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </button>
              </li>
            </ul>

            <div v-else class="mt-6 rounded-2xl bg-white/40 backdrop-blur-xl border border-dashed border-white/60 p-8 text-center">
              <p class="font-medium text-slate-600">Aucun devoir pour l'instant</p>
              <p class="text-sm text-slate-400 mt-1">Ajoute ton premier devoir avec le formulaire ci-dessus.</p>
            </div>
          </section>

          <!-- Carte : Plan de révision IA -->
          <section class="lg:col-span-5 bg-gradient-to-br from-indigo-950 via-slate-900 to-indigo-900 rounded-3xl shadow-xl shadow-indigo-900/30 border border-indigo-500/20 p-6 md:p-8">
            <div class="flex items-start justify-between gap-4 flex-wrap">
              <div class="flex items-center gap-4">
                <div class="w-11 h-11 rounded-xl bg-gradient-to-br from-indigo-500 to-violet-500 flex items-center justify-center shadow-lg shadow-indigo-500/40">
                  <svg class="w-5 h-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z" />
                  </svg>
                </div>
                <div>
                  <h3 class="text-xl font-bold text-white">Plan de révision IA</h3>
                  <p class="text-sm text-indigo-200/70">Coach personnel basé sur tes cours et tes devoirs, propulsé par Gemini.</p>
                </div>
              </div>
              <button
                @click="genererPlanIA"
                :disabled="chargementPlan"
                class="inline-flex items-center gap-2 px-5 py-2.5 rounded-xl bg-gradient-to-r from-indigo-500 to-violet-600 hover:from-indigo-400 hover:to-violet-500 disabled:opacity-60 text-white text-sm font-semibold shadow-lg shadow-indigo-500/40 transition">
                <svg v-if="chargementPlan" class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                <svg v-else class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
                {{ chargementPlan ? "L'IA analyse ta journée…" : (plan ? 'Régénérer le plan' : 'Générer mon plan du soir') }}
              </button>
            </div>

            <div v-if="chargementPlan" class="mt-8 rounded-xl bg-white/5 border border-white/10 p-10 text-center">
              <svg class="animate-spin w-8 h-8 text-indigo-300 mx-auto" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              <p class="mt-4 text-slate-200 font-medium">L'IA analyse ton emploi du temps et tes devoirs…</p>
              <p class="mt-1 text-sm text-slate-400">Le plan de la soirée arrive dans quelques secondes.</p>
            </div>

            <div v-else-if="erreurPlan" class="mt-8 rounded-xl bg-rose-500/10 border border-rose-400/30 p-4 text-rose-200 text-sm">{{ erreurPlan }}</div>

            <div v-else-if="plan" class="mt-8 space-y-6">
              <div class="rounded-xl bg-white/5 border border-white/10 p-5">
                <p class="text-xs font-semibold uppercase tracking-wider text-indigo-300 mb-2">Conseil du soir</p>
                <p class="text-slate-100 leading-relaxed">{{ planContenu.conseil }}</p>
                <p v-if="plan && plan.avertissement" class="mt-3 text-xs text-amber-300">{{ plan.avertissement }}</p>
                <p v-if="plan && plan.mode === 'simulation' && !plan.avertissement" class="mt-3 text-xs text-indigo-200/70">Mode simulation : ajoute une clé Gemini dans Configuration pour un plan personnalisé par l'IA.</p>
              </div>

              <div v-if="planning.length" class="space-y-3">
                <div
                  v-for="(s, i) in planning"
                  :key="i"
                  class="rounded-xl bg-white/5 border border-white/10 p-5 flex flex-wrap items-center gap-4">
                  <span class="w-10 h-10 rounded-lg bg-indigo-500/20 text-indigo-200 font-semibold flex items-center justify-center text-sm tabular-nums">{{ i + 1 }}</span>
                  <div class="min-w-0 flex-1">
                    <h3 class="text-slate-50 font-semibold">{{ s.matiere }}</h3>
                    <p class="text-sm text-slate-300 mt-0.5">{{ s.action }}</p>
                  </div>
                  <span class="inline-flex items-center px-3 py-1.5 rounded-full bg-white/10 text-indigo-100 text-sm font-medium tabular-nums shrink-0">{{ s.heure }}</span>
                </div>
              </div>
              <p v-else class="text-sm text-slate-400">L'IA n'a pas proposé de créneau structuré cette fois-ci.</p>

              <div class="flex items-center justify-between gap-3 flex-wrap">
                <p class="text-sm text-indigo-200/70">Plan adapté à ton emploi du temps du jour</p>
                <button
                  @click="copierPlan"
                  class="inline-flex items-center gap-2 px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-white text-sm font-medium transition">
                  <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                  {{ planCopie ? 'Copié !' : 'Copier le plan' }}
                </button>
              </div>
            </div>

            <div v-else class="mt-8 rounded-xl bg-white/5 border border-dashed border-white/15 p-10 text-center">
              <p class="text-slate-300 font-medium">Aucun plan généré pour l'instant</p>
              <p class="text-sm text-slate-400 mt-1">L'IA analysera ton emploi du temps et tes devoirs pour proposer une soirée de révision optimale.</p>
            </div>
          </section>
        </div>
      </section>

      <!-- ============ ONGLET : SEMAINE ============ -->
      <section v-else-if="ongletActif === 'semaine'" class="max-w-7xl mx-auto">
        <header class="flex flex-wrap items-center justify-between gap-4">
          <div>
            <h2 class="text-2xl font-bold text-slate-900">Emploi du temps de la semaine</h2>
            <p class="text-sm text-slate-500">{{ semaineInfo.date_debut ? 'Semaine du ' + formaterCourte(semaineInfo.date_debut) + ' au ' + formaterCourte(semaineInfo.date_fin) : 'Choisis une semaine pour naviguer dans ton calendrier' }}</p>
          </div>

          <div class="flex items-center gap-2 flex-wrap">
            <button
              @click="changerSemaine(-7)"
              title="Semaine précédente"
              class="inline-flex items-center justify-center w-10 h-10 rounded-xl bg-white/60 backdrop-blur-xl border border-white/40 text-slate-500 hover:text-indigo-600 hover:border-indigo-200 shadow-sm transition">
              <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M15 19l-7-7 7-7" />
              </svg>
            </button>
            <input
              v-model="semaineDate"
              type="date"
              @change="chargerSemaine()"
              class="rounded-xl border border-white/50 bg-white/60 backdrop-blur px-3 py-2 text-sm text-slate-700 shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" />
            <button
              @click="changerSemaine(7)"
              title="Semaine suivante"
              class="inline-flex items-center justify-center w-10 h-10 rounded-xl bg-white/60 backdrop-blur-xl border border-white/40 text-slate-500 hover:text-indigo-600 hover:border-indigo-200 shadow-sm transition">
              <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7" />
              </svg>
            </button>
            <button
              @click="chargerSemaine()"
              :disabled="chargementSemaine"
              class="inline-flex items-center gap-2 px-3.5 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-60 text-white text-sm font-medium shadow-sm transition">
              <svg :class="{ 'animate-spin': chargementSemaine }" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              Rafraîchir
            </button>
          </div>
        </header>

        <p v-if="erreurSemaine" class="mt-6 rounded-xl bg-rose-50 border border-rose-200 text-rose-600 text-sm p-4">{{ erreurSemaine }}</p>
            <p v-if="horsLigne" class="mt-6 rounded-xl bg-amber-50 border border-amber-200 text-amber-700 text-sm p-4">Mode hors ligne : emploi du temps du dernier chargement réussi{{ majFluxLibelle ? ' (' + majFluxLibelle + ')' : '' }}.</p>

        <div v-else-if="chargementSemaine" class="mt-6 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
          <div v-for="i in 5" :key="i" class="rounded-3xl border border-white/40 bg-white/60 backdrop-blur-2xl shadow-lg shadow-indigo-500/5 p-5 h-64 animate-pulse"></div>
        </div>

        <div v-else-if="joursSemaine.length" class="mt-6 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
          <div
            v-for="jour in joursSemaine"
            :key="jour.date"
            class="rounded-3xl border p-5 bg-white/60 backdrop-blur-2xl shadow-lg shadow-indigo-500/5 flex flex-col"
            :class="jour.estAujourdhui ? 'border-indigo-300 ring-1 ring-indigo-200' : 'border-white/40'">
            <div class="flex items-center justify-between gap-2">
              <h3 class="font-semibold text-slate-900">{{ jour.libelle }}</h3>
              <span v-if="jour.estAujourdhui" class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-bold text-white bg-indigo-600">AUJOURD'HUI</span>
            </div>
            <div class="mt-3 space-y-2 flex-1">
              <div
                v-for="c in jour.cours"
                :key="c.titre + c.debut"
                class="rounded-xl border p-3"
                :class="jour.estAujourdhui ? 'border-indigo-100 bg-indigo-50/60' : 'border-slate-100 bg-slate-50/60'">
                <p class="text-xs font-semibold text-indigo-600 tabular-nums">{{ c.debut }} – {{ c.fin }}</p>
                <p class="font-medium text-slate-800 text-sm mt-1 leading-snug">{{ c.titre }}</p>
                <div class="mt-1 flex flex-wrap items-center gap-x-3 gap-y-0.5 text-xs text-slate-500">
                  <span class="inline-flex items-center gap-1">
                    <svg class="w-3 h-3 text-slate-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
                      <path stroke-linecap="round" stroke-linejoin="round" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                    {{ c.salle || 'Salle non renseignée' }}
                  </span>
                  <span v-if="c.enseignant" class="inline-flex items-center gap-1">
                    <svg class="w-3 h-3 text-slate-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                      <circle cx="12" cy="7" r="4" />
                    </svg>
                    {{ c.enseignant }}
                  </span>
                </div>
              </div>
            </div>
            <div v-if="!jour.cours.length" class="mt-3 rounded-xl border border-dashed border-white/60 p-4 text-center flex-1 flex items-center justify-center">
              <p class="text-xs text-slate-400">Pas de cours</p>
            </div>
          </div>
        </div>

        <div v-else class="mt-6 rounded-2xl bg-white/40 backdrop-blur-xl border border-dashed border-white/60 p-10 text-center">
          <p class="font-medium text-slate-600">Aucune donnée pour cette semaine</p>
        </div>
      </section>

      <!-- ============ ONGLET : DEVOIRS (KANBAN) ============ -->
      <section v-else-if="ongletActif === 'devoirs'" class="max-w-7xl mx-auto">
        <header class="flex flex-wrap items-center justify-between gap-4">
          <div>
            <h2 class="text-2xl font-bold text-slate-900">Devoirs</h2>
            <p class="text-sm text-slate-500">Tableau Kanban : glisse tes devoirs entre les statuts.</p>
          </div>
          <div class="flex flex-wrap items-center gap-2">
            <button
              type="button"
              @click="exporterDevoirs"
              title="Sauvegarder tous les devoirs au format JSON"
              class="inline-flex items-center gap-1.5 px-3.5 py-2 rounded-xl text-sm font-medium border transition bg-white/60 backdrop-blur text-slate-600 border-white/50 hover:border-indigo-300 hover:text-indigo-600">
              <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
              </svg>
              Exporter
            </button>
            <button
              type="button"
              @click="ouvrirImportDevoirs"
              title="Restaurer des devoirs depuis un fichier JSON"
              class="inline-flex items-center gap-1.5 px-3.5 py-2 rounded-xl text-sm font-medium border transition bg-white/60 backdrop-blur text-slate-600 border-white/50 hover:border-indigo-300 hover:text-indigo-600">
              <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4 4m0 0L9 8m4 4V4" />
              </svg>
              Importer
            </button>
          </div>
        </header>

        <!-- Formulaire d'ajout -->
        <form @submit.prevent="ajouterDevoir" class="mt-6 bg-white/60 backdrop-blur-2xl rounded-3xl border border-white/40 shadow-lg shadow-indigo-500/5 p-5 space-y-4">
          <div class="flex flex-wrap items-center gap-2">
            <span class="text-sm font-medium text-slate-500 mr-1">Type :</span>
            <button
              v-for="t in types"
              :key="t.id"
              type="button"
              @click="nouveauDevoir.type = t.id"
              class="inline-flex items-center gap-1.5 px-3.5 py-2 rounded-xl text-sm font-medium border transition"
              :class="nouveauDevoir.type === t.id ? 'bg-indigo-600 text-white border-indigo-600 shadow-sm' : 'bg-white/60 backdrop-blur text-slate-600 border-white/50 hover:border-indigo-300'">
              <app-icon :name="t.icone" class="w-4 h-4 shrink-0"></app-icon> {{ t.label }}
            </button>
          </div>
          <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
            <input
              v-model="nouveauDevoir.titre"
              required
              type="text"
              placeholder="Intitulé…"
              class="rounded-xl border border-white/50 bg-white/60 backdrop-blur px-3.5 py-2.5 text-sm placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent" />
            <input
              v-model="nouveauDevoir.matiere"
              type="text"
              placeholder="Matière"
              class="rounded-xl border border-white/50 bg-white/60 backdrop-blur px-3.5 py-2.5 text-sm placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent" />
            <input
              v-model="nouveauDevoir.echeance"
              type="date"
              class="rounded-xl border border-white/50 bg-white/60 backdrop-blur px-3.5 py-2.5 text-sm text-slate-600 focus:outline-none focus:ring-2 focus:ring-indigo-500" />
            <select
              v-model="nouveauDevoir.statut"
              class="rounded-xl border border-white/50 bg-white/60 backdrop-blur px-3.5 py-2.5 text-sm text-slate-700 focus:outline-none focus:ring-2 focus:ring-indigo-500">
              <option value="a_faire">À faire</option>
              <option value="en_cours">En cours</option>
              <option value="termine">Terminé</option>
            </select>
          </div>
          <button
            type="submit"
            class="inline-flex items-center justify-center gap-2 px-5 py-2.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-sm font-medium shadow-sm transition">
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M12 4v16m8-8H4" />
            </svg>
            Ajouter
          </button>
        </form>

        <p v-if="erreurDevoirs" class="mt-4 rounded-xl bg-rose-50 border border-rose-200 text-rose-600 text-sm p-4">{{ erreurDevoirs }}</p>

        <!-- Colonnes Kanban -->
        <div class="mt-6 grid grid-cols-1 md:grid-cols-3 gap-5">
          <div
            v-for="colonne in colonnes"
            :key="colonne.id"
            class="rounded-3xl border border-white/40 bg-white/60 backdrop-blur-2xl shadow-lg shadow-indigo-500/5 p-4 flex flex-col">
            <div class="flex items-center justify-between px-1">
              <span class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full text-sm font-semibold" :class="colonne.badge">
                <app-icon :name="colonne.icone" class="w-4 h-4"></app-icon> {{ colonne.label }}
              </span>
              <span class="text-sm font-bold text-slate-400 tabular-nums">{{ devoirsParStatut(colonne.id).length }}</span>
            </div>

            <div class="mt-4 space-y-3 flex-1">
              <div
                v-for="d in devoirsParStatut(colonne.id)"
                :key="d.id"
                class="rounded-xl border border-slate-100 bg-slate-50/60 p-4">
                <div class="flex items-start justify-between gap-2">
                  <div class="min-w-0">
                    <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs" :class="typeBadge(d.type)">{{ typeLabel(d.type) }}</span>
                    <p class="font-medium text-slate-800 leading-snug mt-1">{{ d.titre }}</p>
                    <div class="mt-1.5 flex flex-wrap items-center gap-1.5">
                      <span v-if="d.matiere" class="text-xs text-slate-500">{{ d.matiere }}</span>
                      <span
                        v-if="d.echeance"
                        class="inline-flex items-center px-2 py-0.5 rounded-full text-xs border"
                        :class="echeanceCouleur(d.echeance)">
                        {{ libelleEcheance(d.echeance) }}
                      </span>
                    </div>
                  </div>
                  <div class="flex items-center gap-1.5 shrink-0">
                    <button type="button" @click="ouvrirEdition(d)" title="Modifier" class="text-slate-300 hover:text-indigo-500 transition">
                      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M12 20h9M16.5 3.5a2.121 2.121 0 013 3L7 19l-4 1 1-4L16.5 3.5z" /></svg>
                    </button>
                    <button
                    type="button"
                    @click="supprimerDevoir(d)"
                    title="Supprimer"
                    class="text-slate-300 hover:text-rose-500 transition shrink-0">
                    <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                  </div>
                </div>
                <div class="mt-3 flex items-center justify-between gap-2">
                  <button
                    type="button"
                    @click="deplacer(d, -1)"
                    :disabled="d.statut === 'a_faire'"
                    title="Revenir en arrière"
                    class="inline-flex items-center justify-center w-8 h-8 rounded-lg bg-white/60 backdrop-blur border border-white/50 text-slate-500 hover:text-indigo-600 hover:border-indigo-200 disabled:opacity-30 disabled:cursor-not-allowed transition">
                    <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M15 19l-7-7 7-7" />
                    </svg>
                  </button>
                  <button
                    type="button"
                    @click="deplacer(d, 1)"
                    :disabled="d.statut === 'termine'"
                    title="Faire avancer"
                    class="inline-flex items-center justify-center w-8 h-8 rounded-lg bg-white/60 backdrop-blur border border-white/50 text-slate-500 hover:text-indigo-600 hover:border-indigo-200 disabled:opacity-30 disabled:cursor-not-allowed transition">
                    <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7" />
                    </svg>
                  </button>
                </div>
              </div>

              <div v-if="!devoirsParStatut(colonne.id).length" class="rounded-xl border border-dashed border-white/60 p-6 text-center">
                <p class="text-xs text-slate-400">Aucun devoir</p>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- ============ ONGLET : CONFIGURATION ============ -->
      <section v-else-if="ongletActif === 'config'" class="max-w-7xl mx-auto">
        <header>
          <h2 class="text-2xl font-bold text-slate-900">Configuration</h2>
          <p class="text-sm text-slate-500">Paramètres de l'emploi du temps ADE, de l'IA et de l'affichage.</p>
        </header>

        <form @submit.prevent="enregistrerConfig(false)" class="mt-8 bg-white/60 backdrop-blur-2xl rounded-3xl border border-white/40 shadow-lg shadow-indigo-500/5 p-6 space-y-5">
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1.5">Pseudo / Identifiant</label>
            <input
              v-model="config.username"
              type="text"
              placeholder="Ton pseudo GitHub ou ton nom"
              class="w-full rounded-xl border border-white/50 bg-white/60 backdrop-blur px-3.5 py-2.5 text-sm placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent" />
          </div>

          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1.5">Lien d'export iCal / ADE</label>
            <input
              v-model="config.ade_url"
              type="url"
              placeholder="https://ton-ecole.example.com/ade/ics?data=…"
              class="w-full rounded-xl border border-white/50 bg-white/60 backdrop-blur px-3.5 py-2.5 text-sm font-mono placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent" />
            <p class="text-xs text-slate-400 mt-1.5">Trouvé dans ton application ADE : Emploi du temps → Export / iCal.</p>
          </div>

          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1.5">Clé API Gemini</label>
            <input
              v-model="config.gemini_key"
              type="password"
              :placeholder="config.gemini_configured ? '•••••••• (clé enregistrée — laisse vide pour la conserver)' : 'AIza…'"
              autocomplete="off"
              class="w-full rounded-xl border border-white/50 bg-white/60 backdrop-blur px-3.5 py-2.5 text-sm font-mono placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent" />
            <p class="text-xs text-slate-400 mt-1.5">Gratuite sur <span class="font-medium">https://aistudio.google.com/apikey</span> — stockée dans le gestionnaire d'identifiants Windows, jamais dans un fichier.</p>
            <div v-if="config.gemini_configured" class="mt-2 flex items-center gap-3 text-xs">
              <span class="inline-flex items-center gap-1 text-emerald-600 font-medium"><app-icon name="check" class="w-3.5 h-3.5"></app-icon>Clé enregistrée</span>
              <button type="button" @click="enregistrerConfig(true)" class="text-rose-500 hover:text-rose-600 font-medium">Supprimer la clé</button>
            </div>
          </div>

          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1.5">Modèle Gemini</label>
            <input
              v-model="config.gemini_model"
              type="text"
              list="modeles-gemini"
              placeholder="gemini-3.8-flash"
              class="w-full rounded-xl border border-white/50 bg-white/60 backdrop-blur px-3.5 py-2.5 text-sm font-mono placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent" />
            <datalist id="modeles-gemini"><option v-for="m in modelesSuggeres" :key="m" :value="m"></option></datalist>
            <p class="text-xs text-slate-400 mt-1.5">Choisis un modèle proposé ou saisis le nom exact d'un modèle Gemini.</p>
          </div>

          <p v-if="configMessage" class="rounded-xl bg-emerald-50 border border-emerald-200 text-emerald-700 text-sm p-3.5">{{ configMessage }}</p>
          <p v-if="configErreur" class="rounded-xl bg-rose-50 border border-rose-200 text-rose-600 text-sm p-3.5">{{ configErreur }}</p>

          <button
            type="submit"
            :disabled="configEnregistrement"
            class="w-full inline-flex items-center justify-center gap-2 px-5 py-3 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-60 text-white text-sm font-semibold shadow-sm transition">
            <svg v-if="configEnregistrement" class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            {{ configEnregistrement ? 'Enregistrement…' : 'Enregistrer la configuration' }}
          </button>
        </form>

        <!-- ============ APPAREINCE & INTERFACE ============ -->
        <section class="mt-8 bg-white/60 backdrop-blur-2xl rounded-3xl border border-white/40 shadow-lg shadow-indigo-500/5 p-6">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-xl bg-violet-50 flex items-center justify-center">
              <svg class="w-5 h-5 text-violet-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M3 12h18M3 6h18M3 18h18M6 3v18M12 3v18M18 3v18" />
              </svg>
            </div>
            <div>
              <h3 class="font-semibold text-slate-900">🎨 Apparence & Interface</h3>
              <p class="text-xs text-slate-400">Choisis le style de navigation principal de ton tableau de bord.</p>
            </div>
          </div>

          <div class="mt-5">
            <label class="block text-sm font-medium text-slate-700 mb-1.5">Style de navigation</label>
            <select
              v-model="interfaceStyle"
              @change="sauvegarderInterface"
              class="w-full rounded-xl border border-white/50 bg-white/60 backdrop-blur px-3.5 py-2.5 text-sm text-slate-700 focus:outline-none focus:ring-2 focus:ring-indigo-500">
              <option value="dock">Modern Dock (Bandeau en bas)</option>
              <option value="classic">Classic Sidebar (Barre latérale)</option>
            </select>
            <p class="text-xs text-slate-400 mt-1.5">Le choix est enregistré automatiquement et appliqué immédiatement.</p>
          </div>

          <div class="mt-5">
            <label class="block text-sm font-medium text-slate-700 mb-1.5">Thème</label>
            <select
              v-model="theme"
              @change="changerPreferences"
              class="w-full rounded-xl border border-white/50 bg-white/60 backdrop-blur px-3.5 py-2.5 text-sm text-slate-700 focus:outline-none focus:ring-2 focus:ring-indigo-500">
              <option value="auto">Automatique (suit Windows)</option>
              <option value="light">Clair</option>
              <option value="dark">Sombre</option>
            </select>
          </div>
        </section>

        <!-- ============ DONNÉES LOCALES ============ -->
        <section class="mt-8 bg-white/60 backdrop-blur-2xl rounded-3xl border border-white/40 shadow-lg shadow-indigo-500/5 p-6">
          <h3 class="font-semibold text-slate-900">💾 Données locales</h3>
          <p class="text-xs text-slate-400 mt-1">Tes devoirs et réglages sont enregistrés sur ce PC (dossier AppData) et conservés entre les mises à jour. Une sauvegarde automatique est créée chaque jour.</p>
          <div class="mt-4 flex flex-wrap items-center gap-3">
            <button
              type="button"
              @click="migrerAncienneBase"
              :disabled="migrationEnCours"
              class="px-4 py-2.5 rounded-xl bg-white/70 border border-white/50 text-sm font-semibold text-slate-700 hover:border-indigo-300 disabled:opacity-60 transition">
              {{ migrationEnCours ? 'Import en cours…' : "Importer l'ancienne base (app.db)" }}
            </button>
            <span v-if="versionApp" class="text-xs text-slate-400">Version {{ versionApp }}</span>
          </div>
          <p v-if="migrationMessage" class="mt-4 rounded-xl bg-emerald-50 border border-emerald-200 text-emerald-700 text-sm p-3.5">{{ migrationMessage }}</p>
          <p v-if="migrationErreur" class="mt-4 rounded-xl bg-rose-50 border border-rose-200 text-rose-600 text-sm p-3.5">{{ migrationErreur }}</p>
        </section>

        <!-- ============ FILTRES & PERSONNALISATION ============ -->
        <section class="mt-8 bg-white/60 backdrop-blur-2xl rounded-3xl border border-white/40 shadow-lg shadow-indigo-500/5 p-6">
          <div class="flex flex-wrap items-start justify-between gap-4">
            <div class="flex items-center gap-3">
              <div class="w-10 h-10 rounded-xl bg-amber-50 flex items-center justify-center">
                <app-icon name="filter" class="w-5 h-5 text-amber-600"></app-icon>
              </div>
              <div>
                <h3 class="font-semibold text-slate-900">Filtres & Personnalisation</h3>
                <p class="text-xs text-slate-400">Analyse automatique de ton flux ADE : coche tes groupes (TP/TD/Khôlles), tes épreuves et tes matières.</p>
              </div>
            </div>
            <button
              @click="analyserCategories"
              :disabled="chargementCategories"
              class="inline-flex items-center gap-2 px-4 py-2.5 rounded-xl bg-gradient-to-r from-amber-500 to-orange-500 hover:from-amber-400 hover:to-orange-400 disabled:opacity-60 text-white text-sm font-semibold shadow-sm transition">
              <svg v-if="chargementCategories" class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              <app-icon v-else name="search" class="w-4 h-4"></app-icon>
              {{ chargementCategories ? 'Analyse en cours…' : 'Analyser mon lien ADE' }}
            </button>
          </div>

          <p v-if="erreurCategories" class="mt-5 rounded-xl bg-rose-50 border border-rose-200 text-rose-600 text-sm p-4">{{ erreurCategories }}</p>
          <p v-if="messageCategories" class="mt-5 rounded-xl bg-emerald-50 border border-emerald-200 text-emerald-700 text-sm p-4">{{ messageCategories }}</p>

          <template v-if="categorisation.groupes.length">
            <div class="mt-6 border-t border-slate-100 pt-5">
              <div class="flex flex-wrap items-center justify-between gap-3">
                <h4 class="font-semibold text-slate-800 text-sm">Groupes détectés</h4>
                <div class="flex items-center gap-2 text-xs">
                  <button
                    type="button"
                    @click="toutCocher('groupes')"
                    class="px-3 py-1.5 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium transition">Tout cocher</button>
                  <button
                    type="button"
                    @click="toutDecocher('groupes')"
                    class="px-3 py-1.5 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium transition">Tout décocher</button>
                  <span class="text-slate-400 tabular-nums">{{ filtresGroupes.length }}/{{ categorisation.groupes.length }} actifs</span>
                </div>
              </div>
              <div class="mt-3 flex flex-wrap gap-2">
                <button
                  v-for="g in categorisation.groupes"
                  :key="g"
                  type="button"
                  @click="basculer('groupes', g)"
                  class="inline-flex items-center gap-1.5 px-3.5 py-2 rounded-full text-sm font-medium border transition"
                  :class="estCoche('groupes', g) ? 'bg-indigo-600 text-white border-indigo-600 shadow-sm' : 'bg-white/60 backdrop-blur text-slate-600 border-white/50 hover:border-indigo-300'">
                  <app-icon v-if="estCoche('groupes', g)" name="check" class="w-3.5 h-3.5"></app-icon>
                  {{ g }}
                </button>
              </div>
              <p class="text-xs text-slate-400 mt-3">Les cours sans groupe (CM, réunions…) restent toujours visibles. Les TP/TD/Khôlles des groupes non cochés sont masqués du Dashboard et de la Semaine.</p>
            </div>

            <div class="mt-6 grid grid-cols-1 sm:grid-cols-2 gap-5 border-t border-slate-100 pt-5">
              <div>
                <div class="flex flex-wrap items-center justify-between gap-3 mb-2">
                  <h4 class="font-semibold text-slate-800 text-sm">Types d'épreuves</h4>
                  <div class="flex items-center gap-2 text-xs">
                    <button
                      type="button"
                      @click="toutCocher('types')"
                      class="px-2 py-1 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium transition">Tout</button>
                    <button
                      type="button"
                      @click="toutDecocher('types')"
                      class="px-2 py-1 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium transition">Aucun</button>
                    <span class="text-slate-400 tabular-nums">{{ filtresTypes.length }}/{{ categorisation.types.length }}</span>
                  </div>
                </div>
                <div class="flex flex-wrap gap-1.5">
                  <button
                    v-for="t in categorisation.types"
                    :key="t"
                    type="button"
                    @click="basculer('types', t)"
                    class="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-medium border transition"
                    :class="estCoche('types', t) ? 'bg-sky-600 text-white border-sky-600' : 'bg-sky-50 text-sky-700 border-sky-100 hover:border-sky-300'">
                    <app-icon v-if="estCoche('types', t)" name="check" class="w-3 h-3"></app-icon>
                    {{ t }}
                  </button>
                  <span v-if="!categorisation.types.length" class="text-xs text-slate-400">Aucun type spécifique détecté.</span>
                </div>
                <p class="text-xs text-slate-400 mt-2">Coche les épreuves à garder (TP, TD, DS…). Les séances d'un type non coché sont masquées.</p>
              </div>
              <div>
                <div class="flex flex-wrap items-center justify-between gap-3 mb-2">
                  <h4 class="font-semibold text-slate-800 text-sm">Matières</h4>
                  <div class="flex items-center gap-2 text-xs">
                    <button
                      type="button"
                      @click="toutCocher('matieres')"
                      class="px-2 py-1 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium transition">Tout</button>
                    <button
                      type="button"
                      @click="toutDecocher('matieres')"
                      class="px-2 py-1 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium transition">Aucun</button>
                    <span class="text-slate-400 tabular-nums">{{ filtresMatieres.length }}/{{ categorisation.matieres.length }}</span>
                  </div>
                </div>
                <div class="flex flex-wrap gap-1.5">
                  <button
                    v-for="m in categorisation.matieres"
                    :key="m"
                    type="button"
                    @click="basculer('matieres', m)"
                    class="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-medium border transition"
                    :class="estCoche('matieres', m) ? 'bg-violet-600 text-white border-violet-600' : 'bg-violet-50 text-violet-700 border-violet-100 hover:border-violet-300'">
                    <app-icon v-if="estCoche('matieres', m)" name="check" class="w-3 h-3"></app-icon>
                    {{ m }}
                  </button>
                  <span v-if="!categorisation.matieres.length" class="text-xs text-slate-400">Aucune matière détectée.</span>
                </div>
                <p class="text-xs text-slate-400 mt-2">Coche les matières à garder. Les cours des matières non cochées sont masqués.</p>
              </div>
            </div>
          </template>

          <div v-else-if="!chargementCategories" class="mt-6 rounded-2xl bg-white/40 backdrop-blur-xl border border-dashed border-white/60 p-8 text-center">
            <p class="text-sm text-slate-500">Clique sur « Analyser mon lien ADE » pour détecter automatiquement tes groupes (TP/TD/Khôlles), les types d'épreuves et les matières.</p>
          </div>
        </section>
      </section>

      <footer class="max-w-7xl mx-auto mt-14">
        <p class="text-xs text-slate-400 text-center">Emploi du temps ADE · Devoirs · Génération IA à la demande · Application locale hors ligne</p>
      </footer>
    </main>
    
    <!-- ==================== ONBOARDING (première configuration) ==================== -->
    <div v-if="onboardingVisible" class="fixed inset-0 z-[60] flex items-center justify-center p-4">
      <div class="absolute inset-0 bg-slate-900/40 backdrop-blur-sm" @click.self="ignorerOnboarding"></div>
      <div class="relative w-full max-w-lg bg-white/90 backdrop-blur-2xl rounded-3xl border border-white/50 shadow-2xl p-6 sm:p-8">
        <div class="flex items-center gap-3">
          <div class="w-11 h-11 rounded-2xl bg-gradient-to-br from-indigo-600 to-violet-600 flex items-center justify-center shadow-lg shadow-indigo-500/30 shrink-0">
            <svg class="w-6 h-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          </div>
          <div>
            <h3 class="text-lg font-bold text-slate-900">Bienvenue, {{ config.username || 'étudiant' }} !</h3>
            <p class="text-sm text-slate-500">Terminons la configuration en 30 secondes.</p>
          </div>
        </div>

        <form @submit.prevent="enregistrerOnboarding" class="mt-6 space-y-4">
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1.5">Lien d'export iCal / ADE <span class="text-rose-500">*</span></label>
            <input
              v-model="onboarding.ade_url"
              type="url"
              required
              placeholder="https://ton-ecole.example.com/ade/ics?data=…"
              class="w-full rounded-xl border border-white/50 bg-white/60 backdrop-blur px-3.5 py-2.5 text-sm font-mono placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent" />
            <p class="text-xs text-slate-400 mt-1.5">Indispensable pour afficher ton emploi du temps. Trouvé dans ton application ADE : Emploi du temps → Export / iCal.</p>
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1.5">Clé API Gemini <span class="text-slate-400">(optionnel)</span></label>
            <input
              v-model="onboarding.gemini_key"
              type="password"
              placeholder="AIza…"
              autocomplete="off"
              class="w-full rounded-xl border border-white/50 bg-white/60 backdrop-blur px-3.5 py-2.5 text-sm font-mono placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent" />
            <p class="text-xs text-slate-400 mt-1.5">Gratuite sur <span class="font-medium">https://aistudio.google.com/apikey</span>. Sans clé, le plan de révision fonctionne en mode simulation.</p>
          </div>

          <p v-if="onboardingErreur" class="rounded-xl bg-rose-50 border border-rose-200 text-rose-600 text-sm p-3.5">{{ onboardingErreur }}</p>

          <div class="flex flex-wrap items-center gap-3">
            <button
              type="submit"
              :disabled="onboardingEnregistrement"
              class="flex-1 inline-flex items-center justify-center gap-2 px-5 py-3 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-60 text-white text-sm font-semibold shadow-sm transition">
              {{ onboardingEnregistrement ? 'Enregistrement…' : 'Commencer' }}
            </button>
            <button
              type="button"
              @click="ignorerOnboarding"
              class="px-4 py-3 rounded-xl text-sm font-medium text-slate-500 hover:text-slate-700 transition">Plus tard</button>
          </div>
        </form>
      </div>
    </div>
  
    <!-- ==================== NOTIFICATION ==================== -->
    <div v-if="toast" class="fixed top-5 right-5 z-[70] max-w-sm rounded-2xl bg-slate-900/90 text-white text-sm px-4 py-3 shadow-xl backdrop-blur">{{ toast }}</div>

    <!-- ==================== MODIFICATION D'UN DEVOIR ==================== -->
    <div v-if="edition" class="fixed inset-0 z-[60] flex items-center justify-center p-4">
      <div class="absolute inset-0 bg-slate-900/40 backdrop-blur-sm" @click.self="fermerEdition"></div>
      <div class="relative w-full max-w-lg bg-white/90 backdrop-blur-2xl rounded-3xl border border-white/50 shadow-2xl p-6 sm:p-8">
        <h3 class="text-lg font-bold text-slate-900">Modifier le devoir</h3>
        <form @submit.prevent="enregistrerEdition" class="mt-5 space-y-4">
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1.5">Titre</label>
            <input v-model="edition.titre" type="text" required maxlength="200" class="w-full rounded-xl border border-white/50 bg-white/60 px-3.5 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" />
          </div>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-slate-700 mb-1.5">Matière</label>
              <input v-model="edition.matiere" type="text" maxlength="120" class="w-full rounded-xl border border-white/50 bg-white/60 px-3.5 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" />
            </div>
            <div>
              <label class="block text-sm font-medium text-slate-700 mb-1.5">Échéance</label>
              <input v-model="edition.echeance" type="date" class="w-full rounded-xl border border-white/50 bg-white/60 px-3.5 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" />
            </div>
            <div>
              <label class="block text-sm font-medium text-slate-700 mb-1.5">Type</label>
              <select v-model="edition.type" class="w-full rounded-xl border border-white/50 bg-white/60 px-3.5 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500">
                <option v-for="t in types" :key="t.id" :value="t.id">{{ t.label }}</option>
              </select>
            </div>
            <div>
              <label class="block text-sm font-medium text-slate-700 mb-1.5">Statut</label>
              <select v-model="edition.statut" class="w-full rounded-xl border border-white/50 bg-white/60 px-3.5 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500">
                <option v-for="c in colonnes" :key="c.id" :value="c.id">{{ c.label }}</option>
              </select>
            </div>
          </div>
          <p v-if="editionErreur" class="rounded-xl bg-rose-50 border border-rose-200 text-rose-600 text-sm p-3.5">{{ editionErreur }}</p>
          <div class="flex items-center gap-3">
            <button type="submit" :disabled="editionEnregistrement" class="flex-1 px-5 py-3 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-60 text-white text-sm font-semibold shadow-sm transition">{{ editionEnregistrement ? 'Enregistrement…' : 'Enregistrer' }}</button>
            <button type="button" @click="fermerEdition" class="px-4 py-3 rounded-xl text-sm font-medium text-slate-500 hover:text-slate-700 transition">Annuler</button>
          </div>
        </form>
      </div>
    </div>
</div>
</template>

<script>
import { appeler } from './api';
import AppIcon from './AppIcon.vue';

const MODELES_SUGGERES = ['gemini-3.8-flash', 'gemini-2.5-flash', 'gemini-2.5-flash-lite', 'gemini-2.5-pro'];
const isoLocal = (d) => `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
const DEVOIR_VIDE = () => ({ titre: '', matiere: '', echeance: '', type: 'devoir', statut: 'a_faire' });

export default {
  name: 'App',
  components: { AppIcon },
  data() {
    return {
      ongletActif: 'dashboard',
      interfaceStyle: 'dock',
      theme: 'auto',
      modelesSuggeres: MODELES_SUGGERES,
      onglets: [
        { id: 'dashboard', label: 'Dashboard', icone: 'dashboard' },
        { id: 'semaine', label: 'Semaine', icone: 'calendar' },
        { id: 'devoirs', label: 'Kanban', icone: 'list' },
        { id: 'config', label: 'Configuration', icone: 'gear' },
      ],
      colonnes: [
        { id: 'a_faire', label: 'À faire', icone: 'dot-rose', badge: 'bg-rose-100 text-rose-700' },
        { id: 'en_cours', label: 'En cours', icone: 'dot-amber', badge: 'bg-amber-100 text-amber-700' },
        { id: 'termine', label: 'Terminé', icone: 'dot-emerald', badge: 'bg-emerald-100 text-emerald-700' },
      ],
      types: [
        { id: 'devoir', label: 'Devoir', icone: 'devoir' },
        { id: 'ie', label: 'IE', icone: 'ie' },
        { id: 'ds', label: 'DS', icone: 'ds' },
        { id: 'exam', label: 'Examen', icone: 'exam' },
      ],
      dateDuJour: new Date().toLocaleDateString('fr-FR', { weekday: 'long', day: 'numeric', month: 'long', year: 'numeric' }),
      heureActuelle: '',
      toast: '',
      // Emploi du temps
      cours: [],
      chargementCours: false,
      erreurCours: null,
      coursSemaine: [],
      chargementSemaine: false,
      erreurSemaine: null,
      semaineDate: '',
      semaineInfo: { date_debut: '', date_fin: '' },
      horsLigne: false,
      majFlux: '',
      // Devoirs
      devoirs: [],
      nouveauDevoir: DEVOIR_VIDE(),
      nouveauDevoirRapide: '',
      erreurDevoirs: null,
      edition: null,
      editionId: null,
      editionEnregistrement: false,
      editionErreur: '',
      // Plan IA
      plan: null,
      chargementPlan: false,
      erreurPlan: null,
      planCopie: false,
      // Configuration
      config: { username: '', ade_url: '', gemini_key: '', gemini_model: '', gemini_configured: false },
      chargementConfig: false,
      configEnregistrement: false,
      configMessage: '',
      configErreur: '',
      migrationEnCours: false,
      migrationMessage: '',
      migrationErreur: '',
      versionApp: '',
      // Filtres & personnalisation
      categorisation: { groupes: [], types: [], matieres: [] },
      chargementCategories: false,
      erreurCategories: '',
      messageCategories: '',
      filtresGroupes: [],
      filtresTypes: [],
      filtresMatieres: [],
      // Onboarding (première configuration)
      onboardingVisible: false,
      onboardingIgnore: false,
      onboarding: { ade_url: '', gemini_key: '' },
      onboardingEnregistrement: false,
      onboardingErreur: '',
    };
  },

  computed: {
    identifiantAffichable() {
      return this.config.username || 'Étudiant';
    },
    themeSombre() {
      return this.theme === 'dark' || (this.theme === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches);
    },
    planContenu() {
      const p = this.plan;
      if (!p) return null;
      return p.plan && typeof p.plan === 'object' ? p.plan : p;
    },
    planning() {
      const pc = this.planContenu;
      return pc && Array.isArray(pc.planning) ? pc.planning : [];
    },
    joursSemaine() {
      const parDate = {};
      for (const c of this.coursSemaine) {
        if (!parDate[c.date]) parDate[c.date] = [];
        parDate[c.date].push(c);
      }
      return Object.keys(parDate).sort().map((dt) => ({
        date: dt,
        libelle: this.libelleJour(dt),
        estAujourdhui: dt === this.dateDuJourIso,
        cours: parDate[dt].sort((a, b) => a.debut.localeCompare(b.debut)),
      }));
    },
    dateDuJourIso() {
      return isoLocal(new Date());
    },
    majFluxLibelle() {
      if (!this.majFlux) return '';
      const d = new Date(this.majFlux);
      return Number.isNaN(d.getTime()) ? '' : d.toLocaleString('fr-FR', { day: 'numeric', month: 'long', hour: '2-digit', minute: '2-digit' });
    },
  },

  methods: {
    notifier(message) {
      this.toast = message;
      clearTimeout(this._toastTimer);
      this._toastTimer = setTimeout(() => { this.toast = ''; }, 4000);
    },
    messageErreur(e, defaut) {
      return (e && e.message) || defaut;
    },
    initialiserDonnees() {
      this.chargerConfig().then(() => this.verifierOnboarding());
      this.chargerCours();
      this.chargerSemaine();
      this.chargerDevoirs();
    },

    // ---- Onboarding (première configuration) ----
    verifierOnboarding() {
      if (!this.onboardingIgnore && !this.config.ade_url) {
        this.onboarding = { ade_url: '', gemini_key: '' };
        this.onboardingErreur = '';
        this.onboardingVisible = true;
      }
    },
    async enregistrerOnboarding() {
      this.onboardingEnregistrement = true;
      this.onboardingErreur = '';
      try {
        if (!this.onboarding.ade_url.trim()) throw new Error('Renseigne ton lien ADE pour continuer.');
        this.config = {
          ...(await appeler('enregistrer_config', {
            maj: {
              username: this.config.username || '',
              ade_url: this.onboarding.ade_url.trim(),
              gemini_key: this.onboarding.gemini_key.trim(),
              gemini_model: this.config.gemini_model || null,
            },
          })),
          gemini_key: '',
        };
        this.onboardingVisible = false;
        this.chargerCours();
        this.chargerSemaine();
      } catch (e) {
        this.onboardingErreur = this.messageErreur(e, "Impossible d'enregistrer la configuration.");
      } finally {
        this.onboardingEnregistrement = false;
      }
    },
    ignorerOnboarding() {
      this.onboardingIgnore = true;
      this.onboardingVisible = false;
    },

    // ---- Emploi du temps ----
    filtres() {
      return { groupes: this.filtresGroupes, types: this.filtresTypes, matieres: this.filtresMatieres };
    },
    appliquerEtatFlux(data) {
      this.horsLigne = Boolean(data.hors_ligne);
      this.majFlux = data.maj || '';
    },
    async chargerCours(silencieux = false) {
      if (!silencieux) this.chargementCours = true;
      this.erreurCours = null;
      try {
        const data = await appeler('cours_du_jour', { filtres: this.filtres() });
        this.cours = data.cours || [];
        this.appliquerEtatFlux(data);
      } catch (e) {
        this.erreurCours = this.messageErreur(e, "Impossible de synchroniser l'emploi du temps.");
      } finally {
        if (!silencieux) this.chargementCours = false;
      }
    },
    async chargerSemaine() {
      this.chargementSemaine = true;
      this.erreurSemaine = null;
      try {
        const data = await appeler('cours_semaine', { date: this.semaineDate || null, filtres: this.filtres() });
        this.coursSemaine = data.cours || [];
        this.semaineInfo = { date_debut: data.date_debut || '', date_fin: data.date_fin || '' };
        this.appliquerEtatFlux(data);
      } catch (e) {
        this.erreurSemaine = this.messageErreur(e, "Impossible de charger l'emploi du temps de la semaine.");
      } finally {
        this.chargementSemaine = false;
      }
    },
    changerSemaine(ecartJours) {
      if (!this.semaineDate) return;
      const d = new Date(this.semaineDate + 'T00:00:00');
      d.setDate(d.getDate() + ecartJours);
      this.semaineDate = isoLocal(d);
      this.chargerSemaine();
    },
    estEnCours(c) {
      if (!this.heureActuelle) return false;
      return this.heureActuelle >= c.debut && this.heureActuelle < c.fin;
    },
    libelleJour(dt) {
      const d = new Date(dt + 'T00:00:00');
      const s = d.toLocaleDateString('fr-FR', { weekday: 'long', day: 'numeric', month: 'short' });
      return s.charAt(0).toUpperCase() + s.slice(1);
    },
    formaterCourte(dt) {
      return new Date(dt + 'T00:00:00').toLocaleDateString('fr-FR', { day: 'numeric', month: 'long' });
    },

    // ---- Devoirs ----
    async chargerDevoirs() {
      this.erreurDevoirs = null;
      try {
        this.devoirs = await appeler('lister_devoirs');
      } catch (e) {
        this.erreurDevoirs = this.messageErreur(e, 'Impossible de charger les devoirs.');
      }
    },
    async ajouterDevoir() {
      const titre = this.nouveauDevoir.titre.trim();
      if (!titre) return;
      try {
        await appeler('creer_devoir', {
          devoir: {
            titre,
            matiere: this.nouveauDevoir.matiere.trim(),
            echeance: this.nouveauDevoir.echeance || '',
            type: this.nouveauDevoir.type,
            statut: this.nouveauDevoir.statut,
          },
        });
        this.nouveauDevoir = DEVOIR_VIDE();
        await this.chargerDevoirs();
      } catch (e) {
        this.erreurDevoirs = this.messageErreur(e, "Impossible d'ajouter le devoir.");
      }
    },
    async ajouterDevoirRapide() {
      const titre = this.nouveauDevoirRapide.trim();
      if (!titre) return;
      try {
        await appeler('creer_devoir', { devoir: { titre, statut: 'a_faire' } });
        this.nouveauDevoirRapide = '';
        await this.chargerDevoirs();
      } catch (e) {
        this.erreurDevoirs = this.messageErreur(e, "Impossible d'ajouter le devoir.");
      }
    },
    async changerStatut(d, statut) {
      try {
        await appeler('modifier_devoir', { id: d.id, modification: { statut } });
        d.statut = statut;
      } catch (e) {
        this.erreurDevoirs = this.messageErreur(e, 'Impossible de mettre à jour le statut du devoir.');
      }
    },
    cycleStatut(d) {
      const ordre = ['a_faire', 'en_cours', 'termine'];
      this.changerStatut(d, ordre[(ordre.indexOf(d.statut) + 1) % ordre.length]);
    },
    deplacer(d, direction) {
      const ordre = ['a_faire', 'en_cours', 'termine'];
      const suivant = ordre[ordre.indexOf(d.statut) + direction];
      if (suivant) this.changerStatut(d, suivant);
    },
    ouvrirEdition(d) {
      this.editionId = d.id;
      this.edition = { titre: d.titre, matiere: d.matiere, echeance: d.echeance, type: d.type, statut: d.statut };
      this.editionErreur = '';
    },
    fermerEdition() {
      this.edition = null;
      this.editionId = null;
    },
    async enregistrerEdition() {
      this.editionEnregistrement = true;
      this.editionErreur = '';
      try {
        await appeler('modifier_devoir', {
          id: this.editionId,
          modification: {
            titre: this.edition.titre,
            matiere: this.edition.matiere,
            echeance: this.edition.echeance || '',
            type: this.edition.type,
            statut: this.edition.statut,
          },
        });
        this.fermerEdition();
        await this.chargerDevoirs();
      } catch (e) {
        this.editionErreur = this.messageErreur(e, 'Impossible de modifier le devoir.');
      } finally {
        this.editionEnregistrement = false;
      }
    },
    async supprimerDevoir(d) {
      try {
        await appeler('supprimer_devoir', { id: d.id });
        await this.chargerDevoirs();
      } catch (e) {
        this.erreurDevoirs = this.messageErreur(e, 'Impossible de supprimer le devoir.');
      }
    },
    async exporterDevoirs() {
      try {
        const chemin = await appeler('exporter_devoirs');
        if (chemin) this.notifier('Devoirs exportés dans ' + chemin);
      } catch (e) {
        this.erreurDevoirs = this.messageErreur(e, "Impossible d'exporter les devoirs.");
      }
    },
    async ouvrirImportDevoirs() {
      try {
        const nb = await appeler('importer_devoirs');
        if (nb !== null && nb !== undefined) {
          this.notifier(`${nb} devoir(s) importé(s).`);
          await this.chargerDevoirs();
        }
      } catch (e) {
        this.erreurDevoirs = this.messageErreur(e, "Impossible d'importer les devoirs : fichier JSON invalide.");
      }
    },
    devoirsParStatut(statut) {
      return this.devoirs.filter((d) => d.statut === statut);
    },
    nbStatut(statut) {
      return this.devoirs.filter((d) => d.statut === statut).length;
    },
    statutLabel(s) {
      return { a_faire: 'À faire', en_cours: 'En cours', termine: 'Terminé' }[s] || s;
    },
    statutDot(s) {
      return { a_faire: 'bg-rose-500', en_cours: 'bg-amber-400', termine: 'bg-emerald-500' }[s] || 'bg-slate-400';
    },
    statutBadge(s) {
      return {
        a_faire: 'bg-rose-100 text-rose-700',
        en_cours: 'bg-amber-100 text-amber-700',
        termine: 'bg-emerald-100 text-emerald-700',
      }[s] || 'bg-slate-100 text-slate-600';
    },
    typeLabel(t) {
      return { devoir: 'Devoir', ie: 'IE', ds: 'DS', exam: 'Examen' }[t] || t;
    },
    typeBadge(t) {
      return {
        devoir: 'bg-slate-100 text-slate-600',
        ie: 'bg-sky-100 text-sky-700',
        ds: 'bg-orange-100 text-orange-700',
        exam: 'bg-rose-100 text-rose-700',
      }[t] || 'bg-slate-100 text-slate-600';
    },
    formaterEcheance(e) {
      return new Date(e + 'T00:00:00').toLocaleDateString('fr-FR', { day: 'numeric', month: 'short' });
    },
    joursRestants(e) {
      const echeance = new Date(e + 'T00:00:00');
      const aujourd = new Date();
      aujourd.setHours(0, 0, 0, 0);
      return Math.round((echeance - aujourd) / 86400000);
    },
    libelleEcheance(e) {
      const diff = this.joursRestants(e);
      if (diff < 0) return 'En retard';
      if (diff === 0) return "Aujourd'hui";
      if (diff === 1) return 'Demain';
      return `J-${diff}`;
    },
    echeanceCouleur(e) {
      const diff = this.joursRestants(e);
      if (diff <= 0) return 'bg-rose-50 text-rose-600 border-rose-200';
      if (diff <= 3) return 'bg-amber-50 text-amber-600 border-amber-200';
      if (diff <= 7) return 'bg-sky-50 text-sky-600 border-sky-200';
      return 'bg-slate-50 text-slate-500 border-slate-200';
    },

    // ---- Plan IA ----
    async genererPlanIA() {
      this.chargementPlan = true;
      this.erreurPlan = null;
      try {
        this.plan = await appeler('generer_plan', { filtres: this.filtres() });
      } catch (e) {
        this.erreurPlan = this.messageErreur(e, 'Impossible de générer le plan de révision.');
      } finally {
        this.chargementPlan = false;
      }
    },
    async copierPlan() {
      const conseil = this.planContenu.conseil || '';
      const planningTxt = this.planning.map((s) => `- ${s.heure} · ${s.matiere} : ${s.action}`).join('\n');
      try {
        await navigator.clipboard.writeText(`Plan de révision du soir\n\n${conseil}\n\n${planningTxt}`);
        this.planCopie = true;
        setTimeout(() => { this.planCopie = false; }, 2000);
      } catch (_) { /* presse-papiers indisponible */ }
    },

    // ---- Configuration ----
    async chargerConfig() {
      this.chargementConfig = true;
      this.configErreur = '';
      try {
        this.config = { ...(await appeler('obtenir_config')), gemini_key: '' };
      } catch (e) {
        this.configErreur = this.messageErreur(e, 'Impossible de charger la configuration.');
      } finally {
        this.chargementConfig = false;
      }
    },
    async enregistrerConfig(retirerCle = false) {
      this.configEnregistrement = true;
      this.configMessage = '';
      this.configErreur = '';
      try {
        this.config = {
          ...(await appeler('enregistrer_config', {
            maj: {
              username: this.config.username,
              ade_url: this.config.ade_url,
              gemini_key: this.config.gemini_key || null,
              gemini_model: this.config.gemini_model || null,
              remove_gemini_key: retirerCle === true,
            },
          })),
          gemini_key: '',
        };
        this.configMessage = retirerCle === true ? 'Clé Gemini supprimée.' : 'Configuration enregistrée avec succès.';
        this.chargerCours();
        this.chargerSemaine();
      } catch (e) {
        this.configErreur = this.messageErreur(e, 'Impossible d’enregistrer la configuration.');
      } finally {
        this.configEnregistrement = false;
      }
    },
    async migrerAncienneBase() {
      this.migrationEnCours = true;
      this.migrationMessage = '';
      this.migrationErreur = '';
      try {
        const r = await appeler('migrer_ancienne_base');
        if (r) {
          this.migrationMessage = `${r.devoirs_importes} devoir(s) importé(s), ${r.devoirs_ignores} ignoré(s) (doublons ou invalides)`
            + (r.parametres_importes ? ', paramètres repris' : '')
            + (r.cle_gemini_trouvee ? ', clé Gemini transférée dans le gestionnaire d’identifiants Windows.' : '.');
          await this.chargerDevoirs();
          await this.chargerConfig();
        }
      } catch (e) {
        this.migrationErreur = this.messageErreur(e, "Impossible d'importer l'ancienne base.");
      } finally {
        this.migrationEnCours = false;
      }
    },

    // ---- Préférences d'affichage (enregistrées dans SQLite) ----
    appliquerTheme() {
      document.documentElement.classList.toggle('dark', this.themeSombre);
    },
    basculerTheme() {
      this.theme = this.themeSombre ? 'light' : 'dark';
      this.changerPreferences();
    },
    async chargerPreferences() {
      try {
        const p = await appeler('obtenir_preferences');
        if (p.interface === 'dock' || p.interface === 'classic') this.interfaceStyle = p.interface;
        if (['auto', 'light', 'dark'].includes(p.theme)) this.theme = p.theme;
        const f = p.filtres || {};
        this.filtresGroupes = Array.isArray(f.groupes) ? f.groupes.filter(Boolean) : [];
        this.filtresTypes = Array.isArray(f.types) ? f.types.filter(Boolean) : [];
        this.filtresMatieres = Array.isArray(f.matieres) ? f.matieres.filter(Boolean) : [];
      } catch (_) { /* valeurs par défaut */ }
      this.appliquerTheme();
    },
    async changerPreferences() {
      this.appliquerTheme();
      try {
        await appeler('enregistrer_preferences', {
          preferences: { interface: this.interfaceStyle, theme: this.theme, filtres: this.filtres() },
        });
      } catch (_) { /* non bloquant */ }
    },
    sauvegarderInterface() {
      this.changerPreferences();
    },
    sauvegarderFiltres() {
      this.changerPreferences();
    },

    // ---- Horloge ----
    majHeure() {
      this.heureActuelle = new Date().toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit' });
    },

    // ---- Filtres & personnalisation ----
    dimProp(dim) {
      if (dim === 'groupes') return 'filtresGroupes';
      if (dim === 'types') return 'filtresTypes';
      return 'filtresMatieres';
    },
    estCoche(dim, val) {
      return this[this.dimProp(dim)].includes(val);
    },
    rechargerSelonFiltres() {
      this.sauvegarderFiltres();
      this.chargerCours();
      this.chargerSemaine();
    },
    basculer(dim, val) {
      const prop = this.dimProp(dim);
      const i = this[prop].indexOf(val);
      if (i === -1) this[prop].push(val);
      else this[prop].splice(i, 1);
      this.rechargerSelonFiltres();
    },
    toutCocher(dim) {
      this[this.dimProp(dim)] = [...this.categorisation[dim]];
      this.rechargerSelonFiltres();
    },
    toutDecocher(dim) {
      this[this.dimProp(dim)] = [];
      this.rechargerSelonFiltres();
    },
    async analyserCategories() {
      this.chargementCategories = true;
      this.erreurCategories = '';
      this.messageCategories = '';
      try {
        const data = await appeler('analyser_categories');
        this.categorisation = { groupes: data.groupes || [], types: data.types || [], matieres: data.matieres || [] };
        if (!this.filtresGroupes.length && this.categorisation.groupes.length) this.filtresGroupes = [...this.categorisation.groupes];
        if (!this.filtresTypes.length && this.categorisation.types.length) this.filtresTypes = [...this.categorisation.types];
        if (!this.filtresMatieres.length && this.categorisation.matieres.length) this.filtresMatieres = [...this.categorisation.matieres];
        this.messageCategories = `${this.categorisation.groupes.length} groupe(s), ${this.categorisation.types.length} type(s) et ${this.categorisation.matieres.length} matière(s) détectés dans ton flux ADE.`;
        this.rechargerSelonFiltres();
      } catch (e) {
        this.erreurCategories = this.messageErreur(e, 'Analyse impossible : vérifie ton lien ADE.');
      } finally {
        this.chargementCategories = false;
      }
    },
  },

  async created() {
    this.semaineDate = isoLocal(new Date());
    this.majHeure();
    await this.chargerPreferences();
    this.initialiserDonnees();
    appeler('version_application').then((v) => { this.versionApp = v; }).catch(() => {});
    // Actualisation automatique toutes les 60 secondes.
    this._minuteur = setInterval(() => this.chargerCours(true), 60000);
    this._horloge = setInterval(() => this.majHeure(), 30000);
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => this.appliquerTheme());
  },
  beforeUnmount() {
    clearInterval(this._minuteur);
    clearInterval(this._horloge);
  },
};
</script>
