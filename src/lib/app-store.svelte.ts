import { AccountStore } from "./account-store.svelte";
import { PostActionsStore } from "./post-actions-store.svelte";
import { SearchStore } from "./search-store.svelte";
import { DEFAULT_SITE, SITES, type Site } from "./site";
import { ViewerStore } from "./viewer-store.svelte";

class SiteContext {
  readonly site: Site;
  readonly search: SearchStore;
  readonly account: AccountStore;
  readonly viewer: ViewerStore;
  readonly postActions: PostActionsStore;

  constructor(site: Site) {
    this.site = site;
    this.search = new SearchStore(site);
    this.account = new AccountStore(site);
    this.viewer = new ViewerStore(site);
    this.postActions = new PostActionsStore(site, this.search, this.viewer);
  }
}

const STORAGE_KEY = "clowder.active-site";

function loadInitialSite(): Site {
  if (typeof localStorage === "undefined") return DEFAULT_SITE;
  const stored = localStorage.getItem(STORAGE_KEY);
  return SITES.includes(stored as Site) ? (stored as Site) : DEFAULT_SITE;
}

class AppStore {
  activeSite = $state<Site>(loadInitialSite());
  private readonly contexts: Record<Site, SiteContext> = {
    e621: new SiteContext("e621"),
    e6ai: new SiteContext("e6ai"),
  };

  get ctx(): SiteContext {
    return this.contexts[this.activeSite];
  }

  get search(): SearchStore {
    return this.ctx.search;
  }

  get account(): AccountStore {
    return this.ctx.account;
  }

  get viewer(): ViewerStore {
    return this.ctx.viewer;
  }

  get postActions(): PostActionsStore {
    return this.ctx.postActions;
  }

  contextFor(site: Site): SiteContext {
    return this.contexts[site];
  }

  setSite(site: Site) {
    if (site === this.activeSite) return;
    this.activeSite = site;
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(STORAGE_KEY, site);
    }
  }

  loadAccounts() {
    for (const site of SITES) {
      void this.contexts[site].account.load();
    }
  }
}

export const appStore = new AppStore();
